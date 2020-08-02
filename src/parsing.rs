use crate::model::{BotCommandError, BotErrorKind};

pub fn pa<'a>(arguments: &Vec<&'a str>, index: usize) -> Result<&'a str, BotCommandError> {
    (arguments.get(index)).ok_or_else(|| {
        BotCommandError { input: "".to_string(), kind: BotErrorKind::MissingArgument }
    }).map(|value| { *value })
}


pub fn parse_discord_id(value: &str) -> Result<u64, BotCommandError> {
    value.parse::<u64>().map_err(|_error| {
        BotCommandError { input: value.to_string(), kind: BotErrorKind::InvalidArgument }
    })
}

