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

pub struct Config {
    pub db_path: String,
    pub prompt: Prompt,
    pub trust_keyring: bool,
}

pub fn get_pass(trust_keyring: bool) -> anyhow::Result<String> {
    if !trust_keyring {
        delete_pass_from_keyring()?;
    }
    let user = std::env::var_os("USER").unwrap_or_default();
    let entry = Entry::new("sshield", user.to_str().unwrap_or_default())?;
    Ok(match entry.get_password() {
        Ok(pass) => pass,
        Err(_) => {
            let pass = crate::gui::get_db_pass();
            if trust_keyring {
                entry.set_password(&pass)?;
            }
            pass
        }
    })
}

pub fn delete_pass_from_keyring() -> anyhow::Result<()> {
    let user = std::env::var_os("USER").unwrap_or_default();
    let entry = Entry::new("sshield", user.to_str().unwrap_or_default())?;
    entry.delete_password()?;
    Ok(())
}

fn get_config_path() -> String {
    match env::var_os("XDG_CONFIG_HOME") {
        Some(path) => {
            let parentconfdir = path.to_str().unwrap_or_default().to_string();
            parentconfdir + "/sshield"
        }
        None => {
            let home_dir = env::var_os("HOME").unwrap();
            let parentconfdir = home_dir.into_string().unwrap();
            parentconfdir + ".config/sshield"
        }
    }
}

fn create_config_folder(config_path: &str) -> anyhow::Result<()> {
    if !Path::new(&config_path).exists() {
        eprintln!("Creating the config folder and default config");
        fs::create_dir(config_path)?;
        let mut config = Table::new();
        config.insert(
            "database".to_string(),
            (config_path.to_string() + "/keys.db3").into(),
        );
        config.insert("keyring".to_string(), false.into());
        config.insert("prompt".to_string(), 60.into());
        let rawconf = config.to_string();
        let conffilename = config_path.to_string() + "/sshield.toml";
        let mut file = fs::File::create(conffilename)?;
        file.write_all(rawconf.as_bytes())?;
    }
    Ok(())
}

pub fn get_all_vars() -> anyhow::Result<Config> {
    let config_path = get_config_path();
    create_config_folder(&config_path)?;
    let file_path = config_path + "/sshield.toml";
    let rawconf = fs::read_to_string(file_path)?;
    let conf: Table = rawconf.parse()?;
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
    // Don't trust keyring by default
    let trust_keyring = conf.get("keyring").unwrap().as_bool().unwrap_or(false);
    Ok(Config {
        db_path,
        prompt: auth_settings,
        trust_keyring,
    })
}
