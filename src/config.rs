use confy::*;
use serde::{Serialize, Deserialize};
use std::env::var;

#[derive(Serialize, Deserialize)]
pub struct IvanConfig {
    version: u8,
    admins: Vec<u64>,
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
        self.admins.retain( |item|{*item != id});
        self.admins.push(id);
        confy::store("ivan",self)
    }
    pub fn remove_admin(&mut self, id: u64) -> Result<(), ConfyError> {
        self.admins.retain( |item|{*item != id});
        confy::store("ivan",self)
    }
}

/// `MyConfig` implements `Default`
impl ::std::default::Default for IvanConfig {
    fn default() -> Self { Self { version: 0, admins: vec!() } }
}

pub fn get_config() -> Result<IvanConfig, confy::ConfyError> {
    let cfg: IvanConfig = confy::load("ivan")?;
    Ok(cfg)
}