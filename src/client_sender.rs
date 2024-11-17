use std::{sync::Arc, time::Duration};

use tokio::{
    sync::{mpsc::Receiver, Mutex},
    task::JoinHandle,
    time::sleep,
};
use twitch_irc::{login::LoginCredentials, transport::Transport, TwitchIRCClient};

use crate::communication::BotMessage;

use futures::join;

pub struct ClientSender<T, C>
where
    C: LoginCredentials,
    T: Transport,
{
    delay_secs: u64,
    client: Arc<Mutex<TwitchIRCClient<T, C>>>,
    recv: Receiver<BotMessage>,
}

impl<T, C> ClientSender<T, C>
where
    T: Transport,
    C: LoginCredentials,
{
    pub fn new(
        client: Arc<Mutex<TwitchIRCClient<T, C>>>,
        recv: Receiver<BotMessage>,
        delay_secs: u64,
    ) -> Self {
        Self {
            client,
            delay_secs,
            recv,
        }
    }

    pub async fn start(mut self) -> ! {
        let duration = Duration::from_secs(self.delay_secs);
        loop {
            let send = self.recv.recv();
            let sleep = sleep(duration);
            let (msg, _) = join!(send, sleep);

            if let Some(msg) = msg {
                let message = format!(
                    "@{} {}",
                    match msg.reciever {
                        Some(r) => r,
                        None => String::new(),
                    },
                    msg.message
                );

                self.client
                    .lock()
                    .await
                    .privmsg(msg.channel, message)
                    .await
                    .unwrap();
            }
        }
    }
}

pub fn spawn_sender_worker<T: Transport, C: LoginCredentials>(
    client_sender: ClientSender<T, C>,
) -> JoinHandle<()> {
    tokio::spawn(async move { client_sender.start().await })
}
