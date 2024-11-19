use std::sync::Arc;
use std::time::Duration;

use channel_joiner::ChannelJoiner;
use client_sender::{spawn_sender_worker, ClientSender};
use config::{load_data, save_data, CredentialsFile, ModSet};
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use tokio::runtime::Builder;
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, Mutex};
use tokio::time::sleep;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::TwitchIRCClient;
use twitch_irc::{ClientConfig, SecureTCPTransport};
use web::spawn_axum_worker;

mod bot;
mod channel_joiner;
mod client_sender;
mod communication;
mod config;
mod file_names;
mod lang;
mod web;

/// Asynchron programmierter todo bot für Twitch
/// (Wichtig:) Von dem Asynchronen nicht beeindrucken lassen,
/// wichtig zum lesen ist nur, dass alle async functions erst ausgeführt werden,
/// wenn hinter ihnen ein ".await" aufgerufen wird (momentan ist eh nichts glechzeitig)
///

pub async fn async_main() {
    // initialisieren eines Loggers und anderes "vorgeplänkel" (generiert logs nach /log/logfile)
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build("log/output.log")
        .unwrap();
    let log_level = if cfg!(debug_assertions) {
        LevelFilter::Trace
    } else {
        LevelFilter::Info
    };
    let config = log4rs::Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(log_level))
        .unwrap();
    log4rs::init_config(config).unwrap();

    loop {
        match loop_cycle().await {
            Err(e) => log::error!("Error: {e}"),
            Ok(r) => match r {
                true => return,
                false => log::error!("internal error!"),
            },
        }
        sleep(Duration::from_secs(60)).await;
    }
}

pub fn main() {
    let runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("TwitchTodo")
        .thread_stack_size(1048576 * 32)
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        async_main().await;
    });
}

pub async fn loop_cycle() -> anyhow::Result<bool> {
    // die statischen Anmeldedaten aus der credentials.json werden geladen
    let config = ClientConfig::new_simple(CredentialsFile::load().await.into());
    let (incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    let client = Arc::new(Mutex::new(client));

    let mods = Arc::new(Mutex::new(ModSet::load().await));

    let (send, recv) = mpsc::channel(30);
    let sender_worker = spawn_sender_worker(ClientSender::new(client.clone(), recv, 3));

    let data = load_data().await?;

    let todo_update_subscriber = Arc::new(Mutex::new(vec![]));

    let bot_worker = bot::create_bot_worker(
        incoming_messages,
        send,
        data.clone(),
        mods.clone(),
        todo_update_subscriber.clone(),
    );

    let channel_joiner = Arc::new(Mutex::new(ChannelJoiner::load(client.clone()).await));

    let (stop_sender, mut stop_recv): (Sender<()>, Receiver<()>) = mpsc::channel(1);

    let stop_sender = Arc::new(Mutex::new(stop_sender));

    let web_worker = spawn_axum_worker(
        channel_joiner.clone(),
        mods,
        stop_sender,
        todo_update_subscriber,
        data.clone(),
    );

    let non_blocking = tokio::spawn(async move {
        let (web, bot, send) = futures::join!(web_worker, bot_worker, sender_worker);
        web.unwrap();
        bot.unwrap();
        send.unwrap();
    });

    let blocking_thread = tokio::spawn(async move {
        let _ = stop_recv.recv().await.unwrap();
        save_data(&data).await.unwrap();
    });

    select! {
        test = blocking_thread => {
            test.unwrap();
            return Ok(true);
        }
        test = non_blocking => {
            test.unwrap();
        }

    }

    Ok(false)
}
