use std::fmt::{Display, Formatter};
use core::fmt;
use serenity::model::channel::{Message, ReactionType, MessageReaction, GuildChannel};
use serenity::client::Context;
use crate::discord::{CustomFramework, ConcurrentFramework};
use std::ops::{Add};
use serenity::model::id::{MessageId, ChannelId, GuildId};
use serenity::model::channel::ReactionType::Unicode;
use crate::model::{IvanError, BotErrorKind, reply, assign_skins, get_users_from_channel};
use crate::pavlov::{PavlovCommands, GameMode, Skin};
use rand::seq::{IteratorRandom, SliceRandom};
use serenity::http::{CacheHttp, Http};
use serenity::{CacheAndHttp};
use std::sync::{Arc, Mutex, RwLockReadGuard};
use std::thread::sleep;
use std::time::{Duration, Instant};
use crate::config::{GunMode};
use serenity::utils::with_cache;
use serenity::prelude::RwLock;

const KNIFE: char = '🍴';
const SALT: char = '🧂';
const HUNDRED: char = '💯';
const MONEY: char = '💸';
const CHAMPAGNE: char = '🍾';
const SMIRK: char = '😏';
const HAND: char = '👌';
const BARF: char = '🤮';
const TURD: char = '💩';
const GOBLIN: char = '👺';

const ALL_VOTE_OPTIONS: [char; 10] = [HUNDRED, KNIFE, SALT, MONEY, CHAMPAGNE, SMIRK, HAND, BARF, TURD, GOBLIN];

pub const MAX_VOTE_MAPS: u32 = 4;


pub struct Vote {
    maps: Vec<Choice>,
    message_id: MessageId,
    channel_id: ChannelId,
    countdown: u64,
    users: Vec<u64>,
    pub teams: Option<(Vec<u64>, Vec<u64>)>,
}

struct Choice {
    id: String,
    map: String,
    alias: String,
    gamemode: GameMode,
}

impl Display for Vote {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let timer_message = format!("\nThe vote will end in: \"{}\" seconds", self.countdown);

        let message: String = self.maps.iter().map(|element| {
            format!("Vote: {} for map: \"{}\" gamemode: {}", element.id, element.alias, element.gamemode)
        }).fold(timer_message, |a, b| {
            let with_enter = a.add("\n");
            with_enter.add(b.as_str())
        });
        write!(f, "{}", message)?;

        match &self.teams {
            Some((team1, team2)) => {
                write!(f, "\nRed team: {}", format_users(team1))?;
                write!(f, "\nBlue team: {}", format_users(team2))
            }
            None => {
                write!(f, "\nGet ready to vote: {}", format_users(&self.users))
            }
        }
    }
}

fn format_users(vec: &Vec<u64>) -> String {
    let value = vec.iter().fold("".to_string(), |a, b| {
        a.add(format!("<@{}> ", b).as_str())
    });
    value.trim_end().to_string()
}

impl Display for Choice {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} gamemode: {}", self.alias, self.gamemode)
    }
}

pub fn handle_vote_start(framework: &mut CustomFramework, msg: &mut Message, ctx: &mut Context, concurrent_framework: &ConcurrentFramework, game_mode: Option<GameMode>, users: Vec<u64>, teams: Option<(Vec<u64>, Vec<u64>)>) -> Result<(), IvanError> {
    match &mut framework.vote {
        Some(_) => {
            Err(IvanError {
                input: "".to_string(),
                kind: BotErrorKind::VoteInProgress,
            })
        }
        None => {
            let maps = framework.config.get_maps_random(game_mode)?;
            if maps.is_empty() {
                reply(msg, ctx.http(), "Can not start a vote without a map pool, add maps with -map add [url/map] [gamemode] [alias]".to_string())?;
                return Ok(());
            }
            let emojis = get_random_emojis(framework.config.get_vote_amount() as usize)?;
            let choices: Vec<Choice> = maps.iter().zip(emojis).map(|(poolmap, emoji)| {
                Choice {
                    id: emoji.to_string(),
                    map: poolmap.map.clone(),
                    alias: poolmap.alias.clone(),
                    gamemode: handle_gunmode(poolmap.gamemode.clone(), framework.config.get_gun_mode()),
                }
            }).collect();

            let mut vote = Vote { maps: choices, message_id: MessageId(0), channel_id: msg.channel_id, countdown: 30, users, teams };
            let mut reply = reply(msg, ctx.http(), vote.to_string())?;
            for x in &vote.maps {
                _react(&mut reply, ctx, &Unicode(x.id.clone())).map_err(|_| {
                    IvanError { kind: BotErrorKind::CouldNotReply, input: "tried to react".to_string() }
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
    if gamemode == GameMode::GUN && gun_mode == GunMode::WW2 {
        GameMode::WW2GUN
    } else if gamemode == GameMode::GUN && gun_mode == GunMode::OitcRandom {
        vec![GameMode::GUN, GameMode::WW2GUN, GameMode::OITC].choose(&mut rand::thread_rng()).unwrap().clone()
    } else if gamemode == GameMode::GUN && gun_mode == GunMode::Random {
        vec![GameMode::GUN, GameMode::WW2GUN].choose(&mut rand::thread_rng()).unwrap().clone()
    } else {
        gamemode
    }
}

fn vote_thread(framework_arc: Arc<Mutex<CustomFramework>>, cache: Arc<CacheAndHttp>) {
    std::thread::spawn(move || {
        wait_until_ready(framework_arc.clone(), &cache).unwrap_or_else(|error| {
            println!("waiting for vote failed because: {}", error);
            return;
        });
        let (mut msg, skin_shuffle) = match framework_arc.lock() {
            Ok(mut value) => {
                if let Some(vote) = &value.vote {
                    let mut message = cache.http.get_message(vote.channel_id.clone().0, vote.message_id.clone().0).unwrap();
                    handle_vote_finish(&mut value, &mut message, cache.clone()).unwrap_or_else(|err| {
                        println!("{}", err)
                    });
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
                    reply(&mut msg, &cache.http, assign_skins(&mut value, Skin::get_random).unwrap_or_else(|error| { error.to_string() })).unwrap_or_else(|value| {
                        println!("{}", value);
                        msg
                    });
                }
                Err(err) => {
                    println!("mutex error {}", err.to_string());
                    return ();
                }
            }
        }
    });
}

fn wait_until_ready(framework_clone: Arc<Mutex<CustomFramework>>, cache_clone: &Arc<CacheAndHttp>) -> Result<(), IvanError> {
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

fn update_vote(framework_clone: Arc<Mutex<CustomFramework>>, cache_clone: &Arc<CacheAndHttp>, time_left: Duration) -> Result<(), IvanError> {
    match framework_clone.lock() {
        Ok(mut framework) => {
            match &mut framework.vote {
                Some(vote) => {
                    vote.countdown = time_left.as_secs();
                    let mut message = cache_clone.http.get_message(vote.channel_id.clone().0, vote.message_id.clone().0).map_err(|err| {
                        IvanError { input: format!("{}", err), kind: BotErrorKind::MessageRetrieveError }
                    })?;
                    message.edit(cache_clone, |m| { m.content(format!("{}", vote)) }).map_err(|err| {
                        IvanError { input: format!("{}", err), kind: BotErrorKind::MessageEditError }
                    })?;
                    Ok(())
                }
                None => { Err(IvanError { kind: BotErrorKind::VoteNotInProgress, input: "".to_string() }) }
            }
        }
        Err(_) => {
            println!("could not update vote, poison error");
            Ok(())
        }
    }
}


fn _react(msg: &mut Message, cache_http: &mut impl CacheHttp, reaction_type: &ReactionType) -> serenity::Result<()> {
    cache_http.http().create_reaction(msg.channel_id.clone().0, msg.id.clone().0, reaction_type)
}


pub fn handle_vote_finish(framework: &mut CustomFramework, msg: &mut Message, mut ctx: Arc<CacheAndHttp>) -> Result<(), IvanError> {
    match &mut framework.vote {
        Some(vote) => {
            let mut message = ctx.http.get_message(vote.channel_id.clone().0, vote.message_id.clone().0).unwrap();
            let winner = determine_winner(vote, &mut message);
            reply(msg, &ctx.http, format!("The winner is: {}", winner))?;
            let response = framework.connection.execute_command(PavlovCommands::SwitchMap { map: winner.map.clone(), gamemode: winner.gamemode.clone() });
            reply(msg, &ctx.http, response)?;
            let teams = vote.teams.clone();

            framework.vote = None;
            match teams {
                None => {println!("there are no teams so no moving")}
                Some((team1, team2)) => {
                    return match framework.config.get_team_channels() {
                        Some((channel1, channel2)) => {
                            let channel_1 = get_channel(&mut ctx, channel1)?;
                            let channel_2 = get_channel(&mut ctx, channel2)?;
                            move_to_channel(&mut ctx, channel_1.clone(), channel_2.clone(), team2)?;
                            move_to_channel(&mut ctx, channel_2.clone(), channel_1.clone(), team1)?;

                            Ok(())
                        }
                        None => {
                            reply(msg, &ctx.http, "No team channels configured so users will not be moved".to_string());
                            Ok(())
                        }
                    };
                }
            }
            return Ok(());
        }
        None => Err(IvanError { input: "".to_string(), kind: BotErrorKind::VoteNotInProgress })
    }
}

fn move_to_channel(ctx: &mut Arc<CacheAndHttp>, channel_from: Arc<RwLock<GuildChannel>>, channel_to: Arc<RwLock<GuildChannel>>, team: Vec<u64>) -> Result<(), IvanError> {
    for member in channel_from.read().members(&ctx.cache).map_err(|err| {
        IvanError { input: err.to_string(), kind: BotErrorKind::DiscordError }
    })? {
        if team.contains(&member.user_id().0) {
            let user = member.user.read();
            println!("moving user {} to team channel {}", user.id.0, channel_to.read().id.0);
            channel_from.read().guild_id.move_member(&ctx.http, user.id, channel_to.read().id).unwrap_or_else(|err| {
                println!("{}", err);
            });
        }
    };
    Ok(())
}

fn get_channel(ctx: &mut Arc<CacheAndHttp>, channel: u64) -> Result<Arc<RwLock<GuildChannel>>, IvanError> {
    let dc_channel = ctx.http.get_channel(channel).map_err(|err| {
        IvanError { input: err.to_string(), kind: BotErrorKind::DiscordError }
    })?;
    let guild_channel = dc_channel.guild().ok_or_else(|| {
        IvanError { input: "channel was not a guild channel".to_string(), kind: BotErrorKind::DiscordError }
    })?;
    Ok(guild_channel)
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

fn get_random_emojis(amount: usize) -> Result<Vec<&'static char>, IvanError> {
    if ALL_VOTE_OPTIONS.len() < amount {
        return Err(IvanError { kind: BotErrorKind::InvalidVoteAmount, input: format!("{} was more than the amount of emojis I have hardcoded :)", amount) });
    }
    let mut chosen = ALL_VOTE_OPTIONS.iter().choose_multiple(&mut rand::thread_rng(), amount);
    chosen.shuffle(&mut rand::thread_rng());
    Ok(chosen.iter().map(|value| { (value).clone() }).collect())
}




