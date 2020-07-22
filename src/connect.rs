use std::io::prelude::*;
use std::net::{TcpStream, SocketAddr};
use std::str::FromStr;
use std::time::Duration;
use std::io::BufReader;
use hex::encode;
use crate::pavlov::{PavlovError, ErrorKind};
use crate::pavlov::ErrorKind::ConnectionError;
use std::io;

const AUTHENTICATED: &str = "Authenticated=1";


pub fn pavlov_connect<'a, >(address: &String, pass: &String) -> Result<PavlovConnection, PavlovError> {
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
