use {
    clap::{Parser, Subcommand, ValueEnum},
    serde::{Deserialize, Serialize},
    std::{fs, io, path::PathBuf, str::FromStr},
};

#[derive(ValueEnum, Clone, Debug)]
enum Templates {
    TsNode,
    TsExpress,
}

impl Templates {
    fn get_templates_dir(&self) -> PathBuf {
        let templates_dir = dirs::home_dir()
            .unwrap()
            .join("mine")
            .join("play-cli")
            .join("templates")
            .join(self.to_str());

        match self {
            self::Templates::TsNode => templates_dir,
            self::Templates::TsExpress => templates_dir,
        }
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
    /// displays config file contents
    Config,
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
            copy_dir_recursive(&template.get_templates_dir(), &project_dir_path).unwrap();
        }
        Actions::Ls { template } => {
            println!("Ls command for {template:?}");
        }
        Actions::Config => {
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

    Ok(())
}

fn copy_dir_recursive(src_dir: &PathBuf, target_dir: &PathBuf) -> io::Result<()> {
    if !src_dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Source directory does not exist",
        ));
    }

    if !target_dir.exists() {
        fs::create_dir_all(target_dir)?;
    }

    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        let target_path = target_dir.join(entry.file_name());
        if entry_type.is_dir() {
            copy_dir_recursive(&entry.path(), &target_path)?;
        } else {
            fs::copy(&entry.path(), &target_path)?;
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
}

fn get_config() -> Config {
    let config_dir_path = get_config_dir_path();
    let config_string = fs::read_to_string(config_dir_path).unwrap();
    serde_json::from_str::<Config>(config_string.as_str())
        .expect("parsing json from string to Config failed")
}
