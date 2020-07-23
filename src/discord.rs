use serenity::client::{Client, EventHandler};
use serenity::model::channel::Message;
use serenity::prelude::{Context};
use serenity::framework::standard::{
    StandardFramework,
    CommandResult,
    macros::{
        command,
        group,
    },
};

use serenity::model::*;
use std::collections::HashMap;
use std::env;
use serenity::framework::Framework;
use crate::connect::{PavlovConnection, pavlov_connect, get_error, get_connection};
use crate::pavlov::{PavlovCommands, PavlovError, ErrorKind};
use std::process::exit;
use std::env::{var, VarError};
use crate::credentials::{LoginData, get_login};
use threadpool::ThreadPool;
use serenity::http::CacheHttp;

use text_io::read;
use std::thread::sleep;
use std::time::Duration;

struct Handler;
impl EventHandler for Handler {}

pub fn run_discord() {
    let token = get_discord_token();
    let mut client = Client::new(&token, Handler {}).unwrap();
    let login = get_login();
    let mut connection = get_connection(&login);
    match connection {
        Err(err)=> {
            let (error,fatal) = get_error(&err);
            println!("{}",error);
            return exit(1)
        }
        Ok(conn) => {
            client.with_framework(
                CustomFramework {
                    pavlov_connection: conn,
                    is_working: true,
                    login_data: login,
                }
            );
            if let Err(why) = client.start() {
                println!("Err with client: {:?}", why);
            }
        }
    }
}

 struct CustomFramework {
    pavlov_connection: PavlovConnection,
    is_working: bool,
    login_data: LoginData,
}


impl Framework for CustomFramework {
    fn dispatch(&mut self, ctx: Context, msg: Message, threadpool: &ThreadPool) {
        if msg.author.bot {
            return
        }
        if !msg.content.starts_with("-") {
            return
        }
        if !self.is_working {
            let connection = get_connection(&self.login_data);
            match connection {
                Ok(conn) => {
                    self.pavlov_connection = conn;
                    self.is_working = true
                },
                Err(why) => {
                    let (error,_) = get_error(&why);
                    output(ctx,msg,error);
                    return
                }
            }
        }
        let cloned = msg.content.clone();
        let stripped = cloned.trim_start_matches("-");
        let values: Vec<&str> = stripped.split_whitespace().collect();
        let command_string = PavlovCommands::parse_from_arguments(&values);

        match command_string {
            Ok(command) => {
                println!("{}",&command.to_string());
                let result = self.pavlov_connection.sent_command(command.to_string());
                match result {
                    Err(err) => {
                        let (error_message, should_restart) = get_error(&err);
                        output(ctx, msg, error_message);
                        if should_restart {
                            self.is_working = false;
                        };
                    }
                    Ok(value) => {
                        output(ctx, msg, value);
                    }
                }
            }
            Err(err) => {
                let (error_message, should_restart) = get_error(&err);
                output(ctx, msg, error_message);
                if should_restart {
                    self.is_working = false;
                };
            }
        };
    }
}






fn output(ctx: Context, msg: Message, message: String) {
    println!("{}",&message);
    msg.reply( ctx,message);
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