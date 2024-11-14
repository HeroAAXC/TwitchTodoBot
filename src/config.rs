use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use twitch_irc::login::{CredentialsPair, StaticLoginCredentials};

use crate::file_names::{CHANNELS_TO_WATCH, CREDENTIALS, MODS};

#[derive(Serialize, Deserialize)]
pub struct CredentialsFile {
    login: String,
    token: String,
}

impl CredentialsFile {
    pub async fn load() -> Self {
        let config_str = tokio::fs::read_to_string(CREDENTIALS).await.unwrap();
        serde_json::from_str(config_str.as_str()).unwrap()
    }
}

impl Into<StaticLoginCredentials> for CredentialsFile {
    fn into(self) -> StaticLoginCredentials {
        StaticLoginCredentials {
            credentials: CredentialsPair {
                login: self.login,
                token: Some(self.token),
            },
        }
    }
}

/*
    in der file channels.csv werden (durch zeilenm getrennt) alle (twitch-) Kanäle gespeichert,
    die der Bot nach Kommandos durchscannt
*/
pub async fn load_channels_to_watch() -> Vec<String> {
    let mut channels = vec![];
    for channel in tokio::fs::read_to_string(CHANNELS_TO_WATCH)
        .await
        .unwrap()
        .split("\n")
    {
        let channel = channel.trim().replace(",", "");
        if channel != "".to_owned() {
            channels.push(channel);
        }
    }
    channels
}

pub async fn gen_mod_set() -> anyhow::Result<HashSet<String>> {
    let file_content = match tokio::fs::read_to_string(MODS).await {
        Ok(r) => r,
        Err(_) => {
            // wenn die file nicht gelesen werden kann, erstelle eine neue und gib eine default config mit vanimio zurück
            let mut hashset = HashSet::new();
            hashset.insert("vanimio".to_owned());
            let _ = tokio::fs::write(MODS, serde_json::to_string(&hashset).unwrap()).await;
            return Ok(hashset);
        }
    };
    Ok(serde_json::from_str(&file_content.as_str())?)
}
