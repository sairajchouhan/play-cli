#![allow(unused)]

use clap::{command, Arg};

use {
    clap::{arg, Command, Parser, Subcommand},
    ignore::WalkBuilder,
    serde::{Deserialize, Serialize},
    std::{fs, path::Path, path::PathBuf},
};

// #[derive(Parser, Debug)]
// #[command(author, version, about, long_about = None)]
// #[command(propagate_version = true)]
// struct Cli {
//     #[command(subcommand)]
//     actions: Actions,
//
//     /// override the default config path
//     #[arg(short, long)]
//     config_path: Option<PathBuf>,
// }
//
// #[derive(Subcommand, Debug)]
// enum Actions {
//     /// create a new project from a template
//     New {
//         #[arg(value_parser = template_names())]
//         template: String,
//
//         #[arg(short, long)]
//         name: Option<String>,
//     },
//     /// list all the projects created from a template
//     Ls {
//         #[arg(value_parser = template_names())]
//         template: String,
//     },
//     /// edit the config file
//     Config {
//         #[arg(short, long)]
//         open: bool,
//     },
// }

fn main() -> anyhow::Result<()> {
    let matches = command!()
        .arg_required_else_help(true)
        .subcommand_required(true)
        .subcommand(
            Command::new("new")
                .about("create a new project from a template")
                .arg(Arg::new("name").short('n').long("name"))
                .arg(Arg::new("template").value_parser(template_names()))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("ls")
                .about("list all the projects created from a template")
                .arg(
                    Arg::new("template")
                        .short('t')
                        .long("template")
                        .value_parser(template_names()),
                )
                .arg_required_else_help(true),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("add", sub_matches)) => println!(
            "'myapp add' was used, name is: {:?}",
            sub_matches.get_one::<String>("NAME")
        ),
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    }

    Ok(())
}

fn template_names() -> Vec<String> {
    let templates_dir = get_config().templates_dir;
    let dir_contents = fs::read_dir(templates_dir).expect("could not read the templates dir");
    let mut template_names = vec![];

    for item in dir_contents {
        let item = item.unwrap();

        if item.path().is_dir() {
            template_names.push(item.file_name().into_string().unwrap())
        }
    }

    template_names
}

fn copy_dir_recursive(src_dir: &PathBuf, dest_dir: &PathBuf) -> anyhow::Result<()> {
    let read_path = Path::new(src_dir);
    let walk_dir = WalkBuilder::new(read_path).hidden(false).build();
    let target_dir = Path::new(dest_dir);

    for item in walk_dir {
        let item = item?;
        let relative = item.path().strip_prefix(read_path);
        let relative = relative?;
        let item_type = item.file_type().unwrap();

        if item_type.is_dir() {
            fs::create_dir_all(target_dir.join(relative)).expect("Unable to create the dir");
        }

        // continue;
        if item_type.is_file() {
            let dest_path = target_dir.join(relative);
            let dest_parent = dest_path.parent().unwrap();

            if !dest_path.exists() {
                fs::create_dir_all(&dest_parent).unwrap()
            }

            fs::File::create(&dest_path).unwrap();
            fs::copy(item.path(), dest_path).unwrap();
        }
    }

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Config {
    target_dir: PathBuf,
    templates_dir: PathBuf,
}

impl Config {
    fn setup(custom_config_path: Option<&PathBuf>) -> Self {
        if let Some(config_path) = custom_config_path {
            let config_string = fs::read_to_string(config_path).unwrap();
            let config = serde_json::from_str::<Config>(config_string.as_str());

            match config {
                Ok(value) => value,
                Err(_) => error("failed to parse the config file, please check the format"),
            }

            // TODO: validate that the user has created the target and the templates dir and that they are valid
        } else {
            let user_local_config =
                dirs::config_local_dir().expect("cannot fine users local config dir");
            let config_path = user_local_config.join("play").join("play.json");

            if !user_local_config.join("play").exists() {
                fs::create_dir(user_local_config.join("play")).expect("failead to create play");
            }

            if !user_local_config.join("play").join("play.json").exists() {
                let temp = user_local_config.join("play").join("play.json");
                fs::File::create(&temp).expect("failed to create play.json");
                fs::write(
                    user_local_config.join("play").join("play.json"),
                    String::from("{}"),
                )
                .unwrap();
            }
            let target_dir = dirs::home_dir()
                .expect("could not get the uesr's home dir")
                .join("playground");

            if !target_dir.exists() {
                fs::create_dir(&target_dir).expect("cannot create target dir")
            }

            let templates_dir = target_dir.join(".templates");

            if !templates_dir.exists() {
                fs::create_dir(&templates_dir).expect("cannot create templates dir")
            }

            // write the config to the file
            let config = Config {
                target_dir: target_dir.clone(),
                templates_dir: templates_dir.clone(),
            };

            let config_string = serde_json::to_string(&config).unwrap();

            fs::write(config_path, config_string).unwrap();

            Self {
                target_dir,
                templates_dir,
            }
        }
    }
}

fn get_config_dir_path() -> PathBuf {
    dirs::config_local_dir()
        .expect("cannot get users local config dir")
        .join("play")
        .join("play.json")
}

fn get_config() -> Config {
    let config_dir_path = get_config_dir_path();
    let config_string = fs::read_to_string(config_dir_path).unwrap();
    serde_json::from_str::<Config>(config_string.as_str())
        .expect("parsing json from string to Config failed")
}

fn error(message: &str) -> ! {
    eprintln!("{}", message);
    std::process::exit(1);
}
