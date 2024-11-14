use std::collections::{HashMap, HashSet};
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use std::time::Duration;

use config::{gen_mod_set, load_channels_to_watch, CredentialsFile};
use file_names::TODO_SAVE;
use handle_commands::{
    handle_add_todo, handle_check_command, handle_list_todos, split_command_message,
};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use tokio::sync::Mutex;
use tokio::time::sleep;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::{message, TwitchIRCClient};
use twitch_irc::{ClientConfig, SecureTCPTransport};

mod config;
mod file_names;
mod handle_commands;

/// Asynchron programmierter todo bot für Twitch
/// (Wichtig:) Von dem Asynchronen nicht beeindrucken lassen,
/// wichtig zum lesen ist nur, dass alle async functions erst ausgeführt werden,
/// wenn hinter ihnen ein ".await" aufgerufen wird (momentan ist eh nichts glechzeitig)
///

pub type Data = Arc<Mutex<HashMap<String, Vec<String>>>>;

/*
    hier sind die konstanten abgebildet, die die Kommandos (ohne Ausrufezeichen repräsentieren)
*/
const LIST_TODO_COMMAND: &str = "todos";
const ADD_TODO_COMMAND: &str = "todo";
const CHECK_TODO_COMMAND: &str = "check";
const TODO_HELP: &str = "todohelp";
const FLUSH_TODOS: &str = "todoflush";
const SAVE_TODO: &str = "savetodos";

const HELP_REPLY: &str = include_str!("./help_reply");

#[tokio::main]
pub async fn main() {
    // initialisieren eines Loggers und anderes "vorgeplänkel" (generiert logs nach /log/logfile)
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build("log/output.log")
        .unwrap();
    let config = log4rs::Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(
            Root::builder()
                .appender("logfile")
                .build(log::LevelFilter::Trace),
        )
        .unwrap();
    log4rs::init_config(config).unwrap();

    // die loop sollte "eigentlich" nur relevant werden, wenn sie nur wieder ausgeführt wird, wenn die innere Funktion abstürzt
    // solange nichts abstürzt blockt die aufgerufene Funktion loop_cycle die Ausführung
    loop {
        match loop_cycle().await {
            Err(e) => log::error!("Error: {e}"),
            Ok(()) => log::error!("internal error!"),
        }
        // warte 15 Minuten bis der Bot erneut versucht wird, zu starten (bspw. bei Internetausfall)
        sleep(Duration::from_secs(900)).await;
    }
}

pub async fn loop_cycle() -> anyhow::Result<()> {
    // die statischen Anmeldedaten aus der credentials.json werden geladen
    let config = ClientConfig::new_simple(CredentialsFile::load().await.into());
    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    let client = Arc::new(client);

    // hier werden die Daten gespeichert
    let data: Arc<Mutex<HashMap<String, Vec<String>>>> = Arc::new(Mutex::new(load_data().await?));

    let mods = gen_mod_set().await.unwrap();

    let client_clone = client.clone();

    let join_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.recv().await {
            match message {
                message::ServerMessage::Privmsg(msg) => {
                    if (&msg).message_text.starts_with("!") {
                        let (cmd, text) = split_command_message(msg.message_text.clone());

                        // hier werden die einzelnen Kommandos gecheckt
                        // wenn eines der Kommandos (bspw. ADD_TODO_COMMAND) am Anfang steht, werden diese abgehandelt

                        match cmd.as_str() {
                            ADD_TODO_COMMAND => {
                                log::info!("adding command: {:?}", &text);
                                if let Some(response) =
                                    handle_add_todo(text, data.clone(), &msg).await
                                {
                                    sleep(Duration::from_secs(1)).await;
                                    client_clone
                                        .privmsg(msg.channel_login.clone(), response)
                                        .await
                                        .unwrap();
                                }
                            }
                            LIST_TODO_COMMAND => {
                                if let Some(response) =
                                    handle_list_todos(text, data.clone(), &msg).await
                                {
                                    sleep(Duration::from_secs(1)).await;
                                    client_clone
                                        .privmsg(msg.channel_login.clone(), response)
                                        .await
                                        .unwrap();
                                }
                            }
                            CHECK_TODO_COMMAND => {
                                log::info!("checked command: {:?}", &text);
                                if let Some(response) =
                                    handle_check_command(text, data.clone(), &msg).await
                                {
                                    sleep(Duration::from_secs(1)).await;
                                    client_clone
                                        .privmsg(msg.channel_login.clone(), response)
                                        .await
                                        .unwrap();
                                }
                            }
                            TODO_HELP => {
                                sleep(Duration::from_secs(1)).await;
                                client_clone
                                    .privmsg(
                                        msg.channel_login.clone(),
                                        HELP_REPLY.to_owned().replace("\n", "  "),
                                    )
                                    .await
                                    .unwrap();
                            }
                            FLUSH_TODOS => {
                                if mods.contains(&msg.sender.login) {
                                    log::warn!(
                                        "flushed data: {}\n",
                                        data.lock()
                                            .await
                                            .drain()
                                            .map(|e| format!("[{}, {:?}]", e.0, e.1))
                                            .collect::<String>()
                                    );
                                }
                            }
                            SAVE_TODO => {
                                if mods.contains(&msg.sender.login) {
                                    match save_data(&data).await {
                                        Ok(_) => log::warn!("saved data"),
                                        Err(e) => log::error!("Error when saving todos: {e}"),
                                    }
                                }
                            }
                            _ => (),
                        }
                    }
                }
                ServerMessage::Notice(s) => {
                    // diese art von Nachricht wird vom server zurückgegeben, wenn etwas beim senden schief gelaufen ist
                    // (bspw. wenn die Anmeldung nicht funktioniert hat oder zu schnell gesendet wurde)
                    log::error!("{:?}", s)
                }
                _ => (),
            }
        }
    });

    // alle Räume, die durchscannt werden sollen, werden betreten (siehe IRC Protokoll)
    for channel in load_channels_to_watch().await {
        client.join(channel).unwrap();
    }

    // blockiere den thread und führe den Code oben aus, wenn eine Nachricht hereinkommt
    join_handle.await.unwrap();
    Ok(())
}

async fn load_data() -> anyhow::Result<HashMap<String, Vec<String>>> {
    let save_file = tokio::fs::read_to_string(TODO_SAVE).await;

    let file_string = match save_file {
        Ok(r) => r,
        Err(_) => return Ok(HashMap::new()),
    };

    Ok(serde_json::from_str(file_string.as_str())?)
}

async fn save_data(data: &Data) -> anyhow::Result<()> {
    // das Datenobjekt muss in eine einfache Hashmap verwandelt werden, damit serde_json diesen in json verwandeln kann
    let data = data.lock().await.clone();
    let file_content = serde_json::to_string(&data)?;
    tokio::fs::write(TODO_SAVE, &file_content.as_str()).await?;
    Ok(())
}
