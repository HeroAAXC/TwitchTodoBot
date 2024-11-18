use std::{collections::HashMap, sync::Arc};

use handle_commands::{
    handle_add_todo, handle_check_command, handle_list_todos, split_command_message,
};
use tokio::{
    sync::{
        mpsc::{Sender, UnboundedReceiver},
        Mutex,
    },
    task::JoinHandle,
};
use twitch_irc::message::{self, ServerMessage};

use crate::{
    communication::{BotMessage, TodoUpdate},
    config::{save_data, ModSet},
    lang::lang,
};

mod handle_commands;

pub use handle_commands::hash_message;

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

pub fn create_bot_worker(
    mut incoming_messages: UnboundedReceiver<ServerMessage>,
    client: Sender<BotMessage>,
    data: Data,
    mods: Arc<Mutex<ModSet>>,
    todo_subscribers: Arc<Mutex<Vec<Sender<TodoUpdate>>>>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        // hier werden die Daten gespeichert

        while let Some(message) = incoming_messages.recv().await {
            match message {
                message::ServerMessage::Privmsg(msg) => {
                    if (&msg).message_text.starts_with("!") {
                        let (cmd, text) = split_command_message(msg.message_text.clone());

                        // hier werden die einzelnen Kommandos gecheckt
                        // wenn eines der Kommandos (bspw. ADD_TODO_COMMAND) am Anfang steht, werden diese abgehandelt

                        if let Some(response) = match cmd.as_str() {
                            ADD_TODO_COMMAND => {
                                log::info!("adding command: {:?}", &text);
                                handle_add_todo(text, data.clone(), &msg, &todo_subscribers).await
                            }
                            LIST_TODO_COMMAND => handle_list_todos(text, data.clone(), &msg).await,
                            CHECK_TODO_COMMAND => {
                                log::info!("checked command: {:?}", &text);
                                handle_check_command(text, data.clone(), &msg, &todo_subscribers).await
                            }
                            TODO_HELP => Some(lang::HELP_REPLY.to_owned()),
                            FLUSH_TODOS => {
                                if mods.lock().await.set.contains(&msg.sender.login) {
                                    log::warn!(
                                        "flushed data: {}\n",
                                        data.lock()
                                            .await
                                            .drain()
                                            .map(|e| format!("[{}, {:?}]", e.0, e.1))
                                            .collect::<String>()
                                    );
                                }
                                Some(lang::FLUSHED_TODOS.to_owned())
                            }
                            SAVE_TODO => {
                                if mods.lock().await.set.contains(&msg.sender.login) {
                                    match save_data(&data).await {
                                        Ok(_) => {
                                            log::warn!("saved data");
                                            Some(lang::SAVED_DATA.to_owned())
                                        }
                                        Err(e) => {
                                            log::error!("Error when saving todos: {e}");
                                            Some(lang::ERROR_WHEN_SAVING_DATA.to_owned())
                                        }
                                    }
                                } else {
                                    Some(lang::NO_PERMISSION.to_owned())
                                }
                            }
                            _ => None,
                        } {
                            client
                                .send(BotMessage {
                                    reciever: Some(msg.sender.login),
                                    message: response,
                                    channel: msg.channel_login,
                                })
                                .await
                                .unwrap();
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
    })
}
