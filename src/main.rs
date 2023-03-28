use {
    clap::{Parser, Subcommand, ValueEnum},
    ignore::WalkBuilder,
    serde::{Deserialize, Serialize},
    std::{fs, path::Path, path::PathBuf, process::Command, str::FromStr},
};

#[derive(ValueEnum, Clone, Debug)]
enum Templates {
    TsNode,
    TsExpress,
}

impl Templates {
    fn get_template_dir(&self) -> PathBuf {
        let templates_dir = dirs::home_dir()
            .unwrap()
            .join("mine")
            .join("play-cli")
            .join("templates")
            .join(self.to_str());

        templates_dir
    }

    fn to_str(&self) -> &'static str {
        match self {
            self::Templates::TsNode => "ts-node",
            self::Templates::TsExpress => "ts-express",
        }
    }
}

impl FromStr for Templates {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ts-node" => Ok(Templates::TsNode),
            "ts-express" => Ok(Templates::TsExpress),
            _ => Err(()),
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    actions: Actions,
}

#[derive(Subcommand, Debug)]
enum Actions {
    New {
        #[arg(value_enum)]
        template: Templates,

        #[arg(short, long)]
        name: Option<String>,
    },
    Ls {
        template: Option<Templates>,
    },
    Config {
        #[arg(short, long)]
        open: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    init_config().expect("config directory initialization failed");

    let config = get_config();

    let target_dir_path = config.target_dir;

    if !target_dir_path.exists() {
        fs::create_dir(&target_dir_path).expect("target dir creation failed")
    }

    match cli.actions {
        Actions::New { template, name } => {
            let has_template_root_dir = target_dir_path.join(template.to_str()).exists();
            if !has_template_root_dir {
                fs::create_dir(target_dir_path.join(template.to_str())).unwrap();
            }
            let project_dir = name.unwrap_or(memorable_wordlist::snake_case(32));
            let project_dir_path = target_dir_path.join(template.to_str()).join(project_dir);
            copy_dir_recursive(&template.get_template_dir(), &project_dir_path).unwrap();
        }
        Actions::Ls { template } => {
            println!("Ls command for {template:?}");
        }
        Actions::Config { open } => {
            if open {
                let editor = std::env::var("EDITOR").unwrap();
                let file_path = get_config_dir_path();

                Command::new(editor)
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
        }
    }

    Ok(())
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

fn get_config_dir_path() -> PathBuf {
    dirs::config_local_dir()
        .expect("cannot get users local config dir")
        .join("play-cli")
        .join("play.json")
}

fn init_config() -> anyhow::Result<()> {
    let user_local_config = dirs::config_local_dir().expect("cannot fine users local config dir");

    if !user_local_config.join("play-cli").exists() {
        fs::create_dir(user_local_config.join("play-cli"))?;
    }

    if !user_local_config
        .join("play-cli")
        .join("play.json")
        .exists()
    {
        let temp = user_local_config.join("play-cli").join("play.json");
        fs::File::create(&temp)?;
        fs::write(temp, String::from("{}"))?;
    }

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Config {
    target_dir: PathBuf,
    // templates_dir: PathBuf,
}

fn get_config() -> Config {
    let config_dir_path = get_config_dir_path();
    let config_string = fs::read_to_string(config_dir_path).unwrap();
    serde_json::from_str::<Config>(config_string.as_str())
        .expect("parsing json from string to Config failed")
}
