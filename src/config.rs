use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use twitch_irc::login::{CredentialsPair, StaticLoginCredentials};

use crate::{
    bot::Data,
    file_names::{CHANNELS_TO_WATCH, CREDENTIALS, MODS, TODO_SAVE},
};

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

pub async fn load_data() -> anyhow::Result<Data> {
    let save_file = tokio::fs::read_to_string(TODO_SAVE).await;

    let file_string = match save_file {
        Ok(r) => r,
        Err(_) => return Ok(Arc::new(Mutex::new(HashMap::new()))),
    };

    Ok(Arc::new(Mutex::new(serde_json::from_str(
        file_string.as_str(),
    )?)))
}

pub async fn save_data(data: &Data) -> anyhow::Result<()> {
    // das Datenobjekt muss in eine einfache Hashmap verwandelt werden, damit serde_json diesen in json verwandeln kann
    let data = data.lock().await.clone();
    let data: HashMap<&String, &Vec<String>> = data.iter().filter(|(_, v)| v.len() > 0).collect();
    let file_content = serde_json::to_string(&data)?;
    tokio::fs::write(TODO_SAVE, &file_content.as_str()).await?;
    Ok(())
}

pub struct ModSet {
    pub set: HashSet<String>,
}

impl ModSet {
    pub async fn load() -> Self {
        let set = match tokio::fs::read_to_string(MODS).await {
            Ok(r) => HashSet::from_iter(
                serde_json::from_str::<Vec<String>>(r.as_str())
                    .unwrap()
                    .into_iter(),
            ),
            Err(e) => {
                log::error!("error while loading mods file: {e}");
                HashSet::new()
            }
        };
        Self { set }
    }

    pub async fn save(&self) -> anyhow::Result<()> {
        let v: Vec<&String> = self.set.iter().collect();
        let file_content = serde_json::to_string(&v)?;
        tokio::fs::write(MODS, file_content).await?;
        Ok(())
    }

    pub fn update(&mut self, mods: Vec<String>) {
        self.set.clear();
        self.set = HashSet::from_iter(mods.into_iter());
    }
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
