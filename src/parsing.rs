use crate::model::{AdminCommandError, BotErrorKind};

pub fn pa<'a>(arguments: &Vec<&'a str>, index: usize) -> Result<&'a str, AdminCommandError> {
    (arguments.get(index)).ok_or_else(|| {
        AdminCommandError { input: "".to_string(), kind: BotErrorKind::MissingArgument }
    }).map(|value| { *value })
}


pub fn parse_discord_id(value: &str) -> Result<u64, AdminCommandError> {
    value.parse::<u64>().map_err(|_error| {
        AdminCommandError { input: value.to_string(), kind: BotErrorKind::InvalidArgument }
    })
}

