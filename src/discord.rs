use serenity::client::{Client, EventHandler};
use serenity::model::channel::Message;
use serenity::prelude::{Context};
use serenity::framework::Framework;
use crate::connect::{get_error_pavlov, maintain_connection, get_error_botcommand};
use crate::pavlov::{PavlovCommands};
use std::process::exit;
use std::env::{var};
use crate::credentials::{get_login};
use threadpool::ThreadPool;


use std::sync::mpsc::{Sender, Receiver};
use crate::config::{get_config, IvanConfig};


struct Handler;

impl EventHandler for Handler {}

pub fn run_discord() {
    let token = get_discord_token();
    let mut client = Client::new(&token, Handler {}).unwrap();
    let login = get_login();
    let config = get_config();
    if let Err(why) = config {
        println!("can't read config file: {}", why.to_string());
        exit(1);
    }
    let (sender, receiver) = maintain_connection(login);
    client.with_framework(
        CustomFramework {
            sender,
            receiver,
            config: config.unwrap(),
        }
    );
    if let Err(why) = client.start() {
        println!("Err with client: {:?}", why);
    }
}

struct CustomFramework {
    sender: Sender<PavlovCommands>,
    receiver: Receiver<String>,
    config: IvanConfig,
}


impl Framework for CustomFramework {
    fn dispatch(&mut self, ctx: Context, msg: Message, _: &ThreadPool) {
        if !authenticate(&msg, &self.config) {
            return;
        }
        if msg.author.bot {
            return;
        }
        if !msg.content.starts_with("-") {
            return;
        }
        let cloned = msg.content.clone();
        let stripped = cloned.trim_start_matches("-");
        let values: Vec<&str> = stripped.split_whitespace().collect();

        let bot_command = handle_bot_command(&values, &mut self.config);
        if let Some(value) = bot_command {
            match value {
                Ok(result) => output(ctx,msg,result),
                Err(error)=> output(ctx,msg,get_error_botcommand(&error))
            }
            return;
        }
        let command_string = PavlovCommands::parse_from_arguments(&values);
        match command_string {
            Ok(command) => {
                println!("{}", &command.to_string());
                self.sender.send(command).unwrap();
                let receive = self.receiver.recv().unwrap();
                output(ctx, msg, receive);
            }
            Err(err) => {
                let (error_message, _) = get_error_pavlov(&err);
                output(ctx, msg, error_message);
            }
        };
    }
}

fn authenticate(msg: &Message, config: &IvanConfig) -> bool {
    let uid = msg.author.id.0;
    config.is_admin(uid)
}

fn output(ctx: Context, msg: Message, message: String) {
    println!("{}", &message);
    msg.reply(ctx, message).unwrap();
}

fn get_discord_token() -> String {
    let discord_token = var("DISCORD_TOKEN");
    match discord_token {
        Ok(token) => token,
        Err(_) => {
            println!("could not find DISCORD_TOKEN");
            exit(1)
        }
    }
}

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
}

fn handle_bot_command(arguments: &Vec<&str>, config : &mut IvanConfig) -> Option<Result<String, BotCommandError>> {
    let first_argument = *arguments.get(0).unwrap_or_else(|| { &"" });
    match first_argument.to_lowercase().as_str() {
        "admin" => Some(handle_admin(arguments,config)),
        _ => None
    }
}

fn handle_admin(arguments: &Vec<&str>, config: &mut IvanConfig) -> Result<String, BotCommandError> {
    let mode = pa1(arguments)?;
    match mode {
        "add" => add_admin(parse_discord_id(pa2(arguments)?)?, config),
        "remove" => remove_admin(parse_discord_id(pa2(arguments)?)?, config),
        _ => Err(BotCommandError {
            input: mode.to_string(),
            kind: BotErrorKind::InvalidArgument,
        })
    }
}
fn remove_admin(id: u64, config: &mut IvanConfig) -> Result<String, BotCommandError> {
    config.remove_admin(id).map_err(|err| {
        BotCommandError {
            input: err.to_string(),
            kind: BotErrorKind::ErrorConfig,
        }
    }).map(|_| {
        format!("Removed admin with id \"{}\" from the admin list", id)
    })
}

fn add_admin(id: u64, config: &mut IvanConfig) -> Result<String, BotCommandError> {
    config.add_admin(id).map_err(|err| {
        BotCommandError {
            input: err.to_string(),
            kind: BotErrorKind::ErrorConfig,
        }
    }).map(|_| {
        format!("Added admin with id \"{}\" to the admin list", id)
    })
}


pub fn pa1<'a>(arguments: &Vec<&'a str>) -> Result<&'a str, BotCommandError> {
    (arguments.get(1)).ok_or_else(|| {
        BotCommandError { input: "".to_string(), kind: BotErrorKind::MissingArgument }
    }).map(|value| { *value })
}

pub fn pa2<'a>(arguments: &Vec<&'a str>) -> Result<&'a str, BotCommandError> {
    (arguments.get(2)).ok_or_else(|| {
        BotCommandError { input: "".to_string(), kind: BotErrorKind::MissingArgument }
    }).map(|value| { *value })
}

fn parse_discord_id(value: &str) -> Result<u64, BotCommandError> {
    value.parse::<u64>().map_err(|_error| {
        BotCommandError { input: value.to_string(), kind: BotErrorKind::InvalidArgument }
    })
}