mod structs;
mod options;

use std::{fs, io};
use std::io::Write;
use std::process::exit;
use structs::Config;

const CONFIG_FILE: &str = "caddy_manager.toml";

fn main() {
    if fs::metadata("Caddyfile").is_err() {
        println!("Caddyfile not found, check that you are in the correct directory");
        exit(1);
    }

    let mut config = Config::init();

    config.hosts.sort();
    // Technically duplicates can only happen, if the user modified the .toml file manually
    config.hosts.dedup();
    config.targets.sort();
    config.targets.dedup();

    let options = options::get_main_options();

    println!("Welcome to Caddy Manager!");

    loop {
        println!("{options}");
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim().to_lowercase().as_str() {
            "add" | "a" => add(),
            "remove" | "r" => remove(),
            "show" | "s" => show(),
            "enable" | "e" => enable(),
            "disable" | "d" => disable(),
            "configure" | "config" | "c" => configure(&mut config),
            "quit" | "q" => break,
            _ => println!(r#"Invalid input, "exit" to exit"#),
        }
    }

    println!("Goodbye!");

    config.dump();
}

fn add() {}

fn remove() {}

fn show() {}

fn enable() {}

fn disable() {}

fn configure(config: &mut Config) {
    config.changed = true;
    let type_options = options::get_conf_options();
    let operation_options = options::get_conf_sub_options();

    loop {
        println!("{type_options}");
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let to_conf = match input.trim().to_lowercase().as_str() {
            "hosts" | "host" | "h" => &mut config.hosts,
            "targets" | "target" | "t" => &mut config.targets,
            "quit" | "q" => return,
            _ => {
                println!(r#"Invalid input, "quit" to quit"#);
                continue;
            }
        };

        loop {
            println!("{operation_options}");

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();

            match input.trim().to_lowercase().as_str() {
                "add" | "a" => con_add(to_conf),
                "remove" | "r" => con_rem(to_conf),
                "quit" | "q" => return,
                _ => {
                    println!(r#"Invalid input, "quit" to quit"#);
                    continue;
                }
            };
            break;
        }

        break;
    }
}

fn con_add(vec: &mut Vec<String>) {
    println!("Enter the string to save");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    input = String::from(input.trim());

    if vec.contains(&input) {
        println!("Already exists");
        return;
    }

    vec.push(String::from(input));
    vec.sort();
}

fn con_rem(vec: &mut Vec<String>) {
    let mut len = vec.len();
    if len == 0 {
        println!("Config is empty, nothing to remove");
        return;
    }

    let mut width = 0;

    while len > 0 {
        len /= 10;
        width += 1;
    }

    len = vec.len();

    for i in 0..len {
        println!("[{i:0>width$}]: {}", vec.get(i).unwrap());
    }

    let idx;
    loop {
        print!("Remove [idx] or [Q]uit: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim().to_lowercase().as_str() {
            "quit" | "q" => return,
            _ => {}
        }

        match input.trim().parse::<usize>() {
            Ok(u) => {
                if u >= len {
                    println!("Input out of range");
                    continue;
                }
                idx = u;
                break;
            }
            Err(_) => {
                println!("Invalid input");
                continue;
            }
        }
    }

    vec.remove(idx);
}
