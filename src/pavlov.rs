use crate::pavlov::PavlovCommands::{Help, Ban, Kick, RotateMap, SwitchMap, Unban, GiveItem, GiveCash, GiveTeamCash, InspectPlayer, RefreshList, ServerInfo,  ResetSND, SetPlayerSkin, SetLimitedAmmoType, SwitchTeam};
use std::fmt::{Display, Formatter};
use core::fmt;
use crate::pavlov::GameMode::{SND, TDM, DM, GUN, CUSTOM};
use crate::pavlov::Skin::{Clown, Prisoner, Naked, Russian, Farmer, Nato};
use regex::{Regex};
use crate::pavlov::ErrorKind::{InvalidMap, InvalidArgument, MissingArgument, InvalidCommand};
use crate::config::IvanConfig;
use serde::{Serialize, Deserialize, Serializer, Deserializer};

pub enum PavlovCommands {
    Help,
    Ban(SteamId),
    Kick(SteamId),
    Unban(SteamId),
    RotateMap,
    SwitchMap {
        map: String,
        gamemode: GameMode,
    },
    SwitchTeam(SteamId, TeamId),
    GiveItem(SteamId, u32),
    GiveCash(SteamId, u32),
    GiveTeamCash(TeamId, u32),
    InspectPlayer(SteamId),
    RefreshList,
    ServerInfo,
    //Disconnect,
    ResetSND,
    SetPlayerSkin(SteamId, Skin),
    SetLimitedAmmoType(u32),
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameMode {
    SND,
    DM,
    TDM,
    CUSTOM,
    GUN,
}

pub type SteamId = u32;
pub type TeamId = u32;

pub enum Skin {
    Clown,
    Prisoner,
    Naked,
    Farmer,
    Russian,
    Nato,
}

impl PavlovCommands {
    pub fn parse_from_arguments(arguments: &Vec<&str>, config: &IvanConfig) -> Result<PavlovCommands, PavlovError> {
        let first_argument = *arguments.get(0).unwrap_or_else(|| { &"" });
        let command = match first_argument.to_lowercase().as_str() {
            "help" => Help,
            "ban" => Ban(parse_steam_id(pa(arguments,1)?)?),
            "kick" => Kick(parse_steam_id(pa(arguments,1)?)?),
            "unban" => Unban(parse_steam_id(pa(arguments,1)?)?),
            "rotatemap" => RotateMap,
            "switchmap" => SwitchMap {
                map: parse_map(pa(arguments,1)?, config)?,
                gamemode: parse_game_mode(pa(arguments,2)?)?,
            },
            "switchteam" => SwitchTeam(parse_steam_id(pa(arguments,1)?)?, parse_uint(pa(arguments,2)?)?),
            "giveitem" => GiveItem(parse_steam_id(pa(arguments,1)?)?, parse_uint(pa(arguments,2)?)?),
            "givecash" => GiveCash(parse_steam_id(pa(arguments,1)?)?, parse_uint(pa(arguments,2)?)?),
            "giveteamcash" => GiveTeamCash(parse_team(pa(arguments,1)?)?, parse_uint(pa(arguments,2)?)?),
            "inspectplayer" => InspectPlayer(parse_steam_id(pa(arguments,1)?)?),
            "refreshlist" => RefreshList,
            "serverinfo" => ServerInfo,
            //"disconnect" => Disconnect,
            "resetsnd" => ResetSND,
            "setplayerskin" => SetPlayerSkin(parse_steam_id(pa(arguments,1)?)?, parse_skin(pa(arguments,2)?)?),
            "setlimitedammotype" => SetLimitedAmmoType(parse_ammo(pa(arguments,1)?)?),
            x => return Err(PavlovError { input: x.to_string(), kind: InvalidCommand })
        };
        return Ok(command);
    }
}

pub fn pa<'a>(arguments: &Vec<&'a str>,index : usize) -> Result<&'a str, PavlovError> {
    (arguments.get(index)).ok_or_else(|| {
        PavlovError { input: "".to_string(), kind: MissingArgument }
    }).map(|value| { *value })
}

fn parse_steam_id(value: &str) -> Result<u32, PavlovError> {
    value.parse::<u32>().map_err(|_error| {
        PavlovError { input: value.to_string(), kind: InvalidArgument }
    })
}

pub fn parse_map(value: &str, config: &IvanConfig) -> Result<String, PavlovError> {
    let map_string = match config.resolve_alias(value) {
        Some(value) => value,
        None => value.to_string()
    };
    let map = map_string.as_str();


    let steam_workshop_regex: Regex = Regex::new("id=([0-9]+)").unwrap();
    let valid_mapname: Regex = Regex::new("[UGC]*[0-9]+").unwrap();
    if map_string.contains("steamcommunity.com") {
        let capture = steam_workshop_regex.captures_iter(map).next().unwrap();
        let first = capture.get(1);
        if first.is_some() {
            Ok(format!("UGC{}", parse_uint(first.unwrap().as_str())?))
        } else {
            Err(PavlovError { input: map_string.to_string(), kind: InvalidMap })
        }
    } else if valid_mapname.is_match(map) {
        Ok(map_string.to_string())
    } else {
        Err(PavlovError { input: map_string.to_string(), kind: InvalidMap })
    }
}

fn parse_team(value: &str) -> Result<u32, PavlovError> {
    parse_steam_id(value)
}

fn parse_skin<'a>(value: &str) -> Result<Skin, PavlovError> {
    let skin = match value.to_lowercase().as_str() {
        "clown" => Clown,
        "prisoner" => Prisoner,
        "naked" => Naked,
        "farmer" => Farmer,
        "russian" => Russian,
        "nato" => Nato,
        x => return Result::Err(PavlovError { input: x.to_string(), kind: ErrorKind::InvalidArgument })
    };
    Ok(skin)
}

fn parse_ammo(value: &str) -> Result<u32, PavlovError> {
    parse_steam_id(value)
}

fn parse_uint(value: &str) -> Result<u32, PavlovError> {
    parse_steam_id(value)
}

pub fn parse_game_mode(value: &str) -> Result<GameMode, PavlovError> {
    let result = match value.to_lowercase().as_str() {
        "snd" => SND,
        "dm" => DM,
        "tdm" => TDM,
        "custom" => CUSTOM,
        "gun" => GUN,
        x => return Err(PavlovError { input: x.to_string(), kind: ErrorKind::InvalidArgument })
    };
    Ok(result)
}

impl Display for Skin {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let value = match self {
            Clown => "clown",
            Prisoner => "prisoner",
            Naked => "naked",
            Farmer => "farmer",
            Russian => "russian",
            Nato => "nato"
        };
        write!(f, "{}", value)
    }
}

impl Display for GameMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let value = match self {
            SND => "SND",
            DM => "DM",
            TDM => "TDM",
            CUSTOM => "CUSTOM",
            GUN => "GUN"
        };
        write!(f, "{}", value)
    }
}

impl Display for PavlovCommands {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let command = match self {
            Help => "Help".to_string(),
            Ban(steamid) => format!("Ban {}", steamid),
            Kick(steamid) => format!("Kick {}", steamid),
            Unban(steamid) => format!("Unban {}", steamid),
            RotateMap => "RotateMap".to_string(),
            SwitchMap { map, gamemode } => format!("SwitchMap {} {}", map, gamemode),
            SwitchTeam(steamid, teamid) => format!("SwitchTeam {} {}", steamid, teamid),
            GiveItem(steamid, itemid) => format!("GiveItem {} {}", steamid, itemid),
            GiveCash(steamid, cashamt) => format!("GiveCash {} {}", steamid, cashamt),
            GiveTeamCash(teamid, cashamt) => format!("GiveTeamCash {} {}", teamid, cashamt),
            InspectPlayer(steamid) => format!("InspectPlayer {}", steamid),
            RefreshList => "RefreshList".to_string(),
            ServerInfo => "ServerInfo".to_string(),
            Disconnect => "Disconnect".to_string(),
            ResetSND => "ResetSND".to_string(),
            SetPlayerSkin(steamid, skin) => format!("SetPlayerSkin {} {}", steamid, skin),
            SetLimitedAmmoType(ammo) => format!("SetLimitedAmmoType {}", ammo),
        };
        write!(f, "{}", command)
    }
}

#[derive(Debug, Clone)]
pub struct PavlovError {
    pub(crate) input: String,
    pub(crate) kind: ErrorKind,
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    ConnectionError,
    InvalidCommand,
    InvalidArgument,
    Authentication,
    InvalidConnectionAddress,
    MissingArgument,
    InvalidMap,
}


