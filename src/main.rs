mod discord;
mod pavlov;
mod connect;

use text_io::read;
use crate::connect::{pavlov_connect, PavlovConnection};
use crate::pavlov::{PavlovCommands, ErrorKind, PavlovError};
use std::thread::sleep;
use std::time::Duration;
use std::env;

fn main() {

}


pub fn get_arguments() {
    let args: Vec<String> = env::args().collect();
    let command_name = args.get(0).unwrap();
    let ip = args.get(1);
    if ip.is_none() {
        println!("{}",format!("usage: {} ip:port password",command_name));
        return ()
    }
    let password = args.get(2);
    if password.is_none() {
        println!("{}",format!("usage: {} ip:port password",command_name));
        return ()
    }

}
pub fn get_connection() -> PavlovConnection {
    loop {
        let connection_result = pavlov_connect(ip.unwrap(), password.unwrap());
        match connection_result {
            Ok(connection) => {
                return connection;
            }
            Err(err) => {
                let (error_message, _should_restart) = get_error(&err);
                output(error_message);
                sleep(Duration::from_secs(5));
                break;
            }
        }
    }
}

fn command_accept_loop(connection: &mut PavlovConnection) {
    loop {
        let input: String = read!("{}\n");
        if input == "" { continue; }
        let iter = input.split_whitespace();
        let values: Vec<&str> = iter.collect();
        let command_string = PavlovCommands::parse_from_arguments(&values);
        match command_string {
            Ok(command) => {
                let result = connection.sent_command(command.to_string());
                match result {
                    Err(err)=> {
                        let (error_message, should_restart) = get_error(&err);
                        output(error_message);
                        if should_restart { break; };
                    }
                    Ok(value) => {
                        output(value);
                    }
                }
            }
            Err(err) => {
                let (error_message, should_restart) = get_error(&err);
                output(error_message);
                if should_restart { break; };
            }
        };
    }
}


fn get_error(error: &PavlovError) -> (String, bool) {
    match &error.kind {
        ErrorKind::InvalidArgument => (format!("Invalid argument \"{}\"", error.input), false),
        ErrorKind::InvalidCommand => (format!("Invalid command \"{}\"", error.input), false),
        ErrorKind::ConnectionError => (format!("Connection error: {}", error.input), true),
        ErrorKind::Authentication => (format!("Authentication error with password: {}", error.input), true),
        ErrorKind::InvalidConnectionAddress => (format!("Connection error connecting to \"{}\", make sure this is a valid address like google.com:9293", error.input), true),
        ErrorKind::MissingArgument => (format!("Missing argument {}", error.input), false),
        ErrorKind::InvalidMap => (format!("Invalid map name {}", error.input), false),
    }
}

fn output(message: String) {
    println!("{}", message.trim().trim_end_matches("\n"));
}