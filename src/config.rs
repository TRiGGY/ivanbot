use std::env::var;
use rand;
use rand::seq::{IteratorRandom};
use crate::pavlov::{GameMode};
use derive_more::{Display};
use core::{fmt};
use crate::model::{BotErrorKind, IvanError};
use serde::{Deserialize, Serialize};
use std::{fs};
use serde_json::{to_string_pretty, from_str};
use dirs::home_dir;
use std::fmt::Formatter;
use std::cmp::min;

const IVAN_CONFIG: &str = "ivan.json";

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct Players {
    pub(crate) PlayerList: Vec<Player>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Clone)]
pub struct Player {
    pub(crate) Username: String,
    pub(crate) UniqueId: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct PlayerInfoContainer {
    pub(crate) PlayerInfo: PlayerInfo,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Clone)]
pub struct PlayerInfo {
    pub(crate) PlayerName: String,
    pub(crate) UniqueId: String,
    pub(crate) KDA: String,
    pub(crate) Score: String,
    pub(crate) Cash: String,
    pub(crate) TeamId: String,

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

    #[serde(default)]
    gun_mode: GunMode,

    #[serde(default = "default_vote_count")]
    map_vote_count: u64,

    #[serde(default = "default_option_none")]
    team_channels: Option<(u64, u64)>,
}

fn default_option_none() -> Option<(u64,u64)> {
    Option::None
}

fn default_vote_count() -> u64 {
    8
}

#[derive(Serialize, Deserialize, Clone, Copy, Display, Eq, PartialEq)]
pub enum GunMode {
    Modern,
    WW2,
    Random,
    OitcRandom,
}

impl Default for GunMode {
    fn default() -> GunMode {
        GunMode::Modern
    }
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

    pub fn add_admin(&mut self, id: u64) -> Result<(), IvanError> {
        self.admins.retain(|item| { *item != id });
        self.admins.push(id);
        write_config(&self)
    }
    pub fn remove_admin(&mut self, id: u64) -> Result<(), IvanError> {
        self.admins.retain(|item| { *item != id });
        write_config(&self)
    }
    pub fn add_mod(&mut self, id: u64) -> Result<(), IvanError> {
        self.mods.retain(|item| { *item != id });
        self.mods.push(id);
        write_config(&self)
    }
    pub fn remove_mod(&mut self, id: u64) -> Result<(), IvanError> {
        self.mods.retain(|item| { *item != id });
        write_config(&self)
    }
    pub fn add_alias(&mut self, alias: String, mapname: String) -> Result<(), IvanError> {
        self.aliases.push((alias, mapname));
        write_config(&self)
    }
    pub fn remove_alias(&mut self, alias: String) -> Result<(), IvanError> {
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

    pub fn get_team_channels(&self) -> Option<(u64, u64)> {
        self.team_channels
    }
    pub fn set_team_channels(&mut self,team1 : u64, team2 : u64) -> Result<(), IvanError> {
        self.team_channels = Some((team1,team2));
        write_config(&self)
    }
    pub fn add_map(&mut self, map: String, gamemode: GameMode, alias: String) -> Result<(), IvanError> {
        self.maps.push(PoolMap { map, gamemode, alias });
        write_config(&self)
    }
    pub fn remove_map(&mut self, alias: String) -> Result<(), IvanError> {
        self.maps.retain(|map| {
            map.alias != alias && map.map != alias
        });
        write_config(&self)
    }
    pub fn get_maps_random(&self, game_mode: Option<GameMode>) -> Result<Vec<&PoolMap>, IvanError> {
        if self.maps.len() < 1 {
            return Err(IvanError { input: format!("there were no maps in the pool"), kind: BotErrorKind::InvalidVoteAmount });
        }
        let filtered_maps: Vec<&PoolMap> = self.maps.iter().filter(|map| {
            match game_mode {
                Some(value) => {
                    if value == GameMode::GUN {
                        match self.gun_mode {
                            GunMode::Random => vec![GameMode::GUN, GameMode::WW2GUN].contains(&map.gamemode),
                            GunMode::WW2 => value == GameMode::WW2GUN,
                            GunMode::OitcRandom => vec![GameMode::GUN, GameMode::WW2GUN, GameMode::OITC].contains(&map.gamemode),
                            GunMode::Modern => value == GameMode::GUN
                        }
                    } else {
                        value == map.gamemode
                    }
                }
                None => true
            }
        }).collect();

        let amount = min(filtered_maps.len(), self.map_vote_count as usize);

        if amount < 2 {
            return Err(IvanError {
                input: "Could not start map vote because the map pool didn't contain at least 2 maps (with the selected gameMode)".to_string(),
                kind: BotErrorKind::InvalidVoteAmount,
            }
            );
        }

        let value = filtered_maps.iter().map(|value| { *value }).choose_multiple(&mut rand::thread_rng(), amount);
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

    pub fn add_channel_lock(&mut self, channel_id: u64) -> Result<(), IvanError> {
        self.channel_lock = Some(channel_id);
        write_config(&self)
    }
    pub fn remove_channel_lock(&mut self) -> Result<(), IvanError> {
        self.channel_lock = None;
        write_config(&self)
    }

    pub fn set_skin_shuffle(&mut self, value: bool) -> Result<(), IvanError> {
        self.skin_shuffle = value;
        write_config(&self)
    }

    pub fn get_skin_shuffle(&self) -> bool {
        self.skin_shuffle
    }

    pub fn set_gun_mode(&mut self, value: GunMode) -> Result<(), IvanError> {
        self.gun_mode = value;
        write_config(&self)
    }

    pub fn get_gun_mode(&self) -> GunMode {
        self.gun_mode
    }

    pub fn set_vote_amount(&mut self, value: u64) -> Result<(), IvanError> {
        if value <= 10 && value >= 2 {
            self.map_vote_count = value;
            write_config(&self)
        } else {
            Result::Err(IvanError {
                input: "Invalid vote amount, it should be within 2-10 range".to_string(),
                kind: BotErrorKind::InvalidVoteAmount,
            })
        }
    }

    pub fn get_vote_amount(&self) -> u64 {
        self.map_vote_count
    }
}


pub fn get_config() -> Result<IvanConfig, IvanError> {
    let file = fs::read_to_string(get_path()).map_err(|err| {
        IvanError { input: err.to_string(), kind: BotErrorKind::ReadConfigError }
    })?;
    return from_str(file.as_str()).map_err(|err| {
        IvanError { input: err.to_string(), kind: BotErrorKind::DeserializeError }
    });
}

fn write_config(config: &IvanConfig) -> Result<(), IvanError> {
    let values = to_string_pretty(&config).map_err(|err| {
        IvanError { input: err.to_string(), kind: BotErrorKind::SerializeError }
    })?;
    fs::write(get_path(), values).map_err(|err| {
        IvanError { input: err.to_string(), kind: BotErrorKind::WriteError }
    })
}

/// `MyConfig` implements `Default`
impl ::std::default::Default for IvanConfig {
    fn default() -> Self { Self { version: 3, admins: vec!(), mods: vec![], aliases: vec![], maps: vec![], channel_lock: None, skin_shuffle: false, gun_mode: GunMode::Modern, map_vote_count: 8, team_channels: None } }
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