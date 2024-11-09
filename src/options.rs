pub fn get_main_options() -> String {
    let options = vec![
        "Add",
        "Remove",
        "Show",
        "Enable",
        "Disable",
        "Configure",
        "Quit",
    ];
    map_options("Options", options)
}

pub fn get_conf_options() -> String {
    let options = vec![
        "Hosts",
        "Targets",
        "Quit",
    ];
    map_options("Configure", options)
}

pub fn get_conf_sub_options() -> String {
    let options = vec![
        "Add",
        "Remove",
        "Quit",
    ];
    map_options("Configure", options)
}

fn map_options(prefix: &str, options: Vec<&str>) -> String {
    options.into_iter()
           .map(|s| {
               let mut chars = s.chars();
               let first = chars.next().unwrap();
               let remaining = chars.as_str();
               format!("[{first}]{remaining}")
           })
           .fold(format!("{prefix}:"), |acc, s| format!("{acc} {s}"))
}

