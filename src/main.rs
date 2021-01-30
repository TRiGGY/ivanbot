mod output;
mod parsing;
mod permissions;
mod voting;
mod model;
mod config;

use crate::discord::run_discord;
mod discord;
mod connect;
mod pavlov;
mod credentials;
mod help;

fn main() {
    run_discord();
}

