use crate::model::{IvanError, BotErrorKind};


pub fn parse_discord_id(value: &str) -> Result<u64, IvanError> {
    value.parse::<u64>().map_err(|_error| {
        IvanError { input: format!("Invalid discord id \"{}\"",value), kind: BotErrorKind::InvalidArgument }
    })
}

