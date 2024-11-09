mod config;

use std::io;
use config::Config;

const CONFIG_FILE: &str = "caddy_manager.toml";

fn main() {
    let mut config = Config::init();
    let options = format!("{} exit", get_options());

    println!("Welcome to Caddy Manager");

    loop {
        println!("{options}");
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "exit" => break,
            _ => println!(r#"Invalid input, "exit" to exit"#),
        }
    }

    if config.changed { config.dump(); }
}

fn get_options() -> String {
    vec![
        "Add",
        "Remove",
        "Show",
        "Enable",
        "Disable",
        "Configure",
    ].into_iter()
        .map(|s| {
            let mut chars = s.chars();
            let first = chars.next().unwrap();
            let remaining = chars.as_str();
            format!("[{first}]{remaining}")
        })
        .fold(String::from("Options:"), |acc, s| format!("{acc} {s}"))
}
