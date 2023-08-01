use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use toml::Table;

fn get_config_path() -> String {
    match env::var_os("XDG_CONFIG_HOME") {
        Some(path) => {
            let parentconfdir = path.to_str().unwrap_or_default().to_string();
            let config_path = parentconfdir + "/sshield";
            config_path
        }
        None => {
            let home_dir = env::var_os("HOME").unwrap();
            let parentconfdir = home_dir.into_string().unwrap().to_string();
            let fallback_config_path = parentconfdir + ".config/sshield";
            fallback_config_path
        }
    }
}

fn create_config_folder(config_path: &str) {
    if Path::new(&config_path).exists() {
        return;
    } else {
        eprintln!("Creating the config folder and default config");
        fs::create_dir(&config_path).unwrap();
        let mut config = Table::new();
        config.insert(
            "database".to_string(),
            (config_path.to_string() + "/keys.db3").into(),
        );
        let rawconf = config.to_string();
        let conffilename = config_path.to_string() + "/sshield.toml";
        let mut file = fs::File::create(&conffilename).unwrap();
        file.write_all(rawconf.as_bytes()).unwrap();
    }
}

pub fn get_db_path() -> String {
    let config_path = get_config_path();
    create_config_folder(&config_path);
    let file_path = config_path + "/sshield.toml";
    let rawconf = fs::read_to_string(file_path).unwrap();
    let conf: Table = rawconf.parse().unwrap();
    let path = conf.get("database").unwrap().as_str().unwrap();
    // TODO: Figure out a way to remove these quotation marks elegantly
    path.replace("\"", "")
}
