use crate::discord::BotCommandError;
use crate::voting::BotErrorKind;
use crate::config::IvanConfig;
use crate::parsing::{pa, parse_discord_id};
use crate::model::{BotCommandError, BotErrorKind};

pub fn handle_mod(arguments: &Vec<&str>, config: &mut IvanConfig) -> Result<String, BotCommandError> {
    let mode = pa(arguments, 1)?;
    match mode {
        "add" => add_mod(parse_discord_id(pa(arguments, 2)?)?, config),
        "remove" => remove_mod(parse_discord_id(pa(arguments, 2)?)?, config),
        _ => Err(BotCommandError {
            input: mode.to_string(),
            kind: BotErrorKind::InvalidArgument,
        })
    }
}
pub fn handle_admin(arguments: &Vec<&str>, config: &mut IvanConfig) -> Result<String, BotCommandError> {
    let mode = pa(arguments, 1)?;
    match mode {
        "add" => add_admin(parse_discord_id(pa(arguments, 2)?)?, config),
        "remove" => remove_admin(parse_discord_id(pa(arguments, 2)?)?, config),
        _ => Err(BotCommandError {
            input: mode.to_string(),
            kind: BotErrorKind::InvalidArgument,
        })
    }
}

fn add_mod(id: u64, config: &mut IvanConfig) -> Result<String, BotCommandError> {
    config.add_mod(id).map_err(|err| {
        BotCommandError {
            input: err.to_string(),
            kind: BotErrorKind::ErrorConfig,
        }
    }).map(|_| {
        format!("Added moderator with id \"{}\" to the moderator list", id)
    })
}



fn add_admin(id: u64, config: &mut IvanConfig) -> Result<String, BotCommandError> {
    config.add_admin(id).map_err(|err| {
        BotCommandError {
            input: err.to_string(),
            kind: BotErrorKind::ErrorConfig,
        }
    }).map(|_| {
        format!("Added admin with id \"{}\" to the admin list", id)
    })
}


fn remove_mod(id: u64, config: &mut IvanConfig) -> Result<String, BotCommandError> {
    config.remove_mod(id).map_err(|err| {
        BotCommandError {
            input: err.to_string(),
            kind: BotErrorKind::ErrorConfig,
        }
    }).map(|_| {
        format!("Removed moderator with id \"{}\" from the moderator list", id)
    })
}

fn remove_admin(id: u64, config: &mut IvanConfig) -> Result<String, BotCommandError> {
    config.remove_admin(id).map_err(|err| {
        BotCommandError {
            input: err.to_string(),
            kind: BotErrorKind::ErrorConfig,
        }
    }).map(|_| {
        format!("Removed admin with id \"{}\" from the admin list", id)
    })
}