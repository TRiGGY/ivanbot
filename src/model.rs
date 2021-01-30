use serenity::client::Context;
use serenity::model::channel::Message;
use crate::pavlov::{PavlovCommands, parse_map, parse_game_mode, parse_number, Skin, DEFAULT_MAPS, pa};
use regex::Regex;
use crate::permissions::{handle_admin, handle_mod, PermissionLevel, mod_allowed, user_allowed};
use crate::discord::{CustomFramework, ConcurrentFramework};
use crate::output::output;
use crate::config::{IvanConfig, Players, GunMode, Player, PlayerInfo, PlayerInfoContainer};
use crate::model::BotErrorKind::InvalidMapAlias;
use crate::voting::{handle_vote_start, handle_vote_finish, MAX_VOTE_MAPS};
use std::ops::{Add};
use serde::export::fmt::Display;
use serenity::http::{CacheHttp, Http};
use serenity::static_assertions::_core::fmt::Formatter;
use core::fmt;
use crate::pavlov::PavlovCommands::{SetPlayerSkin, SwitchTeam};
use rand::seq::SliceRandom;
use crate::help::{HELP_TEAM_MODE, HELP_GUNMODE, HELP_SKIN_TEAM, HELP_SKIN_MODE, HELP_CHANNEL_MODE, HELP_MAP, HELP_MAP_ARGUMENT, HELP_VOTE_ARGUMENT, HELP_VOTE_NUMBER, HELP_ALIAS_ARGUMENT, HELP_ALIAS, HELP_GAMEMODE, HELP_ALIAS_OR_MAP};

const BOT_HELP: &str =
    "
-admin [add,remove] discord_id_64 #Add/remove admin users
-mod [add,remove] discord_id_64 #Add/remove moderator users
-alias [add,remove] {url/map} alias #Create a map alias
-alias list #Show all aliases
-bothelp #Help command
-mod [add,remove] discord_id_64 #Add moderator
-map add {url/map} gamemode alias #Add map to pool
-map vote start (X) #Start map vote with X (optional) choices, default 3
-map vote stop #Conclude the map vote and switch map
-map list #List the map pool
-map default #List default maps
-team {shuffle, balance} #Will shuffle or balance the teams always creating evenly matched teams.
-gunmode {modern,ww2,random}
-skin {random, clown, prisoner, naked, farmer, russian, nato, german, soviet, us} #Change all current players to either a random skin or a specific skin
-skin shuffle {true/false} #When enabled will execute \"skin random\" 90 seconds after a vote is completed
";

#[derive(Debug, Clone)]
pub struct IvanError {
    pub(crate) input: String,
    pub(crate) kind: BotErrorKind,
}

impl Display for IvanError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.kind.to_string(), self.input)
    }
}


#[derive(Debug, Clone)]
pub enum BotErrorKind {
    InvalidCommand,
    InvalidArgument,
    MissingArgument,
    ErrorConfig,
    InvalidMapAlias,
    InvalidGameMode,
    VoteInProgress,
    VoteNotInProgress,
    CouldNotReply,
    InvalidVoteAmount,
    ConnectionError,
    Authentication,
    InvalidConnectionAddress,
    InvalidMap,
    InvalidPlayerList,
    ReadConfigError,
    SerializeError,
    DeserializeError,
    WriteError,
    MessageRetrieveError,
    MessageEditError,
}

impl Display for BotErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            BotErrorKind::InvalidArgument => "Invalid argument",
            BotErrorKind::InvalidCommand => "Invalid command try -bothelp (for bot commands) and -help (for pavlov commands)",
            BotErrorKind::MissingArgument => "Missing argument",
            BotErrorKind::ErrorConfig => "Config error",
            BotErrorKind::InvalidMapAlias => "Invalid map Alias",
            BotErrorKind::VoteInProgress => "There's already a vote in progress",
            BotErrorKind::VoteNotInProgress => "There's no vote in progress",
            BotErrorKind::CouldNotReply => "Could not reply to the channel",
            BotErrorKind::InvalidGameMode => "Invalid game mode valid valid [DM,TDM,GUN,SND]",
            BotErrorKind::InvalidVoteAmount => "Can't start a vote of this size",
            BotErrorKind::ConnectionError => "Connection error",
            BotErrorKind::Authentication => "Authentication error with password: ",
            BotErrorKind::InvalidConnectionAddress => "Connection error connecting",
            BotErrorKind::InvalidMap => "Invalid map name",
            BotErrorKind::InvalidPlayerList => "Invalid player format",
            BotErrorKind::ReadConfigError => "Error reading config",
            BotErrorKind::SerializeError => { "Config serialization error" }
            BotErrorKind::DeserializeError => "Deserialization error",
            BotErrorKind::WriteError => "Write config error",
            BotErrorKind::MessageRetrieveError => "Could not retrieve message error",
            BotErrorKind::MessageEditError => "Could not edit message",
        })
    }
}

pub fn handle_command(framework: &mut CustomFramework, mut ctx: Context, mut msg: Message, arguments: &Vec<&str>, concurrent_framework: &ConcurrentFramework, permission: PermissionLevel) {
    let tree = combine_trees(framework, &mut ctx, &mut msg, arguments, concurrent_framework, permission);
    match tree {
        Ok(_) => {}
        Err(error) => {
            output(&mut ctx, &mut msg, error.to_string())
        }
    }
}

fn combine_trees(framework: &mut CustomFramework, ctx: &mut Context, msg: &mut Message, arguments: &Vec<&str>, concurrent_framework: &ConcurrentFramework, permission: PermissionLevel) -> Result<(), IvanError> {
    let first_argument = *arguments.get(0).unwrap_or_else(|| { &"" });
    let first_argument = first_argument.to_lowercase();
    let can_execute = match &permission {
        PermissionLevel::Admin => true,
        PermissionLevel::Mod => mod_allowed(first_argument.clone()),
        PermissionLevel::User => user_allowed(first_argument.clone()),
        PermissionLevel::None => false
    };
    if !can_execute {
        output(ctx, msg, format!("You're not allowed to execute the command: {}, your rank is currently {}", first_argument, permission));
        return Ok(());
    }

    match first_argument.as_str() {
        "admin" => output(ctx, msg, handle_admin(arguments, &mut framework.config)?),
        "alias" => output(ctx, msg, handle_alias(arguments, &mut framework.config)?),
        "gunmode" => output(ctx, msg, handle_gunmode(arguments, &mut framework.config)?),
        "bothelp" => output(ctx, msg, BOT_HELP.to_string()),
        "mod" => output(ctx, msg, handle_mod(arguments, &mut framework.config)?),
        "map" => handle_map(arguments, framework, msg, ctx, concurrent_framework)?,
        "team" => output(ctx, msg, handle_team(arguments, framework)?),
        "channel" => {
            let channel = handle_channel(arguments, msg, &mut framework.config)?;
            output(ctx, msg, channel);
        }
        "skin" => output(ctx, msg, handle_skin(arguments, framework)?),
        _ => {
            let command = PavlovCommands::parse_from_arguments(arguments, &framework.config)?;
            println!("{}", &command.to_string());
            let response = framework.connection.execute_command(command);
            output(ctx, msg, response);
        }
    };
    Ok(())
}

fn handle_team(arguments: &Vec<&str>, framework: &mut CustomFramework) -> Result<String, IvanError> {
    let argument = pa(arguments, 1, HELP_TEAM_MODE)?;

    return match argument {
        "shuffle" => handle_team_shuffle(framework),
        "balance" => handle_balance(framework),
        _ => Err(IvanError { input: format!("{}", HELP_TEAM_MODE), kind: BotErrorKind::InvalidArgument }),
    };
}

fn get_player_list(framework: &mut CustomFramework) -> Result<Vec<Player>, IvanError> {
    let players_string = framework.connection.execute_command(PavlovCommands::RefreshList);
    let player = serde_json::from_str::<Players>(players_string.as_str()).map_err(|err| IvanError { input: format!("tried to get players but failed {}", err), kind: BotErrorKind::InvalidPlayerList })?;
    Ok(player.PlayerList)
}

fn inspect_all(player: Vec<Player>, framework: &mut CustomFramework) -> Result<Vec<PlayerInfo>, IvanError> {
    let player_list: Vec<Result<PlayerInfo, IvanError>> = player.iter().map(|value| {
        inspect_player(value, framework)
    }).collect();

    let error: Option<&IvanError> = player_list.iter().filter_map(|element| {
        match element {
            Err(err) => Option::Some(err),
            Ok(_) => Option::None
        }
    }).find(|_| true);

    if let Option::Some(some) = error {
        return Err(some.clone());
    } else {
        let filtered_list: Vec<PlayerInfo> = player_list.iter().filter_map(|value| { value.as_ref().map(|player| Some(player.clone())).unwrap_or_else(|_| Option::None) }).collect();
        Ok(filtered_list)
    }
}

fn handle_balance(framework: &mut CustomFramework) -> Result<String, IvanError> {
    let mut player_list = get_player_list(framework)?;
    if player_list.is_empty() { return Ok("could not shuffle teams because the server doesn't have players".to_string()); }
    player_list.shuffle(&mut rand::thread_rng());
    let inspect_list = inspect_all(player_list, framework)?;
    let balance = inspect_list.iter().fold(
        0, |first, second| -> i32 {
            let team_id: i32 = parse_number(second.TeamId.as_str()).unwrap();
            first + if team_id == 0 {
                -1
            } else {
                1
            }
        },
    ) / 2;

    if balance > 0 {
        Ok(switch_team(inspect_list, 0, balance, framework))
    } else if balance < -1 {
        Ok(switch_team(inspect_list, 1, balance * -1, framework))
    } else {
        Ok("teams are already balanced".to_string())
    }
}

fn switch_team(inspect_list: Vec<PlayerInfo>, move_to_team: u32, player_amount: i32, framework: &mut CustomFramework) -> String {
    let mut count = player_amount;
    let list: Vec<String> = inspect_list.iter().map(|value| {
        let team_id: u32 = parse_number(value.TeamId.as_str()).unwrap();
        if count > 0 && team_id != move_to_team {
            let result = team_switch(value, &move_to_team, framework)
                .unwrap_or_else(|err| err.to_string());
            count = count - 1;
            result
        } else {
            format!("")
        }
    }).collect();

    list.iter().fold("".to_string(), |a, b| format!("{}\n{}", a, b))
}

fn inspect_player(player: &Player, framework: &mut CustomFramework) -> Result<PlayerInfo, IvanError> {
    let inspect = framework.connection.execute_command(PavlovCommands::InspectPlayer(parse_number(player.UniqueId.as_str())?));
    Ok(serde_json::from_str::<PlayerInfoContainer>(inspect.as_str()).map_err(|err| IvanError { input: format!("could not parse PlayerInfo because of {}", err.to_string()), kind: BotErrorKind::InvalidPlayerList })?.PlayerInfo)
}


fn handle_team_shuffle(framework: &mut CustomFramework) -> Result<String, IvanError> {
    let mut players_result = get_player_list(framework)?;
    if players_result.is_empty() { return Ok("could not shuffle teams because the server doesn't have players".to_string()); }
    players_result.shuffle(&mut rand::thread_rng());
    let player_amount = players_result.len();
    let mut counter: i32 = player_amount as i32 / 2;
    let ids: Vec<(u32, PlayerInfo)> = players_result.iter().filter_map(|player| -> Option<(u32, PlayerInfo)> {
        counter = counter - 1;
        randomize_player(framework, player, &counter).unwrap_or(Option::None)
    }).collect();
    let message = ids.iter().map(|value| {
        let (team, player) = value;
        team_switch(player, team, framework).unwrap_or(format!("error switching team for {}", player.PlayerName))
    }).fold("".to_string(), |a, b| format!("{}\n{}", a, b)).trim().to_string();
    Ok(message)
}

fn team_switch(player: &PlayerInfo, team: &u32, framework: &mut CustomFramework) -> Result<String, IvanError> {
    let player_team: u32 = parse_number(&player.TeamId)?;
    let color = team_color(team);
    if !team.eq(&player_team) {
        let steamid: u64 = parse_number(player.UniqueId.as_str())?;
        framework.connection.execute_command(SwitchTeam(steamid, team.clone()));
        Ok(format!("{} switched to team {}", player.PlayerName, color))
    } else {
        Ok(format!("{} stays with team {}", player.PlayerName, color))
    }
}

fn team_color(team_color: &u32) -> String {
    match team_color {
        0 => "BLUE".to_string(),
        1 => "RED".to_string(),
        x => format!("unknown team {}", x)
    }
}


fn randomize_player(framework: &mut CustomFramework, player: &Player, counter: &i32) -> Result<Option<(u32, PlayerInfo)>, IvanError> {
    let inspect = framework.connection.execute_command(PavlovCommands::InspectPlayer(parse_number(player.UniqueId.as_str())?));
    let info = serde_json::from_str::<PlayerInfoContainer>(inspect.as_str()).map_err(|err| IvanError { input: format!("could not parse PlayerInfo because of {}", err.to_string()), kind: BotErrorKind::InvalidPlayerList });
    match info {
        Ok(value) => {
            let choice = if *counter >= 0 { 0 } else { 1 };
            Ok(Some((choice, value.PlayerInfo)))
        }
        Err(err) => {
            println!("encountered error {}", err);
            Ok(Option::None)
        }
    }
}


fn handle_gunmode(arguments: &Vec<&str>, config: &mut IvanConfig) -> Result<String, IvanError> {
    let argument = pa(arguments, 1, HELP_GUNMODE)?;
    let lower_argument = argument.to_lowercase();
    let gunmode = match lower_argument.as_str() {
        "modern" => GunMode::Modern,
        "ww2" => GunMode::WW2,
        "random" => GunMode::Random,
        x => return Err(
            IvanError {
                kind: BotErrorKind::InvalidArgument,
                input: format!("Invalid arguments \"{}\" {}", x, HELP_GUNMODE),
            }
        )
    };
    config.set_gun_mode(gunmode)?;
    Ok(format!("Set gunmode to {}", gunmode))
}


fn handle_skin(arguments: &Vec<&str>, framework: &mut CustomFramework) -> Result<String, IvanError> {
    match pa(arguments, 1, HELP_SKIN_TEAM)? {
        "shuffle" => handle_skin_shuffle(arguments, framework),
        "random" => {
            let random = || -> Skin {
                Skin::get_random()
            };
            assign_skins(framework, random)
        }
        "clown" => assign_skins(framework, || { Skin::Clown }),
        "prisoner" => assign_skins(framework, || { Skin::Prisoner }),
        "naked" => assign_skins(framework, || { Skin::Naked }),
        "farmer" => assign_skins(framework, || { Skin::Farmer }),
        "russian" => assign_skins(framework, || { Skin::Russian }),
        "nato" => assign_skins(framework, || { Skin::Nato }),
        "german" => assign_skins(framework, || { Skin::German }),
        x => {
            invalid_argument(x, HELP_SKIN_TEAM)
        }
    }
}

fn handle_skin_shuffle(arguments: &Vec<&str>, framework: &mut CustomFramework) -> Result<String, IvanError> {
    let argument = pa(arguments, 2, HELP_SKIN_MODE)?;
    match argument {
        "on" | "true" => {
            framework.config.set_skin_shuffle(true)?;
            Ok(format!("Skin shuffle set to true"))
        }
        "off" | "false" => {
            framework.config.set_skin_shuffle(false)?;
            Ok(format!("Skin shuffle set to false"))
        }
        x => invalid_argument(x, HELP_SKIN_MODE)
    }
}

pub fn invalid_argument(input: &str, help: &str) -> Result<String, IvanError> {
    return Err(IvanError { input: format!("\"{}\" {}", input, help), kind: BotErrorKind::InvalidArgument });
}

pub fn assign_skins(framework: &mut CustomFramework, skin_decider: fn() -> Skin) -> Result<String, IvanError> {
    let players_string = framework.connection.execute_command(PavlovCommands::RefreshList);
    let players_result = serde_json::from_str::<Players>(players_string.as_str());
    match players_result {
        Ok(players) => {
            if players.PlayerList.is_empty() {
                return Ok(format!("Could not assign skins because there are no players on the server"));
            }
            let mut msg = String::from("\n");
            for player in players.PlayerList {
                let skin = skin_decider();
                msg = msg.add(format!("Player: \"{}\" gets the skin: \"{}\"\n", player.Username, &skin).as_str());
                println!("{}", framework.connection.execute_command(SetPlayerSkin(parse_number(player.UniqueId.as_str())?, skin)));
            }
            Ok(msg)
        }
        Err(error) => {
            Err(IvanError { input: error.to_string(), kind: BotErrorKind::InvalidPlayerList })
        }
    }
}


pub fn reply(msg: &mut Message, cache_http: &Http, message: String) -> Result<Message, IvanError> {
    println!("{}", &message);
    msg.reply(cache_http, message).map_err(|_| {
        IvanError { input: "Couldn't send reply".to_string(), kind: BotErrorKind::CouldNotReply }
    })
}

fn check_alias(value: &str) -> Result<String, IvanError> {
    let regex = Regex::new("[A-z0-9]{3}[A-z0-9]*").unwrap();
    match regex.is_match(value) {
        true => Ok(value.to_string()),
        false => Err(IvanError {
            input: value.to_string(),
            kind: BotErrorKind::InvalidMapAlias,
        })
    }
}

fn handle_channel(arguments: &Vec<&str>, msg: &mut Message, config: &mut IvanConfig) -> Result<String, IvanError> {
    let argument = pa(arguments, 1, HELP_CHANNEL_MODE)?;
    match argument {
        "lock" => {
            config.add_channel_lock(msg.channel_id.0)?;
            Ok(format!("Locked bot channel to: {}", msg.channel_id))
        }
        "unlock" => {
            config.remove_channel_lock()?;
            Ok("removed channel lock".to_string())
        }
        x => invalid_argument(x, HELP_CHANNEL_MODE)
    }
}


fn handle_map(arguments: &Vec<&str>, framework: &mut CustomFramework, msg: &mut Message, ctx: &mut Context, concurrent_framework: &ConcurrentFramework) -> Result<(), IvanError> {
    let first = pa(arguments, 1, HELP_MAP_ARGUMENT)?;
    match first.to_lowercase().as_str() {
        "add" => map_add(arguments, framework, msg, ctx),
        "remove" => map_remove(arguments, framework, msg, ctx),
        "vote" => handle_vote(arguments, framework, msg, ctx, concurrent_framework),
        "list" => handle_map_pool(framework, msg, ctx),
        "default" => Ok(output(ctx, msg, format_default_maps())),
        command => {
            invalid_argument(command, HELP_MAP_ARGUMENT)?;
            Ok(())
        }
    }
}

fn format_default_maps() -> String {
    DEFAULT_MAPS.iter().fold("".to_string(), |a, b| {
        format!("{}\n{}", a, b)
    })
}

fn handle_map_pool(framework: &mut CustomFramework, msg: &mut Message, ctx: &Context) -> Result<(), IvanError> {
    let maps = framework.config.get_maps();
    let message = "The map pool is currently:\n".to_string().add(make_message(maps).as_str());
    reply(msg, ctx.http(), format!("{}", message))?;
    Ok(())
}

fn make_message<T: Display>(maps: &Vec<T>) -> String {
    let message = maps.iter().fold("".to_string(), |a, b| { format!("{}\n{}", a, b.to_string()) });
    message
}

fn handle_vote(arguments: &Vec<&str>, framework: &mut CustomFramework, msg: &mut Message, ctx: &mut Context, concurrent_framework: &ConcurrentFramework) -> Result<(), IvanError> {
    let second = pa(arguments, 2, HELP_VOTE_ARGUMENT)?;
    let choices = pa(arguments, 3, HELP_VOTE_NUMBER);
    let amount = match choices {
        Ok(value) => parse_number(value).map_err(|err| {
            IvanError { input: err.input, kind: BotErrorKind::InvalidArgument }
        })?,
        Err(_) => MAX_VOTE_MAPS
    };
    if amount < 2 {
        return Err(IvanError { input: "you need at least 2 maps to choose from".to_string(), kind: BotErrorKind::InvalidVoteAmount });
    }
    match second {
        "start" => Ok(handle_vote_start(framework, msg, ctx, concurrent_framework, amount as usize)?),
        "stop" => Ok(handle_vote_finish(framework, msg, &ctx.http)?),
        command => {
            invalid_argument(command, HELP_VOTE_ARGUMENT)?;
            Ok(())
        }
    }
}


fn handle_alias(arguments: &Vec<&str>, config: &mut IvanConfig) -> Result<String, IvanError> {
    let mode = pa(arguments, 1, HELP_ALIAS_ARGUMENT)?;
    match mode {
        "add" => {
            let map = parse_map(pa(arguments, 2, HELP_MAP)?, config).map_err(|err| {
                IvanError { kind: InvalidMapAlias, input: err.input }
            })?;
            let alias = check_alias(pa(arguments, 3, HELP_ALIAS)?)?;
            config.add_alias(alias.clone(), map.clone())?;
            Ok(format!("alias \"{}\" to map \"{}\" created", alias, map))
        }
        "remove" => {
            let argument = pa(arguments, 2, HELP_ALIAS)?;
            config.remove_alias(argument.to_string())?;
            Ok(format!("alias \"{}\" removed (if it existed)", argument))
        }
        "list" => {
            Ok(make_message(&config.get_alias_list()))
        }
        x => invalid_argument(x, HELP_ALIAS_ARGUMENT)
    }
}

fn map_add(arguments: &Vec<&str>, framework: &mut CustomFramework, msg: &mut Message, ctx: &Context) -> Result<(), IvanError> {
    let map = parse_map(pa(arguments, 2,HELP_MAP)?, &framework.config)?;
    let gamemode = parse_game_mode(pa(arguments, 3,HELP_GAMEMODE)?).map_err(|err| {
        IvanError { input: err.input, kind: BotErrorKind::InvalidGameMode }
    })?;
    let alias = check_alias(pa(arguments, 4,HELP_ALIAS)?)?;
    framework.config.add_alias(alias.to_string(), map.clone())?;
    framework.config.add_map(map.clone(), gamemode.clone(), alias.to_string())?;
    reply(msg, ctx.http(), format!("Map added to pool with id: \"{}\",gamemode: \"{}\" alias: \"{}\"", map, gamemode, alias))?;
    Ok(())
}

fn map_remove(arguments: &Vec<&str>, framework: &mut CustomFramework, msg: &mut Message, ctx: &Context) -> Result<(), IvanError> {
    if framework.vote.is_some() {
        return Err(IvanError { input: "Can't remove a map when a vote is in progress".to_string(), kind: BotErrorKind::VoteInProgress }.into());
    }
    let alias_or_map = pa(arguments, 2,HELP_ALIAS_OR_MAP)?;
    framework.config.remove_alias(alias_or_map.to_string())?;
    framework.config.remove_map(alias_or_map.to_string())?;
    handle_map_pool(framework, msg, ctx)
}

