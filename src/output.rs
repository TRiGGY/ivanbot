use serenity::model::channel::Message;
use serenity::http::CacheHttp;

pub fn output(ctx: &impl CacheHttp, msg: &mut Message, message: String) {
    let split = message.as_str();
    let items = split_for_discord(split);
    for item in items.iter() {
        match msg.reply(ctx.http(), item) {
            Ok(_) => println!("sent message: \"{}\"", item),
            Err(error) => println!("Error sending discord message: \"{}\" because of \"{}\"", item, error.to_string())
        };
    }
}

// pub fn output_send_single(cache_and_http: impl CacheHttp, msg: &mut Message, message: String) -> Result<Message, AdminCommandError> {
//     if message.len() < 1500 {
//         msg.reply(cache_and_http, message).map_err(|err| {
//             AdminCommandError {
//                 input: format!("could not sent message because of {}",err),
//                 kind: BotErrorKind::CouldNotReply,
//             }
//         })
//     } else {
//         Err(AdminCommandError {
//             input: "message was too big".to_string(),
//             kind: BotErrorKind::InvalidVoteAmount,
//         })
//     }
// }


fn split_for_discord(msg: &str) -> Vec<&str> {
    if msg.len() > 1900 {
        let msg1: &str = &msg[..1000];
        let msg2: &str = &msg[1000..msg.len()];
        let mut vec = vec!(msg1);
        vec.append(&mut split_for_discord(msg2));
        vec
    } else {
        vec!(msg)
    }
}
