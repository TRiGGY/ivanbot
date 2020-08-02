use serenity::client::{Client, EventHandler};
use serenity::model::channel::{Message, ReactionType};
use serenity::prelude::{Context};
use serenity::framework::Framework;
use crate::connect::{get_error_pavlov, maintain_connection, get_error_botcommand};
use crate::pavlov::{PavlovCommands, parse_map, parse_game_mode, PavlovError, ErrorKind};
use std::process::exit;
use std::env::{var};
use crate::credentials::{get_login};
use threadpool::ThreadPool;


use std::sync::mpsc::{Sender, Receiver};
use crate::config::{get_config, IvanConfig};
use crate::discord::BotErrorKind::InvalidMapAlias;
use regex::Regex;
use std::fmt::{Display};
use serde::export::Formatter;
use core::fmt;
use std::ops::{Add, Deref};
use rand::seq::IteratorRandom;
use serenity::model::id::{MessageId, ChannelId, EmojiId};
use serenity::utils::parse_emoji;
use serenity::model::guild::Emoji;
use serenity::model::channel::ReactionType::Unicode;
use serenity::model::misc::EmojiIdentifier;
use serenity::model::gateway::ActivityEmoji;
use crate::model::{handle_command, BotCommandError, BotErrorKind};
use crate::parsing::pa;
use crate::model::BotErrorKind::InvalidMapAlias;

const MAX_VOTE_MAPS: usize = 3;


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
            vote: None,
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
    vote: Option<Vote>,
}

impl Framework for CustomFramework {
    fn dispatch(&mut self, mut ctx: Context, mut msg: Message, _: &ThreadPool) {
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
        handle_command(self,&mut ctx, &mut msg, &values);
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











fn handle_map_add(arguments: &Vec<&str>, framework: &&mut CustomFramework, msg: &mut Message, ctx: &mut Context) -> Result<Message, BotCommandError> {
    let map = convert_error_not_found(parse_map(pa(arguments, 2)?, &framework.config))?;
    let gamemode = pa(arguments, 3)?;
    let alias = pa(arguments, 4)?;
    framework.config.add_alias(alias.to_string(), map.clone());
    framework.config.add_map(map.clone(), gamemode.to_string(), alias.to_string());
    reply(msg, ctx, format!("Map added to pool with id: \"{}\",gamemode: \"{}\" alias: \"{}\"", map, gamemode, alias))
}


fn convert_error_not_found<T>(result: Result<T, PavlovError>) -> Result<T, BotCommandError> {
    result.map_err(convert_to_not_found())
}

fn handle_alias(arguments: &Vec<&str>, config: &mut IvanConfig) -> Result<String, BotCommandError> {
    let mode = pa(arguments, 1)?;
    match mode {
        "add" => {
            let map = parse_map(pa(arguments, 2)?, config).map_err(|err| {
                BotCommandError { kind: InvalidMapAlias, input: err.input }
            })?;
            let alias = check_alias(pa(arguments, 3)?)?;
            config.add_alias(alias.clone(), map.clone());
            Ok(format!("alias \"{}\" to map \"{}\" created", alias, map))
        }
        "remove" => remove_admin(parse_discord_id(pa(arguments, 2)?)?, config),
        _ => Err(BotCommandError {
            input: mode.to_string(),
            kind: BotErrorKind::InvalidArgument,
        })
    }
}




