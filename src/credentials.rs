use std::env;
use std::process::exit;
use std::env::var;

pub struct LoginData {
    pub ip: String,
    pub password: String,
}

pub fn get_login() -> LoginData {
    let args: Vec<String> = env::args().collect();
    let command_name = args.get(0).unwrap();
    let ip = var("IVAN_CONNECT_IP");
    if ip.is_err() {
        println!("{}", format!("usage: {} ip:port password", command_name));
        exit(1)
    }
    let password = var("IVAN_PASSWORD");
    if password.is_err() {
        println!("{}", format!("usage: {} ip:port password", command_name));
        exit(1)
    }
    return LoginData {
        ip: ip.unwrap().clone(),
        password: password.unwrap().clone(),
    };
}

