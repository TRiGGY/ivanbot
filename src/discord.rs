use serenity::client::{Client, EventHandler};
use serenity::model::channel::{Message};
use serenity::prelude::{Context};
use serenity::framework::Framework;
use crate::connect::{maintain_connection};
use crate::pavlov::{PavlovCommands};
use std::process::exit;
use std::env::{var};
use crate::credentials::{get_login};
use threadpool::ThreadPool;


use std::sync::mpsc::{Sender, Receiver};
use crate::config::{get_config, IvanConfig};
use crate::model::{handle_command};
use crate::voting::Vote;
use std::sync::{Mutex, Arc};
use serenity::CacheAndHttp;
use crate::permissions::PermissionLevel;


struct Handler;

impl EventHandler for Handler {}

pub struct ConcurrentFramework {
    pub data: Arc<Mutex<CustomFramework>>,
    pub cache: Arc<CacheAndHttp>,
}

impl Framework for ConcurrentFramework {
    fn dispatch(&mut self, ctx: Context, msg: Message, _: &ThreadPool) {
        let mutex = self.data.lock();
        return match mutex {
            Ok(mut guard) => {
                event_handler(&mut guard, ctx, msg, self)
            }
            Err(err) => {
                panic!(err.to_string())
            }
        };
    }
}

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
    let arc = Arc::new(Mutex::from(CustomFramework {
        sender,
        receiver,
        config: config.unwrap(),
        vote: None,
    }));

    let concurrent_framework = ConcurrentFramework {
        data: arc,
        cache: client.cache_and_http.clone(),
    };

    client.with_framework(concurrent_framework);
    if let Err(why) = client.start() {
        println!("Err with client: {:?}", why);
    }
}


fn event_handler(framework: &mut CustomFramework, ctx: Context, msg: Message, concurrent_framework: &ConcurrentFramework) {
    let permission_level = authenticate(&msg, &framework.config);
    if let PermissionLevel::None = permission_level {
        return;
    }
    if !(PermissionLevel::Admin == permission_level
        &&
        msg.guild_id == None) &&
        !right_channel(msg.channel_id.0, &framework.config) {
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
    handle_command(framework, ctx, msg, &values, concurrent_framework, permission_level);
}


fn authenticate(msg: &Message, config: &IvanConfig) -> PermissionLevel {
    let uid = msg.author.id.0;
    if config.is_admin(uid) {
        return PermissionLevel::Admin;
    }
    if config.is_mod(uid) {
        return PermissionLevel::Mod;
    }
    if IvanConfig::allow_users() {
        return PermissionLevel::User;
    }
    return PermissionLevel::None;
}

fn right_channel(channel_id: u64, config: &IvanConfig) -> bool {
    match config.get_channel_lock() {
        Some(lock) => lock == channel_id,
        None => true
    }
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





