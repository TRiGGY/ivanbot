use confy::*;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use std::env::var;
use std::collections::HashMap;
use rand;
use rand::seq::SliceRandom;
use serde::ser::SerializeStruct;

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
    pub gamemode: String,
    pub alias: String,
}
// impl Serialize for PoolMap {
//     fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
//         S: Serializer {
//         let mut state = serializer.serialize_struct("Poolmap", 3)?;
//         state.serialize_field("map",&self.map);
//         state.serialize_field("gamemode",&self.gamemode);
//         state.serialize_field("alias",&self.alias);
//         state.end()
//     }
// }


const IVAN_CONFIG: &str = "~/ivan";

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
    }
    pub fn resolve_alias(&self, alias: &str) -> Option<String> {
        self.aliases.iter().find(|(key, value)| {
            key.eq(alias)
        }).map(|(key, value)| {
            value.clone()
        })
    }

    pub fn add_map(&mut self, map: String, gamemode: String, alias: String) {
        self.maps.push(PoolMap { map, gamemode, alias });
    }
    pub fn get_maps_random(&self, amount: usize) -> Vec<&PoolMap> {
        let value: Vec<&PoolMap> = self.maps.choose_multiple(&mut rand::thread_rng(), amount).collect();
        value.clone()
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