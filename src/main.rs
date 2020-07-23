mod management;
mod config;

use crate::discord::run_discord;

mod discord;
mod connect;
mod pavlov;
mod credentials;

fn main() {
    run_discord();
}

