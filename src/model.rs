use serenity::client::Context;
use serenity::model::channel::Message;
use crate::pavlov::{PavlovCommands, PavlovError, ErrorKind};
use crate::connect::{get_error_pavlov, get_error_botcommand};
use regex::Regex;
use crate::parsing::pa;
use crate::permissions::{handle_admin, handle_mod};

#[derive(Debug, Clone)]
pub struct BotCommandError {
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

pub fn handle_command(framework: &mut CustomFramework, mut ctx: &mut Context, mut msg: &mut Message, values: &Vec<&str>) {
    let first_argument = *arguments.get(0).unwrap_or_else(|| { &"" });

    let command_string =
        PavlovCommands::parse_from_arguments(&values, &framework.config);
    match
    command_string {
        Ok(command) => {
            println!("{}", &command.to_string());
            framework.sender.send(command).unwrap();
            let receive = framework.receiver.recv().unwrap();
            output(ctx, msg, receive);
        }
        Err(PavlovError { input, kind }) if kind == ErrorKind::InvalidCommand => {
            let result = match first_argument.to_lowercase().as_str() {
                "admin" => handle_admin(arguments, &mut config.config),
                "alias" => handle_alias(arguments, &mut config.config),
                "mod" => handle_mod(arguments, &mut config.config),
                "map" => handle_map(arguments, config, msg, ctx),
                _ => {
                    let (error_message, _) = get_error_pavlov(&err);
                    output(ctx, msg, error_message);
                }
            };
        }
        Err(err) => {
            let (error_message, _) = get_error_pavlov(&err);
            output(ctx, msg, error_message);
        }
    };
}

fn reply(msg: &mut Message, ctx: &mut Context, message: String) -> Result<Message, BotCommandError> {
    msg.reply(ctx, message).map_err(|err| {
        BotCommandError { input: "Couldn't send reply".to_string(), kind: BotErrorKind::CouldNotReply }
    })
}


fn check_alias(value: &str) -> Result<String, BotCommandError> {
    let regex = Regex::new("[A-z0-9]{3}[A-z0-9]*").unwrap();
    match regex.is_match(value) {
        true => Ok(value.to_string()),
        false => Err(BotCommandError {
            input: value.to_string(),
            kind: BotErrorKind::InvalidMapAlias,
        })
    }
}


fn handle_map(arguments: &Vec<&str>, framework: &mut CustomFramework, msg: &mut Message, ctx: &mut Context) -> Result<Message, BotCommandError> {
    let first = pa(arguments, 1)?;
    match first.to_lowercase().as_str() {
        "add" => handle_map_add(arguments, &framework, msg, ctx),
        "vote" => handle_vote_start(&framework, msg, ctx),
        "finish" => handle_vote_finish(&framework, msg, &ctx),
        command => { Err(BotCommandError { input: command.to_string(), kind: BotErrorKind::InvalidCommand }) }
    }
}