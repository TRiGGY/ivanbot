use serenity::client::Context;
use serenity::model::channel::Message;
use crate::pavlov::{PavlovCommands, PavlovError, parse_map, parse_game_mode, parse_number, Skin, ErrorKind};
use regex::Regex;
use crate::parsing::{pa};
use crate::permissions::{handle_admin, handle_mod, PermissionLevel, mod_allowed, user_allowed};
use crate::discord::{CustomFramework, ConcurrentFramework};
use crate::model::IvanError::{BotPavlovError, BotCommandError, BotConfigError};
use crate::output::output;
use crate::config::{IvanConfig, ConfigError, Players};
use crate::model::BotErrorKind::InvalidMapAlias;
use crate::voting::{handle_vote_start, handle_vote_finish, convert_to_not_found, MAX_VOTE_MAPS};
use std::ops::Add;
use serde::export::fmt::Display;
use serenity::http::{CacheHttp, Http};
use serenity::static_assertions::_core::fmt::Formatter;
use core::fmt;
use crate::pavlov::PavlovCommands::SetPlayerSkin;
use std::error::Error;
use std::time::Duration;
use std::thread::sleep;

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
-map list
-skin {random, clown, prisoner, naked, farmer, russian, nato} #Change all current players to either a random skin or a specific skin
-skin shuffle {true/false} #When enabled will execute \"skin random\" 90 seconds after a vote is completed
";

#[derive(Debug, Clone)]
pub struct AdminCommandError {
    pub(crate) input: String,
    pub(crate) kind: BotErrorKind,
}

impl Display for AdminCommandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} because of: \"{}\"", self.kind.to_string(), self.input)
    }
}

impl Error for AdminCommandError {}


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
}

impl Display for BotErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            BotErrorKind::InvalidArgument => "Invalid argument",
            BotErrorKind::InvalidCommand => "Invalid command try -bothelp (for bot commands) and -help (for pavlov commands)",
            BotErrorKind::MissingArgument => "Missing argument {}",
            BotErrorKind::ErrorConfig => "Missing argument {}",
            BotErrorKind::InvalidMapAlias => "Invalid map Alias",
            BotErrorKind::VoteInProgress => "There's already a vote in progress",
            BotErrorKind::VoteNotInProgress => "There's no vote in progress",
            BotErrorKind::CouldNotReply => "Could not reply to the channel",
            BotErrorKind::InvalidGameMode => "Invalid game mode valid valid [DM,TDM,GUN,SND]",
            BotErrorKind::InvalidVoteAmount => "Can't start a vote of this size"
        })
    }
}

pub enum IvanError {
    BotCommandError(AdminCommandError),
    BotPavlovError(PavlovError),
    BotConfigError(ConfigError),
}

impl Display for IvanError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            BotCommandError(admin) => admin.to_string(),
            BotPavlovError(pavlov) => pavlov.to_string(),
            BotConfigError(config) => config.to_string()
        })
    }
}

impl From<PavlovError> for IvanError {
    fn from(bot_command: PavlovError) -> Self {
        BotPavlovError(bot_command)
    }
}

impl From<ConfigError> for IvanError {
    fn from(bot_command: ConfigError) -> Self {
        BotConfigError(bot_command)
    }
}

impl From<AdminCommandError> for IvanError {
    fn from(bot_command: AdminCommandError) -> Self {
        BotCommandError(bot_command)
    }
}

pub fn handle_command(framework: &mut CustomFramework, mut ctx: Context, mut msg: Message, arguments: &Vec<&str>, concurrent_framework: &ConcurrentFramework, permission: PermissionLevel) {
    let tree = combine_trees(framework, &mut ctx, &mut msg, arguments, concurrent_framework, permission);
    match tree {
        Ok(_) => {}
        Err(error) => {
            match error {
                BotCommandError(admin_error) => {
                    output(&mut ctx, &mut msg, admin_error.to_string())
                }
                BotPavlovError(pavlov_error) => {
                    output(&mut ctx, &mut msg, pavlov_error.to_string());
                }
                BotConfigError(config_error) => {
                    output(&mut ctx, &mut msg, config_error.to_string())
                }
            }
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
        "bothelp" => output(ctx, msg, BOT_HELP.to_string()),
        "mod" => output(ctx, msg, handle_mod(arguments, &mut framework.config)?),
        "map" => handle_map(arguments, framework, msg, ctx, concurrent_framework)?,
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

fn handle_skin(arguments: &Vec<&str>, framework: &mut CustomFramework) -> Result<String, IvanError> {
    match pa(arguments, 1)? {
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
        _ => {
            Err(IvanError::from(AdminCommandError { kind: BotErrorKind::InvalidArgument, input: String::from("") }))
        }
    }
}

fn handle_skin_shuffle(arguments: &Vec<&str>, framework: &mut CustomFramework) -> Result<String, IvanError> {
    let argument = pa(arguments, 2)?;
    match argument {
        "on" | "true" => {
            framework.config.set_skin_shuffle(true)?;
            Ok(format!("Skin shuffle set to true"))
        }
        "off" | "false" => {
            framework.config.set_skin_shuffle(false)?;
            Ok(format!("Skin shuffle set to false"))
        }
        x => { Err(BotCommandError(AdminCommandError { input: x.to_string(), kind: BotErrorKind::InvalidArgument })) }
    }
}

pub fn assign_skins(framework: &mut CustomFramework, skin_decider: fn() -> Skin) -> Result<String, IvanError> {
    let players_string = framework.connection.execute_command(PavlovCommands::RefreshList);
    let players_result = serde_json::from_str::<Players>(players_string.as_str());
    match players_result {
        Ok(players) => {
            if players.PlayerList.is_empty() {
                return Ok(format!("Could not assign skins because there are no players on the server"))
            }
            let mut msg = String::from("\n");
            for player in players.PlayerList {
                let skin = skin_decider();
                msg = msg.add(format!("Player: \"{}\" gets the skin: \"{}\"\n", player.Username, &skin).as_str());
                println!("{}", framework.connection.execute_command(SetPlayerSkin(parse_number(player.UniqueId.as_str())?, skin)));
                sleep(Duration::from_secs(1));
            }
            Ok(msg)
        }
        Err(error) => {
            Err(IvanError::from(PavlovError { input: error.to_string(), kind: ErrorKind::InvalidPlayerList }))
        }
    }
}


pub fn reply(msg: &mut Message, cache_http: &Http, message: String) -> Result<Message, AdminCommandError> {
    println!("{}", &message);
    msg.reply(cache_http, message).map_err(|_| {
        AdminCommandError { input: "Couldn't send reply".to_string(), kind: BotErrorKind::CouldNotReply }
    })
}

fn check_alias(value: &str) -> Result<String, AdminCommandError> {
    let regex = Regex::new("[A-z0-9]{3}[A-z0-9]*").unwrap();
    match regex.is_match(value) {
        true => Ok(value.to_string()),
        false => Err(AdminCommandError {
            input: value.to_string(),
            kind: BotErrorKind::InvalidMapAlias,
        })
    }
}

fn handle_channel(arguments: &Vec<&str>, msg: &mut Message, config: &mut IvanConfig) -> Result<String, IvanError> {
    let argument = pa(arguments, 1)?;
    match argument {
        "lock" => {
            config.add_channel_lock(msg.channel_id.0)?;
            Ok(format!("Locked bot channel to: {}", msg.channel_id))
        }
        "unlock" => {
            config.remove_channel_lock()?;
            Ok("removed channel lock".to_string())
        }
        x => Err(BotCommandError(AdminCommandError { input: x.to_string(), kind: BotErrorKind::InvalidArgument }))
    }
}


fn handle_map(arguments: &Vec<&str>, framework: &mut CustomFramework, msg: &mut Message, ctx: &mut Context, concurrent_framework: &ConcurrentFramework) -> Result<(), IvanError> {
    let first = pa(arguments, 1)?;
    match first.to_lowercase().as_str() {
        "add" => map_add(arguments, framework, msg, ctx),
        "remove" => map_remove(arguments, framework, msg, ctx),
        "vote" => handle_vote(arguments, framework, msg, ctx, concurrent_framework),
        "list" => handle_map_pool(framework, msg, ctx),
        command => Err(BotCommandError(AdminCommandError { input: command.to_string(), kind: BotErrorKind::InvalidCommand }))
    }
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
    let second = pa(arguments, 2)?;
    let choices = pa(arguments, 3);
    let amount = match choices {
        Ok(value) => parse_number(value).map_err(|err| {
            AdminCommandError { input: err.input, kind: BotErrorKind::InvalidArgument }
        })?,
        Err(_) => MAX_VOTE_MAPS
    };
    if amount < 2 {
        return Err(BotCommandError(AdminCommandError { input: "you need at least 2 maps to choose from".to_string(), kind: BotErrorKind::InvalidVoteAmount }));
    }
    match second {
        "start" => Ok(handle_vote_start(framework, msg, ctx, concurrent_framework, amount as usize)?),
        "stop" => Ok(handle_vote_finish(framework, msg, &ctx.http)?),
        command => { Err(BotCommandError(AdminCommandError { input: command.to_string(), kind: BotErrorKind::InvalidCommand })) }
    }
}


fn handle_alias(arguments: &Vec<&str>, config: &mut IvanConfig) -> Result<String, IvanError> {
    let mode = pa(arguments, 1)?;
    match mode {
        "add" => {
            let map = parse_map(pa(arguments, 2)?, config).map_err(|err| {
                AdminCommandError { kind: InvalidMapAlias, input: err.input }
            })?;
            let alias = check_alias(pa(arguments, 3)?)?;
            config.add_alias(alias.clone(), map.clone())?;
            Ok(format!("alias \"{}\" to map \"{}\" created", alias, map))
        }
        "remove" => {
            let argument = pa(arguments, 2)?;
            config.remove_alias(argument.to_string())?;
            Ok(format!("alias \"{}\" removed (if it existed)", argument))
        }
        "list" => {
            Ok(make_message(&config.get_alias_list()))
        }
        _ => Err(AdminCommandError {
            input: mode.to_string(),
            kind: BotErrorKind::InvalidArgument,
        }.into())
    }
}

fn map_add(arguments: &Vec<&str>, framework: &mut CustomFramework, msg: &mut Message, ctx: &Context) -> Result<(), IvanError> {
    let map = convert_error_not_found(parse_map(pa(arguments, 2)?, &framework.config))?;
    let gamemode = parse_game_mode(pa(arguments, 3)?).map_err(|err| {
        AdminCommandError { input: err.input, kind: BotErrorKind::InvalidGameMode }
    })?;
    let alias = check_alias(pa(arguments, 4)?)?;
    framework.config.add_alias(alias.to_string(), map.clone())?;
    framework.config.add_map(map.clone(), gamemode.clone(), alias.to_string())?;
    reply(msg, ctx.http(), format!("Map added to pool with id: \"{}\",gamemode: \"{}\" alias: \"{}\"", map, gamemode, alias))?;
    Ok(())
}

fn map_remove(arguments: &Vec<&str>, framework: &mut CustomFramework, msg: &mut Message, ctx: &Context) -> Result<(), IvanError> {
    if framework.vote.is_some() {
        return Err(AdminCommandError { input: "Can't remove a map when a vote is in progress".to_string(), kind: BotErrorKind::VoteInProgress }.into());
    }
    let alias_or_map = pa(arguments, 2)?;
    framework.config.remove_alias(alias_or_map.to_string())?;
    framework.config.remove_map(alias_or_map.to_string())?;
    handle_map_pool(framework, msg, ctx)
}

pub fn convert_error_not_found<T>(result: Result<T, PavlovError>) -> Result<T, AdminCommandError> {
    result.map_err(convert_to_not_found())
}

