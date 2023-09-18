use keyring::Entry;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use toml::Table;

#[derive(PartialEq, Clone)]
pub enum Prompt {
    NoPrompt,
    EveryNSeconds(i64),
}

pub fn get_pass() -> String {
    let user = std::env::var_os("USER").unwrap();
    let entry = Entry::new("sshield", user.to_str().unwrap()).unwrap();
    match entry.get_password() {
        Ok(pass) => pass,
        Err(_) => {
            let pass = crate::gui::get_db_pass();
            entry.set_password(&pass).unwrap();
            pass
        }
    }
}

pub fn delete_pass_from_keyring() {
    let user = std::env::var_os("USER").unwrap();
    let entry = Entry::new("sshield", user.to_str().unwrap()).unwrap();
    entry.delete_password().unwrap();
}

fn get_config_path() -> String {
    match env::var_os("XDG_CONFIG_HOME") {
        Some(path) => {
            let parentconfdir = path.to_str().unwrap_or_default().to_string();
            parentconfdir + "/sshield"
        }
        None => {
            let home_dir = env::var_os("HOME").unwrap();
            let parentconfdir = home_dir.into_string().unwrap().to_string();
            parentconfdir + ".config/sshield"
        }
    }
}

fn create_config_folder(config_path: &str) {
    if !Path::new(&config_path).exists() {
        eprintln!("Creating the config folder and default config");
        fs::create_dir(config_path).unwrap();
        let mut config = Table::new();
        config.insert(
            "database".to_string(),
            (config_path.to_string() + "/keys.db3").into(),
        );
        let rawconf = config.to_string();
        let conffilename = config_path.to_string() + "/sshield.toml";
        let mut file = fs::File::create(conffilename).unwrap();
        file.write_all(rawconf.as_bytes()).unwrap();
    }
}

pub fn get_all_vars() -> (String, Prompt) {
    let config_path = get_config_path();
    create_config_folder(&config_path);
    let file_path = config_path + "/sshield.toml";
    let rawconf = fs::read_to_string(file_path).unwrap();
    let conf: Table = rawconf.parse().unwrap();
    let db_path = conf
        .get("database")
        .unwrap()
        .as_str()
        .unwrap()
        // TODO: Figure out a way to remove these quotation marks elegantly
        .replace('\"', "");
    let prompt_timeout = conf.get("prompt").unwrap().as_integer().unwrap_or(0);
    let auth_settings = {
        if prompt_timeout <= 0 {
            Prompt::NoPrompt
        } else {
            Prompt::EveryNSeconds(prompt_timeout)
        }
    };
    (db_path, auth_settings)
}
