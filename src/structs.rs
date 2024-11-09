use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter, Read, Write};
use serde::{Deserialize, Serialize};
use crate::CONFIG_FILE;

#[derive(Debug)]
pub struct Config {
    pub hosts: Vec<String>,
    pub targets: Vec<String>,
    pub changed: bool,
}

#[derive(Deserialize, Serialize)]
struct RawConfig {
    hosts: Vec<String>,
    targets: Vec<String>,
}

impl Config {
    pub fn init() -> Self {
        let file = match OpenOptions::new()
            .read(true)
            .open(CONFIG_FILE) {
            Ok(f) => f,
            Err(_) => {
                return Config {
                    hosts: vec![],
                    targets: vec![],
                    changed: true,
                }
            }
        };

        let mut reader = BufReader::new(file);

        let mut content = String::new();
        reader.read_to_string(&mut content)
              .expect(&format!("Unable to read {CONFIG_FILE}"));

        let config = toml::from_str::<RawConfig>(&content)
            .expect(&format!("Unable to parse {CONFIG_FILE}"));

        Self {
            hosts: config.hosts,
            targets: config.targets,
            changed: false,
        }
    }

    pub fn dump(self) {
        let raw = RawConfig {
            hosts: self.hosts,
            targets: self.targets,
        };

        let toml = toml::to_string_pretty(&raw).unwrap();
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(CONFIG_FILE)
            .expect(&format!("Unable to open/crete/truncate {CONFIG_FILE}"));

        let mut writer = BufWriter::new(file);

        writeln!(writer, "{toml}")
            .expect(&format!("Unable to write to {CONFIG_FILE}"));
    }
}
