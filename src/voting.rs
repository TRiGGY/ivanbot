use std::fmt::{Display, Formatter};
use core::fmt;
use serenity::model::channel::Message;
use serenity::client::Context;
use crate::discord::{ CustomFramework};
use std::ops::Add;
use serenity::model::id::{MessageId, ChannelId};
use serenity::model::channel::ReactionType::Unicode;
use crate::model::{AdminCommandError, BotErrorKind, reply};
use crate::pavlov::PavlovError;
use rand::seq::IteratorRandom;

const KNIFE: char = '🍴';
const SALT: char = '🧂';
const HUNDRED: char = '💯';
const MAX_VOTE_MAPS: usize = 3;

pub struct Vote {
    maps: Vec<Choice>,
    message_id: MessageId,
    channel_id: ChannelId,
}

struct Choice { id: String, map: String, alias: String, gamemode: String }

impl Display for Vote {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let message: String = self.maps.iter().map(|element| {
            format!("Vote: {} for map: \"{}\" gamemode: {}", element.id, element.alias, element.gamemode)
        }).fold("".to_string(), |a, b| {
            let with_enter = a.add("\n");
            with_enter.add(b.as_str())
        });
        write!(f, "{}", message)
    }
}

impl Display for Choice {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "The winning map is {} gamemode: {}", self.alias, self.gamemode)
    }
}

pub fn handle_vote_start(framework: &mut CustomFramework, msg: &mut Message, ctx: &mut Context) -> Result<(), AdminCommandError> {
    match &mut framework.vote {
        Some(vote) => {
            Err(AdminCommandError {
                input: "".to_string(),
                kind: BotErrorKind::VoteInProgress,
            })
        }
        None => {
            let maps = framework.config.get_maps_random(MAX_VOTE_MAPS);
            let emojis = get_random_emojis(MAX_VOTE_MAPS);
            let choices: Vec<Choice> = maps.iter().zip(emojis).map(|(poolmap, emoji)| {
                Choice {
                    id: emoji.to_string(),
                    map: poolmap.map.clone(),
                    alias: poolmap.alias.clone(),
                    gamemode: poolmap.gamemode.clone(),
                }
            }).collect();
            let mut vote = Vote { maps: choices, message_id: MessageId(0), channel_id: msg.channel_id };
            let reply = reply(msg, ctx, vote.to_string()).unwrap();
            vote.message_id = reply.id;
            framework.vote = Some(vote);
            Ok(())
        }
    }
}

pub fn handle_vote_finish(framework: &mut CustomFramework, msg: &mut Message, ctx: &mut Context) -> Result<(), AdminCommandError> {
    match &framework.vote {
        Some(vote) => {
            let message = &mut ctx.http.get_message(vote.channel_id.0, vote.message_id.0).unwrap();
            let winner = determine_winner(vote, message);
            reply(msg, ctx, format!("The winner is: {}", winner));
            Ok(())
        }
        None => Err(AdminCommandError { input: "".to_string(), kind: BotErrorKind::VoteNotInProgress })
    }
}

fn determine_winner<'a>(vote: &'a Vote, msg: &mut Message) -> &'a Choice {
    let emoji = msg.reactions.iter().filter_map(|reaction| {
        match &reaction.reaction_type {
            Unicode(value) => { Some((reaction, value)) }
            //ReactionType::Custom { animated, id, name } => { Some(id) }
            _ => { None }
        }
    }).filter(|(reaction, emoji)| {
        vote.maps.iter().any(|value| { value.id == **emoji })
    }).max_by(|(first, _), (second, _)| { first.count.cmp(&second.count) });
    match emoji {
        None => panic!("no emoji found"),
        Some((_, value)) => vote.maps.iter().find(|el| { el.id == *value }).unwrap()
    }
}

pub fn convert_to_not_found() -> fn(PavlovError) -> AdminCommandError {
    return |i: PavlovError| -> AdminCommandError {
        AdminCommandError {
            input: i.input,
            kind: BotErrorKind::MissingArgument,
        }
    };
}

fn get_random_emojis(amount: usize) -> Vec<String> {
    let emojis = vec!(HUNDRED.to_string(), KNIFE.to_string(), SALT.to_string());
    let chosen = emojis.iter().choose_multiple(&mut rand::thread_rng(), amount);
    chosen.iter().map(|value| { (*value).clone() }).collect()
}




