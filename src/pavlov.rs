use crate::pavlov::PavlovCommands::{Help, Ban, Kick, RotateMap, SwitchMap, Unban, GiveItem, GiveCash, GiveTeamCash, InspectPlayer, RefreshList, ServerInfo, ResetSND, SetPlayerSkin, SetLimitedAmmoType, SwitchTeam, BlackList, MapList, SetCash, ItemList, Kill, Raw};
use std::fmt::{Display, Formatter};
use core::fmt;
use crate::pavlov::GameMode::{SND, TDM, DM, GUN, CUSTOM, WW2GUN, TANKTDM};
use crate::pavlov::Skin::{Clown, Prisoner, Naked, Russian, Farmer, Nato, German, Soviet, Us};
use regex::{Regex};
use crate::pavlov::ErrorKind::{InvalidMap, InvalidArgument, MissingArgument, InvalidCommand};
use crate::config::IvanConfig;
use serde::{Serialize, Deserialize};
use serenity::static_assertions::_core::str::FromStr;
use rand::seq::SliceRandom;
use crate::discord::CustomFramework;

pub enum PavlovCommands {
    Help,
    Ban(SteamId),
    Kick(SteamId),
    Kill(SteamId),
    BlackList,
    MapList,
    Unban(SteamId),
    RotateMap,
    ItemList,
    SwitchMap {
        map: String,
        gamemode: GameMode,
    },
    SwitchTeam(SteamId, TeamId),
    GiveItem(SteamId, u32),
    GiveCash(SteamId, u32),
    SetCash(SteamId, u32),
    GiveTeamCash(TeamId, u32),
    InspectPlayer(SteamId),
    RefreshList,
    ServerInfo,
    //Disconnect,
    ResetSND,
    SetPlayerSkin(SteamId, Skin),
    SetLimitedAmmoType(String),
    Raw(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum GameMode {
    SND,
    DM,
    TDM,
    CUSTOM,
    GUN,
    WW2GUN,
    TANKTDM,
}

pub type SteamId = u64;
pub type TeamId = u32;

#[derive(Copy, Clone)]
pub enum Skin {
    Clown,
    Prisoner,
    Naked,
    Farmer,
    Russian,
    Nato,
    German,
    Soviet,
    Us,
}

const SKINS: [Skin; 9] = [
    Skin::Clown,
    Skin::Prisoner,
    Skin::Naked,
    Skin::Farmer,
    Skin::Russian,
    Skin::Nato,
    Skin::German,
    Skin::Soviet,
    Skin::Us];

impl Skin {
    pub fn get_random() -> Skin {
        SKINS.choose(&mut rand::thread_rng()).unwrap().clone()
    }
}

impl PavlovCommands {
    pub fn parse_from_arguments(arguments: &Vec<&str>, config: &IvanConfig) -> Result<PavlovCommands, PavlovError> {
        let first_argument = *arguments.get(0).unwrap_or_else(|| { &"" });
        let command = match first_argument.to_lowercase().as_str() {
            "help" => Help,
            "ban" => Ban(parse_number(pa(arguments, 1)?)?),
            "kick" => Kick(parse_number(pa(arguments, 1)?)?),
            "unban" => Unban(parse_number(pa(arguments, 1)?)?),
            "kill" => Kill(parse_number(pa(arguments, 1)?)?),
            "blacklist" => BlackList,
            "maplist" => MapList,
            "itemlist" => ItemList,
            "rotatemap" => RotateMap,
            "switchmap" => SwitchMap {
                map: parse_map(pa(arguments, 1)?, config)?,
                gamemode: parse_game_mode(pa(arguments, 2)?)?,
            },
            "switchteam" => SwitchTeam(parse_number(pa(arguments, 1)?)?, parse_team(pa(arguments, 2)?)?),
            "setcash" => SetCash(parse_number(pa(arguments, 1)?)?, parse_number(pa(arguments, 2)?)?),
            "giveitem" => GiveItem(parse_number(pa(arguments, 1)?)?, parse_number(pa(arguments, 2)?)?),
            "givecash" => GiveCash(parse_number(pa(arguments, 1)?)?, parse_number(pa(arguments, 2)?)?),
            "giveteamcash" => GiveTeamCash(parse_team(pa(arguments, 1)?)?, parse_number(pa(arguments, 2)?)?),
            "inspectplayer" => InspectPlayer(parse_number(pa(arguments, 1)?)?),
            "refreshlist" => RefreshList,
            "serverinfo" => ServerInfo,
            //"disconnect" => Disconnect,
            "resetsnd" => ResetSND,
            "setplayerskin" => SetPlayerSkin(parse_number(pa(arguments, 1)?)?, parse_skin(pa(arguments, 2)?)?),
            "setlimitedammotype" => SetLimitedAmmoType(pa(arguments, 1)?.to_string()),
            //    "raw" => Raw(handle_raw(arguments)?),
            x => return Err(PavlovError { input: x.to_string(), kind: InvalidCommand })
        };
        return Ok(command);
    }
}

pub fn pa<'a>(arguments: &Vec<&'a str>, index: usize) -> Result<&'a str, PavlovError> {
    (arguments.get(index)).ok_or_else(|| {
        PavlovError { input: "".to_string(), kind: MissingArgument }
    }).map(|value| { *value })
}

pub fn parse_number<T: FromStr>(value: &str) -> Result<T, PavlovError> {
    value.parse::<T>().map_err(|_error| {
        PavlovError { input: value.to_string(), kind: InvalidArgument }
    })
}

pub fn parse_map(value: &str, config: &IvanConfig) -> Result<String, PavlovError> {
    let map_string = match config.resolve_alias(value) {
        Some(value) => value,
        None => value.to_string()
    };

    let map = map_string.to_lowercase();
    let map_str = map.as_str();
    if is_standard_map(map_str) { return Ok(map_string); }
    let steam_workshop_regex: Regex = Regex::new("id=([0-9]+)").unwrap();
    let valid_mapname: Regex = Regex::new("[UGC]*[0-9]+").unwrap();
    if map_string.contains("steamcommunity.com") {
        let capture = steam_workshop_regex.captures_iter(map_str).next().unwrap();
        let first = capture.get(1);
        if first.is_some() {
            Ok(format!("UGC{}", parse_number::<u32>(first.unwrap().as_str())?))
        } else {
            Err(PavlovError { input: map_string.to_string(), kind: InvalidMap })
        }
    } else if valid_mapname.is_match(map_str) {
        Ok(map_string.to_string())
    } else {
        Err(PavlovError { input: map_string.to_string(), kind: InvalidMap })
    }
}

fn is_standard_map(map: &str) -> bool {
    match map {
        "datacenter" |
        "sand" |
        "bridge" |
        "containeryard" |
        "prisonbreak" |
        "hospital" |
        "killhouse" |
        "range" |
        "tutorial" |
        "station" |
        "stalingrad" |
        "santorini" |
        "industry" => true,
        _ => false
    }
}

fn parse_team(value: &str) -> Result<u32, PavlovError> {
    parse_number(value)
}

fn parse_skin<'a>(value: &str) -> Result<Skin, PavlovError> {
    let skin = match value.to_lowercase().as_str() {
        "clown" => Clown,
        "prisoner" => Prisoner,
        "naked" => Naked,
        "farmer" => Farmer,
        "russian" => Russian,
        "nato" => Nato,
        "german" => German,
        "soviet" => Soviet,
        "us" => Us,
        x => return Result::Err(PavlovError { input: x.to_string(), kind: ErrorKind::InvalidArgument })
    };
    Ok(skin)
}

pub fn parse_game_mode(value: &str) -> Result<GameMode, PavlovError> {
    let result = match value.to_lowercase().as_str() {
        "snd" => SND,
        "dm" => DM,
        "tdm" => TDM,
        "custom" => CUSTOM,
        "gun" => GUN,
        "ww2gun" => WW2GUN,
        "tanktdm" => TANKTDM,
        x => return Err(PavlovError { input: x.to_string(), kind: ErrorKind::InvalidArgument })
    };
    Ok(result)
}

fn handle_raw(arguments: &Vec<&str>) -> Result<String, PavlovError> {
    let slice = arguments[1..arguments.len()].to_vec();
    let iter = slice.iter();
    let concat = iter.fold("".to_string(), |a, b| format!("{} {}", a, b));

    let without_prefix = concat.strip_prefix(" ");
    match without_prefix {
        Some(value) => Ok(value.to_string()),
        None => Err(PavlovError {
            input: "Raw input was empty".to_string(),
            kind: ErrorKind::InvalidArgument,
        })
    }
}


impl Display for Skin {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let value = match self {
            Clown => "clown",
            Prisoner => "prisoner",
            Naked => "naked",
            Farmer => "farmer",
            Russian => "russian",
            Nato => "nato",
            German => { "german" }
            Soviet => { "soviet" }
            Us => { "us" }
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
            GUN => "GUN",
            WW2GUN => "WW2GUN",
            TANKTDM => "TANKTDM"
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
            Kill(steamid) => { format!("Kill {}", steamid) }
            Unban(steamid) => format!("Unban {}", steamid),
            BlackList => { "BlackList".to_string() }
            MapList => { "MapList".to_string() }
            RotateMap => "RotateMap".to_string(),
            SwitchMap { map, gamemode } => format!("SwitchMap {} {}", map, gamemode),
            ItemList => { "ItemList".to_string() }
            SwitchTeam(steamid, teamid) => format!("SwitchTeam {} {}", steamid, teamid),
            GiveItem(steamid, itemid) => format!("GiveItem {} {}", steamid, itemid),
            GiveCash(steamid, cashamt) => format!("GiveCash {} {}", steamid, cashamt),
            SetCash(steamid, cash_amt) => { format!("SetCash {} {}", steamid, cash_amt) }
            GiveTeamCash(teamid, cashamt) => format!("GiveTeamCash {} {}", teamid, cashamt),
            InspectPlayer(steamid) => format!("InspectPlayer {}", steamid),
            RefreshList => "RefreshList".to_string(),
            ServerInfo => "ServerInfo".to_string(),
            //Disconnect => "Disconnect".to_string(),
            ResetSND => "ResetSND".to_string(),
            SetPlayerSkin(steamid, skin) => format!("SetPlayerSkin {} {}", steamid, skin),
            SetLimitedAmmoType(ammo) => format!("SetLimitedAmmoType {}", ammo),
            Raw(string) => { format!("{}", string) }
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
    InvalidPlayerList,
}

impl Display for PavlovError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.input)
    }
}

impl ErrorKind {
    pub fn is_fatal(&self) -> bool {
        match self {
            ErrorKind::InvalidArgument => false,
            ErrorKind::InvalidCommand => false,
            ErrorKind::ConnectionError => true,
            ErrorKind::Authentication => true,
            ErrorKind::InvalidConnectionAddress => true,
            ErrorKind::MissingArgument => false,
            ErrorKind::InvalidMap => false,
            ErrorKind::InvalidPlayerList => false
        }
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}",
               match self {
                   ErrorKind::InvalidArgument => "Invalid argument",
                   ErrorKind::InvalidCommand => "Invalid command ",
                   ErrorKind::ConnectionError => "Connection error",
                   ErrorKind::Authentication => "Authentication error with password: ",
                   ErrorKind::InvalidConnectionAddress => "Connection error connecting",
                   ErrorKind::MissingArgument => "Missing argument",
                   ErrorKind::InvalidMap => "Invalid map name",
                   ErrorKind::InvalidPlayerList => "Invalid player format"
               }
        )
    }
}

