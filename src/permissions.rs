use crate::config::IvanConfig;
use crate::parsing::{ parse_discord_id};
use crate::model::{IvanError, BotErrorKind};
extern crate derive_more;
use derive_more::{Display};
use crate::pavlov::pa;
use crate::help::{HELP_STEAM_ID, HELP_ADMIN_MODE, HELP_MOD_MODE, HELP_SKIN_MODE};

#[derive(Display,PartialOrd, PartialEq)]
pub enum PermissionLevel {
    Admin,
    Mod,
    User,
    None
}


pub fn handle_mod(arguments: &Vec<&str>, config: &mut IvanConfig) -> Result<String, IvanError> {
    let mode = pa(arguments, 1,HELP_SKIN_MODE)?;
    match mode {
        "add" => add_mod(parse_discord_id(pa(arguments, 2,HELP_MOD_MODE)?)?, config),
        "remove" => remove_mod(parse_discord_id(pa(arguments, 2,HELP_MOD_MODE)?)?, config),
        _ => Err(IvanError {
            input: mode.to_string(),
            kind: BotErrorKind::InvalidArgument,
        })
    }
}
pub fn handle_admin(arguments: &Vec<&str>, config: &mut IvanConfig) -> Result<String, IvanError> {
    let mode = pa(arguments, 1,HELP_ADMIN_MODE)?;
    match mode {
        "add" => add_admin(parse_discord_id(pa(arguments, 2,HELP_STEAM_ID)?)?, config),
        "remove" => remove_admin(parse_discord_id(pa(arguments, 2,HELP_STEAM_ID)?)?, config),
        _ => Err(IvanError {
            input: mode.to_string(),
            kind: BotErrorKind::InvalidArgument,
        })
    }
}

fn add_mod(id: u64, config: &mut IvanConfig) -> Result<String, IvanError> {
    config.add_mod(id).map(|_| {
        format!("Added moderator with id \"{}\" to the moderator list", id)
    })
}

fn add_admin(id: u64, config: &mut IvanConfig) -> Result<String, IvanError> {
    config.add_admin(id).map(|_| {
        format!("Added admin with id \"{}\" to the admin list", id)
    })
}

fn remove_mod(id: u64, config: &mut IvanConfig) -> Result<String, IvanError> {
    config.remove_mod(id).map(|_| {
        format!("Removed moderator with id \"{}\" from the moderator list", id)
    })
}

fn remove_admin(id: u64, config: &mut IvanConfig) -> Result<String, IvanError> {
    config.remove_admin(id).map(|_| {
        format!("Removed admin with id \"{}\" from the admin list", id)
    })
}

pub fn mod_allowed(argument: String) -> bool {
    user_allowed(argument.clone()) || match argument.as_str() {
        "switchmap" |
        "rotatemap" |
        "alias" |
        "map" |
        "maplist" |
        "switchteam" |
        "giveitem" |
        "givecash" |
        "setCash" |
        "resetsnd" |
        "setplayerskin" |
        "setlimitedammotype"
        => true,
        _ => false
    }
}

pub fn user_allowed(argument: String) -> bool {
    match argument.as_str() {
        "inspectplayer" | "serverinfo" | "refreshlist" | "bothelp"| "maplist" => true,
        _ => false
    }
}


