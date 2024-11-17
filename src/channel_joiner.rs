use std::{collections::BTreeSet, sync::Arc};

use anyhow::Result;
use tokio::sync::Mutex;
use twitch_irc::{
    login::LoginCredentials,
    message::{IRCMessage, IRCTags},
    transport::Transport,
    TwitchIRCClient,
};

use crate::file_names;

pub struct ChannelJoiner<T, C>
where
    T: Transport,
    C: LoginCredentials,
{
    client: Arc<Mutex<TwitchIRCClient<T, C>>>,
    channels: BTreeSet<String>,
}

impl<T, C> ChannelJoiner<T, C>
where
    T: Transport,
    C: LoginCredentials,
{
    #[allow(dead_code)]
    pub fn new(client: Arc<Mutex<TwitchIRCClient<T, C>>>) -> Self {
        Self {
            client,
            channels: BTreeSet::new(),
        }
    }

    pub async fn load(client: Arc<Mutex<TwitchIRCClient<T, C>>>) -> Self {
        let mut channels = BTreeSet::new();
        for channel in tokio::fs::read_to_string(file_names::CHANNELS_TO_WATCH)
            .await
            .unwrap()
            .split("\n")
        {
            let channel = channel.trim().replace(",", "");
            if channel != "".to_owned() {
                channels.insert(channel.clone());
                client.lock().await.join(channel).unwrap();
            }
        }

        Self { client, channels }
    }

    pub async fn update_channels(&mut self, channels: Vec<String>) -> anyhow::Result<()> {
        let new_channels = BTreeSet::from_iter(channels.clone().into_iter());
        let channels_to_disconnect = self
            .channels
            .difference(&new_channels)
            .map(|s| s.clone())
            .collect::<Vec<String>>();
        let channels_to_connect: Vec<String> = new_channels
            .difference(&self.channels)
            .map(|s| s.clone())
            .collect();

        for channel in channels_to_connect {
            match self.client.lock().await.join(channel.clone()) {
                Ok(_) => {
                    self.channels.insert(channel.clone());
                }
                Err(e) => {
                    log::error!("Error when joining channels: {e}");
                }
            }
        }

        for channel in channels_to_disconnect {
            self.client
                .lock()
                .await
                .send_message(IRCMessage::new(
                    IRCTags::new(),
                    None,
                    format!("part #{}", channel),
                    vec![],
                ))
                .await
                .unwrap();
        }

        return self.save_to_file().await;
    }

    pub fn channels(&self) -> Vec<String> {
        self.channels.iter().map(|s| s.clone()).collect()
    }

    pub async fn save_to_file(&self) -> Result<()> {
        let mut out = String::new();
        for channel in &self.channels {
            out.push_str(channel.as_str());
            out.push('\n');
        }
        tokio::fs::write(file_names::CHANNELS_TO_WATCH, out).await?;
        Ok(())
    }
}
