use {
    clap::{command, Arg, ArgAction, Command},
    ignore::WalkBuilder,
    serde::{Deserialize, Serialize},
    std::{collections::HashMap, env, fs, path::Path, path::PathBuf},
};

fn main() -> anyhow::Result<()> {
    let config = Config::setup();

    let matches = command!()
        .arg_required_else_help(true)
        .subcommand_required(true)
        .subcommand(
            Command::new("new")
                .about("create a new project from a template")
                .arg(
                    Arg::new("template")
                        .value_parser(template_names(&config))
                        .required(true),
                )
                .arg(
                    Arg::new("name")
                        .short('n')
                        .long("name")
                        .help("name of the project"),
                )
                .arg(
                    Arg::new("tempfile")
                        .long("tempfile")
                        .hide(false)
                        .required(false),
                )
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("ls")
                .about("list all the projects created from a template")
                .arg(
                    Arg::new("template")
                        .value_parser(template_names(&config))
                        .required(true),
                )
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("config").about("config file").arg(
                Arg::new("open")
                    .short('o')
                    .long("open")
                    .action(ArgAction::SetTrue)
                    .help("open the config file in the default editor"),
            ),
        )
        .subcommand(Command::new("alias").arg(Arg::new("shell")))
        .get_matches();

    match matches.subcommand() {
        Some((subcommand, sub_matches)) => match subcommand {
            "new" => {
                let template = sub_matches.get_one::<String>("template").unwrap();
                let name = sub_matches.get_one::<String>("name");

                let target_dir_path = config.target_dir;
                let has_template_root_dir = target_dir_path.join(&template).exists();
                if !has_template_root_dir {
                    fs::create_dir(target_dir_path.join(&template)).unwrap();
                }

                let fancy_name = memorable_wordlist::snake_case(32).clone();
                let project_dir = if let Some(thing) = name {
                    thing
                } else {
                    &fancy_name
                };

                let project_dir_path = target_dir_path.join(&template).join(project_dir);

                let template_hash_map_option = config.external.get(template);

                match template_hash_map_option {
                    Some(cmd) => {
                        let tempfile = sub_matches.get_one::<String>("tempfile").unwrap();
                        fs::write(tempfile, cmd).unwrap();
                    }
                    None => {
                        let src_template_dir = config.templates_dir.join(template);
                        copy_dir_recursive(&src_template_dir, &project_dir_path).unwrap();
                    }
                }
            }
            "ls" => {
                let template = sub_matches.get_one::<String>("template").unwrap();
                let path = &config.target_dir.join(template);

                fs::read_dir(path).unwrap().for_each(|entry| {
                    let thing = entry.unwrap();
                    if thing.file_type().unwrap().is_dir() {
                        println!("{}", thing.file_name().into_string().unwrap())
                    }
                })
            }
            "config" => {
                let open = sub_matches.get_flag("open");
                if open {
                    let editor = std::env::var("EDITOR").unwrap();
                    let file_path = get_config_dir_path();

                    std::process::Command::new(editor)
                        .arg(&file_path)
                        .status()
                        .expect("Something went wrong");
                } else {
                    let config = fs::read_to_string(get_config_dir_path());

                    match config {
                        Ok(value) => {
                            println!("{}", value)
                        }
                        Err(e) => {
                            eprintln!("{e:?}");
                        }
                    }
                }
            },
            "alias" => {
                let res = fs::read_to_string("alias.sh").unwrap();
                print!("{res}");
            },
            _ => {
                error("Bro what are you doing");
            }
        },
        None => {
            error("Bro what are you doing");
        }
    }

    Ok(())
}

fn template_names(config: &Config) -> Vec<String> {
    let templates_dir = &config.templates_dir;
    let dir_contents = fs::read_dir(templates_dir).expect("could not read the templates dir");
    let mut template_names: Vec<String> = vec![];

    for item in dir_contents {
        let item = item.unwrap();

        if item.path().is_dir() {
            template_names.push(item.file_name().into_string().unwrap())
        }
    }

    for item in &config.external {
        let key = item.0;
        template_names.push(key.clone());
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

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    target_dir: PathBuf,
    templates_dir: PathBuf,
    external: HashMap<String, String>,
}

impl Config {
    fn setup() -> Self {
        let config_path = get_config_dir_path();
        let is_custom_config = env::var("PLAY_CONFIG").is_ok();

        if is_custom_config {
            if !config_path.exists() {
                // NOTE: can prompt for asking "should we create the file for you ?"
                error("the config file does not exist, please create it");
            } else {
                let config_string = fs::read_to_string(&config_path).unwrap();
                let config = serde_json::from_str::<Config>(&config_string).unwrap();
                return config;
            }
        } else {
            if !config_path.exists() {
                fs::create_dir_all(config_path.parent().unwrap()).unwrap();
                fs::File::create(&config_path).unwrap();

                let user_home_dir = dirs::home_dir().expect("cannot find the users home dir");
                let target_dir = user_home_dir.join("playground");
                let templates_dir = user_home_dir.join("playground").join(".templates");

                fs::create_dir_all(&target_dir).unwrap();
                fs::create_dir_all(&templates_dir).unwrap();

                let default_config = Config {
                    target_dir,
                    templates_dir,
                    external: HashMap::new(),
                };
                let config_string =
                    serde_json::to_string(&default_config).expect("failed to serialize the config");

                fs::write(&config_path, config_string).unwrap();

                return default_config;
            } else {
                let config_string = fs::read_to_string(&config_path).unwrap();
                let config = serde_json::from_str::<Config>(&config_string).unwrap();
                return config;
            }
        }
    }
}

fn get_config_dir_path() -> PathBuf {
    env::var("PLAY_CONFIG")
        .ok()
        .map(|x| PathBuf::from(x))
        .unwrap_or_else(|| {
            dirs::config_local_dir()
                .expect("cannot get users local config dir")
                .join("play")
                .join("play.json")
        })
}

fn error(message: &str) -> ! {
    eprintln!("{}", message);
    std::process::exit(1);
}
