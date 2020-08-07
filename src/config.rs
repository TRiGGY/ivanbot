use confy::*;
use serde::{Serialize, Deserialize};
use std::env::var;
use rand;
use rand::seq::SliceRandom;
use crate::pavlov::GameMode;
use std::fmt::Display;
use serde::export::Formatter;
use core::fmt;
use crate::model::{AdminCommandError, BotErrorKind};

const IVAN_CONFIG: &str = "~/ivan";


#[derive(Serialize, Deserialize)]
pub struct IvanConfig {
    version: u8,
    admins: Vec<u64>,
    mods: Vec<u64>,
    aliases: Vec<(String, String)>,
    maps: Vec<PoolMap>,
    channel_lock: Option<u64>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PoolMap {
    pub map: String,
    pub gamemode: GameMode,
    pub alias: String,
}

impl Display for PoolMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "map: \"{}\" \t({}) \tgamemode: {}", self.alias, self.map, self.gamemode)
    }
}

impl IvanConfig {
    pub fn is_admin(&self, id: u64) -> bool {
        let admin = var("ADMIN_ID");
        if admin.is_ok() {
            if id.to_string().eq(&admin.unwrap()) {
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

    pub fn add_admin(&mut self, id: u64) -> Result<(), ConfyError> {
        self.admins.retain(|item| { *item != id });
        self.admins.push(id);
        let path = get_path();
        confy::store_path(path, self)
    }
    pub fn remove_admin(&mut self, id: u64) -> Result<(), ConfyError> {
        self.admins.retain(|item| { *item != id });
        let path = get_path();
        confy::store_path(path, self)
    }
    pub fn add_mod(&mut self, id: u64) -> Result<(), ConfyError> {
        self.mods.retain(|item| { *item != id });
        self.mods.push(id);
        let path = get_path();
        confy::store_path(path, self)
    }
    pub fn remove_mod(&mut self, id: u64) -> Result<(), ConfyError> {
        self.mods.retain(|item| { *item != id });
        let path = get_path();
        confy::store_path(path, self)
    }
    pub fn add_alias(&mut self, alias: String, mapname: String) {
        self.aliases.push((alias, mapname));
        let path = get_path();
        confy::store_path(path, self).unwrap();
    }
    pub fn remove_alias(&mut self, alias: String) {
        self.aliases.retain(|(key, value)| {
            *key.to_lowercase() != alias.to_lowercase() && *value != alias
        });
        let path = get_path();
        confy::store_path(path, self).unwrap();
    }
    pub fn resolve_alias(&self, alias: &str) -> Option<String> {
        self.aliases.iter().find(|(key, _)| {
            *key.to_lowercase() == alias.to_lowercase()
        }).map(|(_, value)| {
            value.clone()
        })
    }

    pub fn add_map(&mut self, map: String, gamemode: GameMode, alias: String) {
        self.maps.push(PoolMap { map, gamemode, alias });
        let path = get_path();
        confy::store_path(path, self).unwrap();
    }
    pub fn remove_map(&mut self, alias: String) {
        self.maps.retain(|map| {
            map.alias != alias && map.map != alias
        });
        let path = get_path();
        confy::store_path(path, self).unwrap();
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

    pub fn add_channel_lock(&mut self, channel_id: u64) {
        self.channel_lock = Some(channel_id);
        let path = get_path();
        confy::store_path(path, self).unwrap();
    }
    pub fn remove_channel_lock(&mut self) {
        self.channel_lock = None;
        let path = get_path();
        confy::store_path(path, self).unwrap();
    }
}

/// `MyConfig` implements `Default`
impl ::std::default::Default for IvanConfig {
    fn default() -> Self { Self { version: 3, admins: vec!(), mods: vec![], aliases: vec![], maps: vec![], channel_lock: None } }
}

pub fn get_config() -> Result<IvanConfig, confy::ConfyError> {
    let cfg: IvanConfig = confy::load_path(get_path())?;
    Ok(cfg)
}

fn get_path() -> String {
    return var("CONFIG_PATH").unwrap_or_else(|_| { String::from(IVAN_CONFIG) });
}