use serenity::client::Context;
use serenity::model::channel::Message;
use crate::pavlov::{PavlovCommands, PavlovError, ErrorKind, parse_map};
use crate::connect::{get_error_pavlov, get_error_botcommand};
use regex::Regex;
use crate::parsing::{pa, parse_discord_id};
use crate::permissions::{handle_admin, handle_mod};
use crate::discord::CustomFramework;
use crate::model::IvanError::{BotPavlovError, BotCommandError};
use crate::output::output;
use crate::config::IvanConfig;
use crate::model::BotErrorKind::InvalidMapAlias;
use crate::voting::{handle_vote_start, handle_vote_finish, convert_to_not_found};

#[derive(Debug, Clone)]
pub struct AdminCommandError {
    pub(crate) input: String,
    pub(crate) kind: BotErrorKind,
}

#[derive(Debug, Clone)]
pub enum BotErrorKind {
    InvalidCommand,
    InvalidArgument,
    MissingArgument,
    ErrorConfig,
    InvalidMapAlias,
    VoteInProgress,
    VoteNotInProgress,
    CouldNotReply,
}

enum IvanError {
    BotCommandError(AdminCommandError),
    BotPavlovError(PavlovError),
}

impl From<PavlovError> for IvanError {
    fn from(bot_command: PavlovError) -> Self {
        BotPavlovError(bot_command)
    }
}

impl From<AdminCommandError> for IvanError {
    fn from(bot_command: AdminCommandError) -> Self {
        BotCommandError(bot_command)
    }
}

pub fn handle_command( framework: &mut CustomFramework, ctx: &mut Context, msg: &mut Message, arguments: &Vec<&str>) {
    let first_argument = *arguments.get(0).unwrap_or_else(|| { &"" });

    let tree = combine_trees(framework, ctx, msg, arguments, first_argument);
    match tree {
        Ok(x) => {}
        Err(error) => {
            match error {
                IvanError::BotCommandError(admin_error) => {
                    let bot_error = get_error_botcommand(&admin_error);
                    output(ctx, msg, bot_error)
                }
                IvanError::BotPavlovError(pavlov_error) => {
                    let (error_message, _) = get_error_pavlov(&pavlov_error);
                    output(ctx, msg, error_message);
                }
            }
        }
    }
}

fn combine_trees(framework:&mut  CustomFramework, mut ctx: &mut Context, msg: &mut Message, arguments: &Vec<&str>, first_argument: &str) -> Result<(), IvanError> {
    match first_argument.to_lowercase().as_str() {
        "admin" => output(ctx, msg, handle_admin(arguments, &mut framework.config)?),
        "alias" => output(ctx, msg, handle_alias(arguments, &mut framework.config)?),
        "mod" => output(ctx, msg, handle_mod(arguments, &mut framework.config)?),
        "map" => handle_map(arguments, framework, msg, ctx)?,
        _ => {
            let command = PavlovCommands::parse_from_arguments(arguments, &framework.config)?;
            println!("{}", &command.to_string());
            framework.sender.send(command).unwrap();
            output(ctx, msg, framework.receiver.recv().unwrap());
        }
    };
    Ok(())
}

fn handle_command_2() {}


pub fn reply(msg: &mut Message, ctx: &mut Context, message: String) -> Result<Message, AdminCommandError> {
    msg.reply(ctx, message).map_err(|err| {
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

fn handle_map(arguments: &Vec<&str>, framework: &mut CustomFramework, msg: &mut Message, ctx: &mut Context) -> Result<(), AdminCommandError> {
    let first = pa(arguments, 1)?;
    match first.to_lowercase().as_str() {
        "add" => map_add(arguments, framework, msg, ctx),
        "vote" => handle_vote(arguments, framework, msg, ctx),
        command => { Err(AdminCommandError { input: command.to_string(), kind: BotErrorKind::InvalidCommand }) }
    }
}

fn handle_vote(arguments: &Vec<&str>, framework: &mut CustomFramework, msg: &mut Message, ctx: &mut Context) -> Result<(), AdminCommandError> {
    let second = pa(arguments, 2)?;
    match second {
        "start" => handle_vote_start(framework, msg, ctx),
        "finish" => handle_vote_finish(framework, msg, ctx),
        command => { Err(AdminCommandError { input: command.to_string(), kind: BotErrorKind::InvalidCommand }) }
    }
}

fn handle_alias(arguments: &Vec<&str>, config: &mut IvanConfig) -> Result<String, AdminCommandError> {
    let mode = pa(arguments, 1)?;
    match mode {
        "add" => {
            let map = parse_map(pa(arguments, 2)?, config).map_err(|err| {
                AdminCommandError { kind: InvalidMapAlias, input: err.input }
            })?;
            let alias = check_alias(pa(arguments, 3)?)?;
            config.add_alias(alias.clone(), map.clone());
            Ok(format!("alias \"{}\" to map \"{}\" created", alias, map))
        }
        _ => Err(AdminCommandError {
            input: mode.to_string(),
            kind: BotErrorKind::InvalidArgument,
        })
    }
}

fn map_add(arguments: &Vec<&str>, framework: &mut CustomFramework, msg: &mut Message, ctx: &mut Context) -> Result<(), AdminCommandError> {
    let map = convert_error_not_found(parse_map(pa(arguments, 2)?, &framework.config))?;
    let gamemode = pa(arguments, 3)?;
    let alias = pa(arguments, 4)?;
    framework.config.add_alias(alias.to_string(), map.clone());
    framework.config.add_map(map.clone(), gamemode.to_string(), alias.to_string());
    reply(msg, ctx, format!("Map added to pool with id: \"{}\",gamemode: \"{}\" alias: \"{}\"", map, gamemode, alias));
    Ok(())
}

pub fn convert_error_not_found<T>(result: Result<T, PavlovError>) -> Result<T, AdminCommandError> {
    result.map_err(convert_to_not_found())
}

