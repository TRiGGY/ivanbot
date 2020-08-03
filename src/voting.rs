use std::fmt::{Display, Formatter};
use core::fmt;
use serenity::model::channel::{Message, ReactionType};
use serenity::client::Context;
use crate::discord::{CustomFramework, ConcurrentFramework};
use std::ops::{Add};
use serenity::model::id::{MessageId, ChannelId};
use serenity::model::channel::ReactionType::Unicode;
use crate::model::{AdminCommandError, BotErrorKind, reply};
use crate::pavlov::{PavlovError, PavlovCommands, GameMode};
use rand::seq::{IteratorRandom, SliceRandom};
use serenity::http::{CacheHttp, Http};
use serenity::{Error, CacheAndHttp};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use rand::thread_rng;

const KNIFE: char = '🍴';
const SALT: char = '🧂';
const HUNDRED: char = '💯';
const MONEY: char = '💸';
const CHAMPAGNE: char = '🍾';
const SMIRK: char = '😏';


pub const MAX_VOTE_MAPS: u32 = 4;

pub struct Vote {
    maps: Vec<Choice>,
    message_id: MessageId,
    channel_id: ChannelId,
}

struct Choice { id: String, map: String, alias: String, gamemode: GameMode }

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
        write!(f, "{} gamemode: {}", self.alias, self.gamemode)
    }
}

pub fn handle_vote_start(framework: &mut CustomFramework, msg: &mut Message, ctx: &mut Context, concurrent_framework: &ConcurrentFramework, choices: usize) -> Result<(), AdminCommandError> {
    match &mut framework.vote {
        Some(_) => {
            Err(AdminCommandError {
                input: "".to_string(),
                kind: BotErrorKind::VoteInProgress,
            })
        }
        None => {
            let maps = framework.config.get_maps_random(choices)?;
            if maps.is_empty() {
                reply(msg, ctx.http(), "Can not start a vote without a map pool, add maps with -map add [url/map] [gamemode] [alias]".to_string())?;
                return Ok(());
            }
            let emojis = get_random_emojis(choices)?;
            let choices: Vec<Choice> = maps.iter().zip(emojis).map(|(poolmap, emoji)| {
                Choice {
                    id: emoji.to_string(),
                    map: poolmap.map.clone(),
                    alias: poolmap.alias.clone(),
                    gamemode: poolmap.gamemode.clone(),
                }
            }).collect();
            let mut vote = Vote { maps: choices, message_id: MessageId(0), channel_id: msg.channel_id };
            let mut reply = reply(msg, ctx.http(), vote.to_string()).unwrap();
            for x in &vote.maps {
                _react(&mut reply, ctx, &Unicode(x.id.clone())).map_err(|_| {
                    AdminCommandError { kind: BotErrorKind::CouldNotReply, input: "tried to react".to_string() }
                })?;
            }
            vote.message_id = reply.id;
            framework.vote = Some(vote);

            let framework_clone = concurrent_framework.data.clone();
            let cache_clone = concurrent_framework.cache.clone();
            create_thread(framework_clone, cache_clone);
            Ok(())
        }
    }
}

fn create_thread(framework_clone: Arc<Mutex<CustomFramework>>, cache_clone: Arc<CacheAndHttp>) {
    std::thread::spawn(move || {
        sleep(Duration::from_secs(30));
        let guard = framework_clone.lock();
        match guard {
            Ok(mut value) => {
                match &value.vote {
                    Some(vote) => {
                        let message = &mut cache_clone.http.get_message(vote.channel_id.0, vote.message_id.0).unwrap();
                        handle_vote_finish(&mut value, message, &cache_clone.http).unwrap();
                        ()
                    }
                    None => {
                        ()
                    }
                }
            }
            Err(err) => {
                println!("{}", err.to_string());
                ()
            }
        }
    });
}


fn _react(msg: &mut Message, cache_http: &mut impl CacheHttp, reaction_type: &ReactionType) -> Result<(), Error> {
    cache_http.http().create_reaction(msg.channel_id.0, msg.id.0, reaction_type)
}


pub fn handle_vote_finish(framework: &mut CustomFramework, msg: &mut Message, http: &Arc<Http>) -> Result<(), AdminCommandError> {
    match &framework.vote {
        Some(vote) => {
            let mut message = http.get_message(vote.channel_id.0, vote.message_id.0).unwrap();
            let winner = determine_winner(vote, &mut message);
            reply(msg, http, format!("The winner is: {}", winner))?;
            framework.sender.send(PavlovCommands::SwitchMap { map: winner.map.clone(), gamemode: winner.gamemode.clone() }).unwrap();
            let pavlov = framework.receiver.recv().unwrap();
            reply(msg, http, pavlov)?;
            framework.vote = None;
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
    }).filter(|(_, emoji)| {
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

fn get_random_emojis(amount: usize) -> Result<Vec<String>, AdminCommandError> {
    if amount > 6 {
        return Err(AdminCommandError { kind: BotErrorKind::InvalidVoteAmount, input: format!("{} was more than the amount of emojis I have hardcoded :)",amount)});
    }
    let emojis = vec!(HUNDRED.to_string(), KNIFE.to_string(), SALT.to_string(), MONEY.to_string(), CHAMPAGNE.to_string(), SMIRK.to_string());
    let mut chosen = emojis.iter().choose_multiple(&mut rand::thread_rng(), amount);
    chosen.shuffle(&mut rand::thread_rng());
    Ok(chosen.iter().map(|value| { (*value).clone() }).collect())
}




