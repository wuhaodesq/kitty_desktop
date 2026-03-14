use std::env;
use std::path::PathBuf;

use kitty_desktop::{
    config_service::render_config_json, normalize_extra_args,
    session_service::render_sessions_json, ConfigService, DesktopConfig, DesktopShell,
    KittyAdapter, KittyLaunchConfig, SessionService, SessionTemplate,
};

fn print_usage() {
    eprintln!(
        "kitty_desktop core-adapter CLI\n\nUsage:\n  kitty_desktop version\n  kitty_desktop launch [--directory DIR] [--shell SHELL] [--session FILE] [--title TITLE] [--config FILE] [--dry-run] [-- ...extra args]\n  kitty_desktop config show [--config-path FILE]\n  kitty_desktop config set [--config-path FILE] [--directory DIR] [--shell SHELL] [--title TITLE] [--kitty-config FILE]\n  kitty_desktop session list [--session-path FILE]\n  kitty_desktop session save --name NAME [--session-path FILE] [--directory DIR] [--shell SHELL] [--title TITLE] [-- ...extra args]
  kitty_desktop shell run [--config-path FILE] [--session-path FILE]"
    );
}

fn default_config_path() -> PathBuf {
    PathBuf::from(".kitty_desktop/config.json")
}

fn default_session_path() -> PathBuf {
    PathBuf::from(".kitty_desktop/sessions.json")
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        std::process::exit(2);
    }

    let adapter = KittyAdapter::default();
    match args[1].as_str() {
        "version" => match adapter.get_version() {
            Ok(version) => println!("{version}"),
            Err(err) => {
                eprintln!("ERROR: {err}");
                std::process::exit(1);
            }
        },
        "launch" => {
            let mut config = KittyLaunchConfig::new();
            let mut dry_run = false;
            let mut i = 2usize;
            let mut raw_extra_args: Vec<String> = Vec::new();

            while i < args.len() {
                match args[i].as_str() {
                    "--directory" => {
                        i += 1;
                        ensure_value(&args, i, "--directory");
                        config.working_directory = Some(PathBuf::from(&args[i]));
                    }
                    "--shell" => {
                        i += 1;
                        ensure_value(&args, i, "--shell");
                        config.shell = Some(args[i].clone());
                    }
                    "--session" => {
                        i += 1;
                        ensure_value(&args, i, "--session");
                        config.session_file = Some(PathBuf::from(&args[i]));
                    }
                    "--title" => {
                        i += 1;
                        ensure_value(&args, i, "--title");
                        config.title = Some(args[i].clone());
                    }
                    "--config" => {
                        i += 1;
                        ensure_value(&args, i, "--config");
                        config.config_file = Some(PathBuf::from(&args[i]));
                    }
                    "--dry-run" => dry_run = true,
                    _ => {
                        raw_extra_args = args[i..].to_vec();
                        break;
                    }
                }
                i += 1;
            }

            config.extra_args = normalize_extra_args(&raw_extra_args);
            match adapter.launch(&config, dry_run) {
                Ok(command) => {
                    if dry_run {
                        println!("{}", command.join(" "));
                    } else {
                        println!("kitty launched");
                    }
                }
                Err(err) => {
                    eprintln!("ERROR: {err}");
                    std::process::exit(1);
                }
            }
        }
        "config" => handle_config_command(&args),
        "session" => handle_session_command(&args),
        "shell" => handle_shell_command(&args),
        _ => {
            print_usage();
            std::process::exit(2);
        }
    }
}

fn ensure_value(args: &[String], index: usize, flag: &str) {
    if index >= args.len() {
        eprintln!("missing value for {flag}");
        std::process::exit(2);
    }
}

fn handle_config_command(args: &[String]) {
    if args.len() < 3 {
        print_usage();
        std::process::exit(2);
    }

    match args[2].as_str() {
        "show" => {
            let mut path = default_config_path();
            let mut i = 3usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--config-path" => {
                        i += 1;
                        ensure_value(args, i, "--config-path");
                        path = PathBuf::from(&args[i]);
                    }
                    _ => {
                        eprintln!("unknown config flag: {}", args[i]);
                        std::process::exit(2);
                    }
                }
                i += 1;
            }

            let service = ConfigService::new(path);
            match service.load() {
                Ok(config) => {
                    println!("{}", render_config_json(&config));
                }
                Err(err) => {
                    eprintln!("ERROR: {err}");
                    std::process::exit(1);
                }
            }
        }
        "set" => {
            let mut path = default_config_path();
            let mut cfg = DesktopConfig::default();
            let mut i = 3usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--config-path" => {
                        i += 1;
                        ensure_value(args, i, "--config-path");
                        path = PathBuf::from(&args[i]);
                    }
                    "--directory" => {
                        i += 1;
                        ensure_value(args, i, "--directory");
                        cfg.default_directory = args[i].clone();
                    }
                    "--shell" => {
                        i += 1;
                        ensure_value(args, i, "--shell");
                        cfg.default_shell = Some(args[i].clone());
                    }
                    "--title" => {
                        i += 1;
                        ensure_value(args, i, "--title");
                        cfg.default_title = args[i].clone();
                    }
                    "--kitty-config" => {
                        i += 1;
                        ensure_value(args, i, "--kitty-config");
                        cfg.kitty_config_file = Some(args[i].clone());
                    }
                    _ => {
                        eprintln!("unknown config flag: {}", args[i]);
                        std::process::exit(2);
                    }
                }
                i += 1;
            }

            let service = ConfigService::new(path);
            if let Err(err) = service.save(&cfg) {
                eprintln!("ERROR: {err}");
                std::process::exit(1);
            }
            println!("config saved");
        }
        _ => {
            print_usage();
            std::process::exit(2);
        }
    }
}

fn handle_session_command(args: &[String]) {
    if args.len() < 3 {
        print_usage();
        std::process::exit(2);
    }

    match args[2].as_str() {
        "list" => {
            let mut path = default_session_path();
            let mut i = 3usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--session-path" => {
                        i += 1;
                        ensure_value(args, i, "--session-path");
                        path = PathBuf::from(&args[i]);
                    }
                    _ => {
                        eprintln!("unknown session flag: {}", args[i]);
                        std::process::exit(2);
                    }
                }
                i += 1;
            }
            let service = SessionService::new(path);
            match service.list_sessions() {
                Ok(list) => {
                    println!("{}", render_sessions_json(&list));
                }
                Err(err) => {
                    eprintln!("ERROR: {err}");
                    std::process::exit(1);
                }
            }
        }
        "save" => {
            let mut path = default_session_path();
            let mut name: Option<String> = None;
            let mut template = SessionTemplate::new(String::new());
            let mut i = 3usize;
            let mut raw_extra_args: Vec<String> = Vec::new();

            while i < args.len() {
                match args[i].as_str() {
                    "--session-path" => {
                        i += 1;
                        ensure_value(args, i, "--session-path");
                        path = PathBuf::from(&args[i]);
                    }
                    "--name" => {
                        i += 1;
                        ensure_value(args, i, "--name");
                        name = Some(args[i].clone());
                    }
                    "--directory" => {
                        i += 1;
                        ensure_value(args, i, "--directory");
                        template.directory = args[i].clone();
                    }
                    "--shell" => {
                        i += 1;
                        ensure_value(args, i, "--shell");
                        template.shell = Some(args[i].clone());
                    }
                    "--title" => {
                        i += 1;
                        ensure_value(args, i, "--title");
                        template.title = Some(args[i].clone());
                    }
                    _ => {
                        raw_extra_args = args[i..].to_vec();
                        break;
                    }
                }
                i += 1;
            }

            let final_name = match name {
                Some(value) => value,
                None => {
                    eprintln!("missing required --name");
                    std::process::exit(2);
                }
            };

            template.name = final_name;
            template.extra_args = normalize_extra_args(&raw_extra_args);

            let service = SessionService::new(path);
            if let Err(err) = service.upsert_session(template) {
                eprintln!("ERROR: {err}");
                std::process::exit(1);
            }
            println!("session saved");
        }
        _ => {
            print_usage();
            std::process::exit(2);
        }
    }
}

fn handle_shell_command(args: &[String]) {
    if args.len() < 3 || args[2].as_str() != "run" {
        print_usage();
        std::process::exit(2);
    }

    let mut config_path = default_config_path();
    let mut session_path = default_session_path();
    let mut i = 3usize;

    while i < args.len() {
        match args[i].as_str() {
            "--config-path" => {
                i += 1;
                ensure_value(args, i, "--config-path");
                config_path = PathBuf::from(&args[i]);
            }
            "--session-path" => {
                i += 1;
                ensure_value(args, i, "--session-path");
                session_path = PathBuf::from(&args[i]);
            }
            _ => {
                eprintln!("unknown shell flag: {}", args[i]);
                std::process::exit(2);
            }
        }
        i += 1;
    }

    let shell = DesktopShell::new(config_path, session_path);
    if let Err(err) = shell.run_with_stdio() {
        eprintln!("ERROR: {err}");
        std::process::exit(1);
    }
}
