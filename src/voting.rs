use std::fmt::{Display, Formatter};
use core::fmt;
use serenity::model::channel::{Message, ReactionType, MessageReaction};
use serenity::client::Context;
use crate::discord::{CustomFramework, ConcurrentFramework};
use std::ops::{Add};
use serenity::model::id::{MessageId, ChannelId};
use serenity::model::channel::ReactionType::Unicode;
use crate::model::{AdminCommandError, BotErrorKind, reply, assign_skins};
use crate::pavlov::{PavlovError, PavlovCommands, GameMode, Skin};
use rand::seq::{IteratorRandom, SliceRandom};
use serenity::http::{CacheHttp, Http};
use serenity::{CacheAndHttp};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration,  Instant};
use std::error::Error;
use crate::config::{ GunMode};
use crate::pavlov::GameMode::WW2GUN;
use crate::config::GunMode::Random;

const KNIFE: char = '🍴';
const SALT: char = '🧂';
const HUNDRED: char = '💯';
const MONEY: char = '💸';
const CHAMPAGNE: char = '🍾';
const SMIRK: char = '😏';
const HAND: char = '👌';

pub const MAX_VOTE_MAPS: u32 = 4;

pub struct Vote {
    maps: Vec<Choice>,
    message_id: MessageId,
    channel_id: ChannelId,
    countdown: u64,
}

struct Choice { id: String, map: String, alias: String, gamemode: GameMode }

impl Display for Vote {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let timer_message = format!("\nThe vote will end in: \"{}\" seconds", self.countdown);

        let message: String = self.maps.iter().map(|element| {
            format!("Vote: {} for map: \"{}\" gamemode: {}", element.id, element.alias, element.gamemode)
        }).fold(timer_message, |a, b| {
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
                    gamemode: handle_gunmode(poolmap.gamemode.clone(), framework.config.get_gun_mode()),
                }
            }).collect();
            let mut vote = Vote { maps: choices, message_id: MessageId(0), channel_id: msg.channel_id, countdown: 30 };
            let mut reply = reply(msg, ctx.http(), vote.to_string())?;
            for x in &vote.maps {
                _react(&mut reply, ctx, &Unicode(x.id.clone())).map_err(|_| {
                    AdminCommandError { kind: BotErrorKind::CouldNotReply, input: "tried to react".to_string() }
                })?;
            }
            vote.message_id = reply.id;
            framework.vote = Some(vote);

            let framework_clone = concurrent_framework.data.clone();
            let cache_clone = concurrent_framework.cache.clone();
            vote_thread(framework_clone, cache_clone);
            Ok(())
        }
    }
}

fn handle_gunmode(gamemode: GameMode, gun_mode: GunMode) -> GameMode {
    if gamemode == GameMode::GUN && gun_mode == GunMode::WW2{
        WW2GUN
    } else if gamemode == GameMode::GUN && gun_mode == GunMode::Random {
        vec![GameMode::GUN, GameMode::WW2GUN].choose(&mut rand::thread_rng()).unwrap().clone()
    } else {
        gamemode.clone()
    }
}

fn vote_thread(framework_arc: Arc<Mutex<CustomFramework>>, cache: Arc<CacheAndHttp>) {
    std::thread::spawn(move || {
        wait_until_ready(framework_arc.clone(), &cache).unwrap_or_else(|error| {
            println!("waiting for vote failded because: {}", error);
            return;
        });
        let (mut msg, skin_shuffle) = match framework_arc.lock() {
            Ok(mut value) => {
                if let Some(vote) = &value.vote {
                    let mut message = cache.http.get_message(vote.channel_id.0, vote.message_id.0).unwrap();
                    handle_vote_finish(&mut value, &mut message, &cache.http).unwrap();
                    (message, value.config.get_skin_shuffle())
                } else {
                    return;
                }
            }
            Err(err) => {
                println!("{}", err.to_string());
                return ();
            }
        };

        if skin_shuffle {
            sleep(Duration::from_secs(90));
            match framework_arc.lock() {
                Ok(mut value) => {
                    reply(&mut msg, &cache.http, assign_skins(&mut value, Skin::get_random).unwrap_or_else(|error| { error.to_string() }));
                }
                Err(err) => {
                    println!("{}", err.to_string());
                    return ();
                }
            }
        }
    });
}

fn wait_until_ready(framework_clone: Arc<Mutex<CustomFramework>>, cache_clone: &Arc<CacheAndHttp>) -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let vote_duration = Duration::from_secs(30);
    let sleep_duration = Duration::from_secs(3);
    let future = start.add(vote_duration);

    loop {
        let now = Instant::now();
        if future < now {
            break;
        } else {
            let time_left = vote_duration - now.duration_since(start);
            if time_left > sleep_duration {
                update_vote(framework_clone.clone(), &cache_clone, time_left)?;
                sleep(sleep_duration);
            } else {
                update_vote(framework_clone.clone(), &cache_clone, time_left)?;
                sleep(time_left);
            }
        }
    }
    Ok(())
}

fn update_vote(framework_clone: Arc<Mutex<CustomFramework>>, cache_clone: &Arc<CacheAndHttp>, time_left: Duration) -> Result<(), Box<dyn Error>> {
    match framework_clone.lock() {
        Ok(mut framework) => {
            match &mut framework.vote {
                Some(vote) => {
                    vote.countdown = time_left.as_secs();
                    let mut message = cache_clone.http.get_message(vote.channel_id.0, vote.message_id.0)?;
                    message.edit(cache_clone, |m| { m.content(format!("{}", vote)) })?;
                    Ok(())
                }
                None => { Err(Box::new(AdminCommandError { kind: BotErrorKind::VoteNotInProgress, input: "".to_string() })) }
            }
        }
        Err(_) => {
            println!("could not update vote, poison error");
            Ok(())
        }
    }
}


fn _react(msg: &mut Message, cache_http: &mut impl CacheHttp, reaction_type: &ReactionType) -> serenity::Result<()> {
    cache_http.http().create_reaction(msg.channel_id.0, msg.id.0, reaction_type)
}


pub fn handle_vote_finish(framework: &mut CustomFramework, msg: &mut Message, http: &Arc<Http>) -> Result<(), AdminCommandError> {
    match &framework.vote {
        Some(vote) => {
            let mut message = http.get_message(vote.channel_id.0, vote.message_id.0).unwrap();
            let winner = determine_winner(vote, &mut message);
            reply(msg, http, format!("The winner is: {}", winner))?;
            let response = framework.connection.execute_command(PavlovCommands::SwitchMap { map: winner.map.clone(), gamemode: winner.gamemode.clone() });
            reply(msg, http, response)?;
            framework.vote = None;

            Ok(())
        }
        None => Err(AdminCommandError { input: "".to_string(), kind: BotErrorKind::VoteNotInProgress })
    }
}

fn determine_winner<'a>(vote: &'a Vote, msg: &mut Message) -> &'a Choice {
    let emoji: Vec<(&MessageReaction, &String)> = msg.reactions.iter().filter_map(|reaction| {
        match &reaction.reaction_type {
            Unicode(value) => { Some((reaction, value)) }
//ReactionType::Custom { animated, id, name } => { Some(id) }
            _ => { None }
        }
    }).filter(|(_, emoji)| {
        vote.maps.iter().any(|value| { value.id == **emoji })
    }).collect();
    let max = emoji.iter().max_by(|(first, _), (second, _)| { first.count.cmp(&second.count) });
    match max {
        None => panic!("no emoji found"),
        Some((reaction, _)) => {
            match emoji.iter().filter(|(react, _)| {
                react.count >= reaction.count
            }).choose(&mut rand::thread_rng()) {
                Some((_, value)) => vote.maps.iter().find(|el| { el.id == **value }).unwrap(),
                None => panic!("no winner found")
            }
        }
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
    if amount > 7 {
        return Err(AdminCommandError { kind: BotErrorKind::InvalidVoteAmount, input: format!("{} was more than the amount of emojis I have hardcoded :)", amount) });
    }
    let emojis = vec!(HUNDRED.to_string(), KNIFE.to_string(), SALT.to_string(), MONEY.to_string(), CHAMPAGNE.to_string(), SMIRK.to_string(), HAND.to_string());
    let mut chosen = emojis.iter().choose_multiple(&mut rand::thread_rng(), amount);
    chosen.shuffle(&mut rand::thread_rng());
    Ok(chosen.iter().map(|value| { (*value).clone() }).collect())
}




