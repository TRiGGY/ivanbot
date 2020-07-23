use confy::*;
use serde::{Serialize, Deserialize};
use std::env::var;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct IvanConfig {
    version: u8,
    admins: Vec<u64>,
    aliases: HashMap<String, String>,
}

impl IvanConfig {
    pub fn is_admin(&self, id: u64) -> bool {
        let admin = var("ADMIN_ID");
        if admin.is_ok() {
            if id.to_string().eq(&admin.unwrap()) {
                return true;
            }
        }
        return self.admins.contains(&id);
    }
    pub fn add_admin(&mut self, id: u64) -> Result<(), ConfyError> {
        self.admins.retain(|item| { *item != id });
        self.admins.push(id);
        confy::store("ivan", self)
    }
    pub fn remove_admin(&mut self, id: u64) -> Result<(), ConfyError> {
        self.admins.retain(|item| { *item != id });
        confy::store("ivan", self)
    }
    pub fn add_alias(&mut self, alias: String, mapname: String) {
        self.aliases.insert(alias, mapname);
    }
    pub fn get_alias(&self, alias : &str) -> Option<&String> {
        self.aliases.get(alias)
    }
}

/// `MyConfig` implements `Default`
impl ::std::default::Default for IvanConfig {
    fn default() -> Self { Self { version: 1, admins: vec!(), aliases: HashMap::new() } }
}

pub fn get_config() -> Result<IvanConfig, confy::ConfyError> {
    let cfg: IvanConfig = confy::load_path("~/ivan.toml")?;
    Ok(cfg)
}