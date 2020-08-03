use serenity::client::Context;
use serenity::model::channel::Message;

pub fn output(ctx: &Context, msg: &mut Message, message: String) {
    println!("{}", &message);
    msg.reply(ctx, message).unwrap();
}
