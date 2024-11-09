mod structs;
mod options;

use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::process::exit;
use std::{fs, io};
use structs::Config;

const CONFIG_FILE: &str = "caddy_manager.toml";
const ENABLED_DIR: &str = "sites-enabled/";
const DISABLED_DIR: &str = "sites-disabled/";

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
            "add" | "a" => add(&config),
            "remove" | "r" => remove(&config),
            "show" | "s" => show(),
            "enable" | "e" => enable(&config),
            "disable" | "d" => disable(&config),
            "configure" | "config" | "c" => configure(&mut config),
            "quit" | "q" => break,
            _ => println!(r#"Invalid input, "exit" to exit"#),
        }
    }

    println!("Remember to apply the changes, goodbye!");

    config.dump();
}

fn add(config: &Config) {
    if fs::metadata(ENABLED_DIR).is_err() {
        fs::create_dir(ENABLED_DIR).expect(&format!("Unable to create {ENABLED_DIR}"));
    }

    let host = get_host(config);

    println!("Which domain should be added?");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let domain = if host.is_empty() {
        String::from(input.trim())
    } else {
        format!("{}.{host}", input.trim())
    };

    let path = format!("{ENABLED_DIR}{domain}");
    if fs::metadata(&path).is_ok() {
        println!("Domain already exists, aborting");
        return;
    }

    let target = get_target(config);

    let port;

    loop {
        println!("Enter the target port");
        input.clear();
        io::stdin().read_line(&mut input).unwrap();
        port = match input.trim().parse::<u16>() {
            Ok(port) => port,
            Err(_) => {
                println!("Invalid input, port must be a valid u16");
                continue;
            }
        };
        break;
    }

    let site_config = format!(
        "\
        {domain} {{\n\
        \treverse_proxy {target}:{port}\n\
        \n\
        \timport ../robots\n\
        }}\n"
    );

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&path)
        .expect(&format!("Unable to open/create file {path}"));

    let mut writer = BufWriter::new(file);

    writeln!(writer, "{site_config}").expect(&format!("Unable to write to {path}"));

    println!("Successfully added {domain}");
}

fn get_target(config: &Config) -> String {
    let mut target = String::new();

    if config.targets.is_empty() {
        return target;
    }

    println!("Do you want to use an existing target [Y/n]");

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    if input.trim().to_lowercase() == "n" {
        println!("Enter the target");
        input.clear();
        io::stdin().read_line(&mut input).unwrap();
        return String::from(input.trim());
    }
    let len = config.targets.len();

    print_indexed_vec(&config.targets);

    loop {
        input.clear();
        print!("Select [idx] or [A]bort adding: ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim().to_lowercase().as_str() {
            "abort" | "a" => return target,
            _ => {}
        }

        match input.trim().parse::<usize>() {
            Ok(idx) => {
                if idx >= len {
                    println!("Input index out of range");
                    continue;
                }
                target = config.targets[idx].clone();
                return target;
            }
            Err(_) => {
                println!("Invalid input");
                continue;
            }
        }
    }
}

fn get_host(config: &Config) -> String {
    let mut host = String::new();

    if config.hosts.is_empty() {
        return host;
    }

    println!("Do you want to use an existing host [Y/n]");

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    if input.trim().to_lowercase() == "n" {
        return host;
    }
    let len = config.hosts.len();

    print_indexed_vec(&config.hosts);

    loop {
        input.clear();
        print!("Select [idx] or [A]bort adding: ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim().to_lowercase().as_str() {
            "abort" | "a" => return host,
            _ => {}
        }

        match input.trim().parse::<usize>() {
            Ok(idx) => {
                if idx >= len {
                    println!("Input index out of range");
                    continue;
                }
                host = config.hosts[idx].clone();
                return host;
            }
            Err(_) => {
                println!("Invalid input");
                continue;
            }
        }
    }
}

fn remove(config: &Config) {
    if fs::metadata(ENABLED_DIR).is_err() {
        println!("There are no enabled sites, aborting");
        return;
    }

    let host = get_host(config);

    println!("Which domain should be removed?");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let domain = if host.is_empty() {
        String::from(input.trim())
    } else {
        format!("{}.{host}", input.trim())
    };

    if domain.is_empty() {
        println!("No domain entered, aborting");
        return;
    }

    let path = format!("{ENABLED_DIR}{domain}");

    if fs::metadata(&path).is_err() {
        println!(r#""{path}" does not exist, aborting"#);
        return;
    }

    println!(r#"Warning, this will delete "{path}", are you sure? [y/N]"#);
    input.clear();
    io::stdin().read_line(&mut input).unwrap();

    if input.trim().to_lowercase() != "y" {
        println!("Aborting, no file will be deleted");
        return;
    }

    fs::remove_file(&path).expect(&format!("Unable to delete {path}"));
    println!("Successfully removed {path}");
}

fn show() {
    let enabled_sites = mk_vec_of_dir(ENABLED_DIR);
    let disabled_sites = mk_vec_of_dir(DISABLED_DIR);

    let mut max_len = 0;

    for site in &enabled_sites {
        if site.len() > max_len { max_len = site.len(); }
    }
    for site in &disabled_sites {
        if site.len() > max_len { max_len = site.len(); }
    }

    print_with_padding("Site", "Status", max_len);
    println!("{}", "-".repeat(max_len + 3 + 6)); // " | " + "Status"

    for site in &enabled_sites {
        print_with_padding(site, "enabled", max_len);
    }
    if !enabled_sites.is_empty() && !disabled_sites.is_empty() { println!("{}", "-".repeat(max_len + 3 + 6)); }
    for site in &disabled_sites {
        print_with_padding(site, "disabled", max_len);
    }
}

fn mk_vec_of_dir(dir: &str) -> Vec<String> {
    let mut vec = vec![];
    if fs::metadata(dir).is_err() {
        return vec;
    }
    fs::read_dir(dir).expect(&format!("Unable to read {dir}"))
                     .for_each(|entry| {
                         vec.push(entry.unwrap().file_name().into_string().unwrap());
                     });
    vec.sort();
    vec
}

fn print_with_padding(site: &str, status: &str, max_len: usize) {
    let remaining = max_len - site.len();
    println!("{site}{} ┃ {status}", " ".repeat(remaining));
}

fn enable(config: &Config) {
    let action = "enabled";
    toggle(config, DISABLED_DIR, ENABLED_DIR, action);
    println!("Successfully {action} site");
}

fn disable(config: &Config) {
    let action = "disabled";
    toggle(config, ENABLED_DIR, DISABLED_DIR, action);
    println!("Successfully {action} site");
}

fn toggle(config: &Config, source_dir: &str, target_dir: &str, action: &str) {
    let host = get_host(config);
    println!("Which domain should be {action}?");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let domain = if host.is_empty() {
        String::from(input.trim())
    } else {
        format!("{}.{host}", input.trim())
    };

    if domain.is_empty() {
        println!("No domain entered, aborting");
        return;
    }

    let source_path = format!("{source_dir}{domain}");
    let target_path = format!("{target_dir}{domain}");

    if fs::metadata(&source_path).is_err() {
        println!(r#""{source_path}" does not exist, aborting"#);
        return;
    }

    if fs::metadata(&target_path).is_ok() {
        println!(r#"Warning, this will delete "{source_path}", are you sure? [y/N]"#);
        input.clear();
        io::stdin().read_line(&mut input).unwrap();

        if input.trim().to_lowercase() != "y" {
            println!("Aborting, no file will be changed");
            return;
        }
    }

    if fs::metadata(&target_dir).is_err() {
        fs::create_dir(&target_dir).expect(&format!("Unable to create {}", target_dir));
    }

    fs::rename(&source_path, &target_path).unwrap();

    if fs::read_dir(&source_dir).unwrap().next().is_none() {
        println!(r#""{source_dir}" is empty, removing"#);
        fs::remove_dir(&source_dir).unwrap()
    }
}

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
    let len = vec.len();
    if len == 0 {
        println!("Config is empty, nothing to remove");
        return;
    }

    print_indexed_vec(&vec);

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

fn print_indexed_vec(vec: &Vec<String>) {
    let mut len = vec.len();

    let mut width = 0;

    while len > 0 {
        len /= 10;
        width += 1;
    }

    len = vec.len();

    for i in 0..len {
        println!("[{i:0>width$}]: {}", vec.get(i).unwrap());
    }
}
