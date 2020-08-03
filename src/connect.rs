use std::io::prelude::*;
use std::net::{TcpStream, SocketAddr};
use std::str::FromStr;
use std::time::Duration;
use std::io::BufReader;
use hex::encode;
use crate::pavlov::{PavlovError, ErrorKind, PavlovCommands};
use crate::pavlov::ErrorKind::ConnectionError;
use std::{io};
use crate::credentials::LoginData;
use std::thread;
use std::sync::mpsc::{Receiver, Sender, channel};
use crate::model::{BotErrorKind, AdminCommandError};

const AUTHENTICATED: &str = "Authenticated=1";


pub fn maintain_connection(login_data: LoginData) -> (Sender<PavlovCommands>, Receiver<String>) {
    let input: (Sender<PavlovCommands>, Receiver<PavlovCommands>) = channel();
    let output: (Sender<String>, Receiver<String>) = channel();
    let (tx, rx) = input;
    let (tx_string, rx_string) = output;
    thread::spawn(move || {
        connection_loop(rx, login_data, tx_string);
    });
    (tx, rx_string)
}

fn connection_loop<'a, 'b>(rx: Receiver<PavlovCommands>, login_data: LoginData, mut tx_string: Sender<String>) {
    let mut connection = get_connection(&login_data);
    loop {
        let input = rx.recv().unwrap();
        connection = match connection {
            Err(err) => {
                let (error, _) = get_error_pavlov(&err);
                println!("{}", error);
                tx_string.send(error).unwrap();
                Err(err)
            }
            Ok(conn) => {
                Ok(execute_command(input, conn, &mut tx_string, &login_data, false))
            }
        }
    }
}

fn execute_command(input: PavlovCommands, mut connection: PavlovConnection, send: &mut Sender<String>, login_data: &LoginData, on_retry: bool) -> PavlovConnection {
    let response = connection.sent_command(input.to_string());
    return match response {
        Err(err) => {
            let (error_message, should_restart) = get_error_pavlov(&err);
            if should_restart && !on_retry {
                let new_connection = get_connection(login_data);
                if let Ok(conn) = new_connection {
                    return execute_command(input, conn, send, login_data, true);
                }
            }
            send.send(error_message).unwrap();
            connection
        }
        Ok(value) => {
            send.send(value).unwrap();
            connection
        }
    };
}


fn get_connection(login_data: &LoginData) -> Result<PavlovConnection, PavlovError> {
    return pavlov_connect(&login_data.ip, &login_data.password);
}


fn pavlov_connect<'a, >(address: &String, pass: &String) -> Result<PavlovConnection, PavlovError> {
    let addr = SocketAddr::from_str(&address.as_str()).map_err(|_err| {
        PavlovError { input: address.clone(), kind: ErrorKind::InvalidConnectionAddress }
    })?;
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(3)).map_err(|_err| {
        PavlovError { input: pass.clone(), kind: ConnectionError }
    })?;
    let mut buf_reader = BufReader::new(stream.try_clone().map_err(|_err| {
        PavlovError { input: "Error reading".to_string(), kind: ConnectionError }
    })?);
    let password = sent_password(pass, &mut stream).map_err(|_err| {
        PavlovError { input: "Error sending password".to_string(), kind: ErrorKind::ConnectionError }
    })?;
    let response1 = read_line(&mut buf_reader).map_err(|_err| {
        PavlovError { input: "unable to read first line".to_string(), kind: ErrorKind::ConnectionError }
    })?;

    return match response1.contains(AUTHENTICATED) {
        true => Ok(PavlovConnection {
            reader: buf_reader,
            writer: stream,
        }),
        false => Err(PavlovError {
            input: password,
            kind: ErrorKind::Authentication,
        })
    };
}


fn read_line(reader: &mut BufReader<TcpStream>) -> io::Result<String> {
    let mut read_line = String::from("");
    reader.read_line(&mut read_line)?;
    return Ok(read_line);
}

fn read_response(reader: &mut BufReader<TcpStream>) -> io::Result<String> {
    let mut buffer = String::from("");
    loop {
        let line = read_line(reader)?;
        if line.eq("\r\n") {
            continue;
        }
        buffer.push_str(line.as_str());
        if line.contains("\r\n") {
            return Ok(buffer);
        }
    }
}

fn sent_message(stream: &mut TcpStream, message: String) -> io::Result<usize> {
    let size = stream.write(message.as_bytes())?;
    write_newline(stream)?;
    stream.flush()?;
    return Ok(size + 1);
}

fn sent_password(pass: &String, stream: &mut TcpStream) -> io::Result<String> {
    let buf = hash_password(pass);
    println!("{}", &buf);
    stream.write(buf.as_bytes())?;
    //stream.write("  -".as_bytes()).unwrap();
    write_newline(stream)?;
    stream.flush()?;
    Ok(buf)
}


fn write_newline(mut stream: &TcpStream) -> io::Result<usize> {
    Ok(stream.write("\n".as_bytes())?)
}

fn hash_password(password: &String) -> String {
    return encode(md5::compute(password.as_bytes()).0);
}


pub struct PavlovConnection {
    reader: BufReader<TcpStream>,
    writer: TcpStream,
}

impl PavlovConnection {
    pub(crate) fn sent_command(&mut self, command: String) -> Result<String, PavlovError> {
        sent_message(&mut self.writer, command).map_err(|_err| {
            PavlovError { input: "Couldn't sent message".to_string(), kind: ErrorKind::ConnectionError }
        })?;
        read_response(&mut self.reader).map_err(|_err| {
            PavlovError { input: "Couldn't read message response".to_string(), kind: ErrorKind::ConnectionError }
        })
    }
}

pub fn get_error_pavlov(error: &PavlovError) -> (String, bool) {
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

pub fn get_error_botcommand(error: &AdminCommandError) -> String {
    match &error.kind {
        BotErrorKind::InvalidArgument => format!("Invalid argument \"{}\"", error.input),
        BotErrorKind::InvalidCommand => format!("Invalid command \"{}\"", error.input),
        BotErrorKind::MissingArgument => format!("Missing argument {}", error.input),
        BotErrorKind::ErrorConfig => format!("Missing argument {}", error.input),
        BotErrorKind::InvalidMapAlias => format!("Invalid map Alias \"{}\"",error.input),
        BotErrorKind::VoteInProgress => format!("There's already a vote in progress: {}",error.input),
        BotErrorKind::VoteNotInProgress => format!("There's no vote in progress: {}",error.input),
        BotErrorKind::CouldNotReply => format!("Could not reply to the channel"),
        BotErrorKind::InvalidGameMode => format!("Invalid game mode valid \"{}\" valid [DM,TDM,GUN,SND]",error.input)
    }
}