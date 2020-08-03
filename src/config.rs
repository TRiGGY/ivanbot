use confy::*;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use std::env::var;
use std::collections::HashMap;
use rand;
use rand::seq::SliceRandom;
use serde::ser::SerializeStruct;
use crate::pavlov::GameMode;
use std::fmt::Display;
use serde::export::Formatter;
use core::fmt;

const IVAN_CONFIG: &str = "~/ivan";

#[derive(Serialize, Deserialize)]
pub struct IvanConfig {
    version: u8,
    admins: Vec<u64>,
    mods: Vec<u64>,
    aliases: Vec<(String, String)>,
    maps: Vec<PoolMap>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PoolMap {
    pub map: String,
    pub gamemode: GameMode,
    pub alias: String,
}

impl Display for PoolMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f,"map: \"{}\" \t({}) \tgamemode: {}",self.alias,self.map,self.gamemode)
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
    pub fn allow_users() -> bool {
        let allow_users = var("ALLOW_USERS").unwrap_or_else(|err| { "false".to_string() });
        if allow_users.to_lowercase() != "true" {
            false
        } else {
            true
        }
    }

    pub fn add_admin(&mut self, id: u64) -> Result<(), ConfyError> {
        self.admins.retain(|item| { *item != id });
        self.admins.push(id);
        let path = var("CONFIG_PATH").unwrap_or_else(|err| { String::from(IVAN_CONFIG) });
        confy::store_path(path, self)
    }
    pub fn remove_admin(&mut self, id: u64) -> Result<(), ConfyError> {
        self.admins.retain(|item| { *item != id });
        let path = var("CONFIG_PATH").unwrap_or_else(|err| { String::from(IVAN_CONFIG) });
        confy::store_path(path, self)
    }
    pub fn add_mod(&mut self, id: u64) -> Result<(), ConfyError> {
        self.mods.retain(|item| { *item != id });
        self.mods.push(id);
        let path = var("CONFIG_PATH").unwrap_or_else(|err| { String::from(IVAN_CONFIG) });
        confy::store_path(path, self)
    }
    pub fn remove_mod(&mut self, id: u64) -> Result<(), ConfyError> {
        self.mods.retain(|item| { *item != id });
        let path = var("CONFIG_PATH").unwrap_or_else(|err| { String::from(IVAN_CONFIG) });
        confy::store_path(path, self)
    }
    pub fn add_alias(&mut self, alias: String, mapname: String) {
        self.aliases.push((alias, mapname));
        let path = var("CONFIG_PATH").unwrap_or_else(|err| { String::from(IVAN_CONFIG) });
        confy::store_path(path,self);
    }
    pub fn remove_alias(&mut self, alias: String) {
        self.aliases.retain(|(key,value)|{
            *key != alias && *value != alias
        })
    }
    pub fn resolve_alias(&self, alias: &str) -> Option<String> {
        self.aliases.iter().find(|(key, value)| {
            key.eq(alias)
        }).map(|(key, value)| {
            value.clone()
        })
    }

    pub fn add_map(&mut self, map: String, gamemode: GameMode, alias: String) {
        self.maps.push(PoolMap { map, gamemode, alias });
    }
    pub fn remove_map(&mut self, alias: String) {
        self.maps.retain(|map|{
            map.alias != alias && map.map != alias
        });
    }
    pub fn get_maps_random(&self, amount: usize) -> Vec<&PoolMap> {
        let value: Vec<&PoolMap> = self.maps.choose_multiple(&mut rand::thread_rng(), amount).collect();
        value.clone()
    }
    pub fn get_maps(&self) -> &Vec<PoolMap> {
        &self.maps
    }

}

/// `MyConfig` implements `Default`
impl ::std::default::Default for IvanConfig {
    fn default() -> Self { Self { version: 2, admins: vec!(), mods: vec![], aliases: vec![], maps: vec![] } }
}

pub fn get_config() -> Result<IvanConfig, confy::ConfyError> {
    let path = var("CONFIG_PATH").unwrap_or_else(|err| { String::from(IVAN_CONFIG) });
    let cfg: IvanConfig = confy::load_path(path)?;
    Ok(cfg)
}