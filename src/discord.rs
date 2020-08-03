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
use crate::model::{handle_command, AdminCommandError, BotErrorKind, reply};
use crate::parsing::{pa, parse_discord_id};
use crate::model::BotErrorKind::InvalidMapAlias;
use crate::voting::Vote;


struct Handler;

impl EventHandler for Handler {}
pub struct CustomFramework {
    pub sender: Sender<PavlovCommands>,
    pub receiver: Receiver<String>,
    pub config: IvanConfig,
    pub vote: Option<Vote>,
}


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
        handle_command(self, &mut ctx, &mut msg, &values);
    }
}

fn authenticate(msg: &Message, config: &IvanConfig) -> bool {
    let uid = msg.author.id.0;
    config.is_admin(uid)
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





