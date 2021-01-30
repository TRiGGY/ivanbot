use crate::config::IvanConfig;
use crate::parsing::{pa, parse_discord_id};
use crate::model::{AdminCommandError, BotErrorKind};
extern crate derive_more;
use derive_more::{Display};

#[derive(Display,PartialOrd, PartialEq)]
pub enum PermissionLevel {
    Admin,
    Mod,
    User,
    None
}


pub fn handle_mod(arguments: &Vec<&str>, config: &mut IvanConfig) -> Result<String, AdminCommandError> {
    let mode = pa(arguments, 1)?;
    match mode {
        "add" => add_mod(parse_discord_id(pa(arguments, 2)?)?, config),
        "remove" => remove_mod(parse_discord_id(pa(arguments, 2)?)?, config),
        _ => Err(AdminCommandError {
            input: mode.to_string(),
            kind: BotErrorKind::InvalidArgument,
        })
    }
}
pub fn handle_admin(arguments: &Vec<&str>, config: &mut IvanConfig) -> Result<String, AdminCommandError> {
    let mode = pa(arguments, 1)?;
    match mode {
        "add" => add_admin(parse_discord_id(pa(arguments, 2,)?)?, config),
        "remove" => remove_admin(parse_discord_id(pa(arguments, 2)?)?, config),
        _ => Err(AdminCommandError {
            input: mode.to_string(),
            kind: BotErrorKind::InvalidArgument,
        })
    }
}

fn add_mod(id: u64, config: &mut IvanConfig) -> Result<String, AdminCommandError> {
    config.add_mod(id).map_err(|err| {
        AdminCommandError {
            input: err.to_string(),
            kind: BotErrorKind::ErrorConfig,
        }
    }).map(|_| {
        format!("Added moderator with id \"{}\" to the moderator list", id)
    })
}



fn add_admin(id: u64, config: &mut IvanConfig) -> Result<String, AdminCommandError> {
    config.add_admin(id).map_err(|err| {
        AdminCommandError {
            input: err.to_string(),
            kind: BotErrorKind::ErrorConfig,
        }
    }).map(|_| {
        format!("Added admin with id \"{}\" to the admin list", id)
    })
}


fn remove_mod(id: u64, config: &mut IvanConfig) -> Result<String, AdminCommandError> {
    config.remove_mod(id).map_err(|err| {
        AdminCommandError {
            input: err.to_string(),
            kind: BotErrorKind::ErrorConfig,
        }
    }).map(|_| {
        format!("Removed moderator with id \"{}\" from the moderator list", id)
    })
}

fn remove_admin(id: u64, config: &mut IvanConfig) -> Result<String, AdminCommandError> {
    config.remove_admin(id).map_err(|err| {
        AdminCommandError {
            input: err.to_string(),
            kind: BotErrorKind::ErrorConfig,
        }
    }).map(|_| {
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


