use std::env::var;
use rand;
use rand::seq::SliceRandom;
use crate::pavlov::GameMode;
use derive_more::{Display};
use serde::export::Formatter;
use core::{fmt, result};
use crate::model::{AdminCommandError, BotErrorKind};
use serde::{Deserialize, Serialize};
use std::{fs};
use serde_json::{to_string_pretty, from_str};
use std::fmt::{Error};
use dirs::home_dir;

const IVAN_CONFIG: &str = "ivan.json";

#[derive(Debug, Clone)]
pub struct ConfigError {
    pub(crate) input: String,
    pub(crate) kind: ConfigErrorKind,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> result::Result<(), Error> {
        write!(f, "Error: {} because of: {}", self.kind, self.input)
    }
}

#[derive(Debug, Clone, Display)]
pub enum ConfigErrorKind {
    ReadConfigError,
    SerializeError,
    DeserializeError,
    WriteError,
}


// impl Display for ConfigErrorKind {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", match self {
//             ConfigErrorKind::DeserializeError => { "Deserialize Error" }
//             ConfigErrorKind::ReadConfigError => { "Read Config Error" }
//             ConfigErrorKind::SerializeError => { "Serialize Error" }
//             ConfigErrorKind::WriteError => { "Write Config Error" }
//         })
//     }
// }

#[derive(Deserialize)]
pub struct Players {
    pub(crate) PlayerList: Vec<Player>
}

#[derive(Deserialize)]
pub struct Player {
    pub(crate) Username: String,
    pub(crate) UniqueId: String,
}


#[derive(Serialize, Deserialize)]
pub struct IvanConfig {
    version: u8,
    admins: Vec<u64>,
    mods: Vec<u64>,
    aliases: Vec<(String, String)>,
    maps: Vec<PoolMap>,
    channel_lock: Option<u64>,

    #[serde(default)]
    skin_shuffle: bool,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PoolMap {
    pub map: String,
    pub gamemode: GameMode,
    pub alias: String,
}

impl fmt::Display for PoolMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "map: \"{}\" \t({}) \tgamemode: {}", self.alias, self.map, self.gamemode)
    }
}

impl IvanConfig {
    pub fn is_admin(&self, id: u64) -> bool {
        let admin = var("ADMIN_ID");
        if admin.is_ok() {
            if id.to_string().eq(&admin.unwrap_or("".to_string())) {
                return true;
            }
        }
        return self.admins.contains(&id);
    }
    pub fn is_mod(&self, id: u64) -> bool {
        return self.mods.contains(&id);
    }

    pub fn allow_users() -> bool {
        let allow_users = var("ALLOW_USERS").unwrap_or_else(|_| { "false".to_string() });
        if allow_users.to_lowercase() != "true" {
            false
        } else {
            true
        }
    }

    pub fn add_admin(&mut self, id: u64) -> Result<(), ConfigError> {
        self.admins.retain(|item| { *item != id });
        self.admins.push(id);
        write_config(&self)
    }
    pub fn remove_admin(&mut self, id: u64) -> Result<(), ConfigError> {
        self.admins.retain(|item| { *item != id });
        write_config(&self)
    }
    pub fn add_mod(&mut self, id: u64) -> Result<(), ConfigError> {
        self.mods.retain(|item| { *item != id });
        self.mods.push(id);
        write_config(&self)
    }
    pub fn remove_mod(&mut self, id: u64) -> Result<(), ConfigError> {
        self.mods.retain(|item| { *item != id });
        write_config(&self)
    }
    pub fn add_alias(&mut self, alias: String, mapname: String) -> Result<(), ConfigError> {
        self.aliases.push((alias, mapname));
        write_config(&self)
    }
    pub fn remove_alias(&mut self, alias: String) -> Result<(), ConfigError> {
        self.aliases.retain(|(key, value)| {
            *key.to_lowercase() != alias.to_lowercase() && *value != alias
        });
        write_config(&self)
    }
    pub fn resolve_alias(&self, alias: &str) -> Option<String> {
        self.aliases.iter().find(|(key, _)| {
            *key.to_lowercase() == alias.to_lowercase()
        }).map(|(_, value)| {
            value.clone()
        })
    }

    pub fn add_map(&mut self, map: String, gamemode: GameMode, alias: String) -> Result<(), ConfigError> {
        self.maps.push(PoolMap { map, gamemode, alias });
        write_config(&self)
    }
    pub fn remove_map(&mut self, alias: String) -> Result<(), ConfigError> {
        self.maps.retain(|map| {
            map.alias != alias && map.map != alias
        });
        write_config(&self)
    }
    pub fn get_maps_random(&self, amount: usize) -> Result<Vec<&PoolMap>, AdminCommandError> {
        if self.maps.len() < amount {
            return Err(AdminCommandError { input: format!("{} was more than the amount of maps in the pool", amount), kind: BotErrorKind::InvalidVoteAmount });
        }
        let value: Vec<&PoolMap> = self.maps.choose_multiple(&mut rand::thread_rng(), amount).collect();
        return Ok(value.clone());
    }
    pub fn get_maps(&self) -> &Vec<PoolMap> {
        &self.maps
    }

    pub fn get_alias_list(&self) -> Vec<String> {
        self.aliases.iter().map(|(key, value)| {
            format!("alias: {} map: {}", key, value)
        }).collect()
    }

    pub fn get_channel_lock(&self) -> Option<u64> {
        return self.channel_lock;
    }

    pub fn add_channel_lock(&mut self, channel_id: u64) -> Result<(), ConfigError> {
        self.channel_lock = Some(channel_id);
        write_config(&self)
    }
    pub fn remove_channel_lock(&mut self) -> Result<(), ConfigError> {
        self.channel_lock = None;
        write_config(&self)
    }

    pub fn set_skin_shuffle(&mut self, value: bool) -> Result<(), ConfigError> {
        self.skin_shuffle = value;
        write_config(&self)
    }

    pub fn get_skin_shuffle(&self) -> bool {
        self.skin_shuffle
    }
}


pub fn get_config() -> Result<IvanConfig, ConfigError> {
    let file = fs::read_to_string(get_path()).map_err(|err| {
        ConfigError { input: err.to_string(), kind: ConfigErrorKind::ReadConfigError }
    })?;
    return from_str(file.as_str()).map_err(|err| {
        ConfigError { input: err.to_string(), kind: ConfigErrorKind::DeserializeError }
    });
}

fn write_config(config: &IvanConfig) -> Result<(), ConfigError> {
    let values = to_string_pretty(&config).map_err(|err| {
        ConfigError { input: err.to_string(), kind: ConfigErrorKind::SerializeError }
    })?;
    fs::write(get_path(), values).map_err(|err| {
        ConfigError { input: err.to_string(), kind: ConfigErrorKind::WriteError }
    })
}

/// `MyConfig` implements `Default`
impl ::std::default::Default for IvanConfig {
    fn default() -> Self { Self { version: 3, admins: vec!(), mods: vec![], aliases: vec![], maps: vec![], channel_lock: None, skin_shuffle: false } }
}

fn get_path() -> String {
    return var("CONFIG_PATH").unwrap_or_else(|_| {
        let dir = home_dir();
        match dir {
            None => { IVAN_CONFIG.to_string() }
            Some(mut path) => {
                path.push(IVAN_CONFIG);
                String::from(path.as_path().to_str().unwrap())
            }
        }
    });
}