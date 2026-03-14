use std::env;
use std::path::PathBuf;

use kitty_desktop::{normalize_extra_args, KittyAdapter, KittyLaunchConfig};

fn print_usage() {
    eprintln!(
        "kitty_desktop core-adapter CLI\n\nUsage:\n  kitty_desktop version\n  kitty_desktop launch [--directory DIR] [--shell SHELL] [--session FILE] [--title TITLE] [--config FILE] [--dry-run] [-- ...extra args]"
    );
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
                        if i >= args.len() {
                            eprintln!("missing value for --directory");
                            std::process::exit(2);
                        }
                        config.working_directory = Some(PathBuf::from(&args[i]));
                    }
                    "--shell" => {
                        i += 1;
                        if i >= args.len() {
                            eprintln!("missing value for --shell");
                            std::process::exit(2);
                        }
                        config.shell = Some(args[i].clone());
                    }
                    "--session" => {
                        i += 1;
                        if i >= args.len() {
                            eprintln!("missing value for --session");
                            std::process::exit(2);
                        }
                        config.session_file = Some(PathBuf::from(&args[i]));
                    }
                    "--title" => {
                        i += 1;
                        if i >= args.len() {
                            eprintln!("missing value for --title");
                            std::process::exit(2);
                        }
                        config.title = Some(args[i].clone());
                    }
                    "--config" => {
                        i += 1;
                        if i >= args.len() {
                            eprintln!("missing value for --config");
                            std::process::exit(2);
                        }
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
        _ => {
            print_usage();
            std::process::exit(2);
        }
    }
}
