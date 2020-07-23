use serenity::Client;
use std::process::exit;
use crate::connect::{get_connection, get_error};
use crate::credentials::get_login;
use serenity::client::EventHandler;
use crate::discord::run_discord;

mod discord;
mod connect;
mod pavlov;
mod credentials;



fn main() {
    run_discord();
}

