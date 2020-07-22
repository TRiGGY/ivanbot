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
use serenity::Framework;
use serenity::model::*;
use tokio_core::reactor::Handle;
use std::collections::HashMap;


#[group]
#[commands(ping)]
struct General;

use std::env;
use serenity::framework::Framework;
use crate::connect::PavlovConnection;
use crate::get_connection;


impl EventHandler for Handler {}

fn main() {
    // Login with a bot token from the environment
    let mut client = Client::new(&token, Handler).unwrap();
    let mut connection = get_connection();

    client.with_framework({
        CustomFramework { pavlov_connection: &mut connection }
    });

    // start listening for events by starting a single shard
    if let Err(why) = client.start() {
        println!("An error occurred while running the client: {:?}", why);
    }
}

struct CustomFramework<'a> {
    pavlov_connection: &'a mut PavlovConnection
}

impl Framework for CustomFramework {
    fn dispatch(&mut self, ctx: Context, msg: Message, tokio_handle: &Handle) {

        self.pavlov_connection.sent_command()
    }
}

struct Handler;

impl EventHandler for Handler {}
