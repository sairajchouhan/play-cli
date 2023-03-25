use {
    clap::{Parser, Subcommand, ValueEnum},
    std::{error::Error, fs, io, path::PathBuf, str::FromStr},
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
}

fn main() -> Result<(), Box<dyn Error>> {
    let target_path = dirs::home_dir()
        .expect("Error while getting home dir")
        .join("playground");

    if !target_path.exists() {
        fs::create_dir(&target_path).expect("Error in creating the target dir")
    }

    let cli = Cli::parse();

    match cli.actions {
        Actions::New { template, name } => {
            let has_template_root_dir = target_path.join(template.to_str()).exists();
            if !has_template_root_dir {
                fs::create_dir(target_path.join(template.to_str())).unwrap();
            }
            let project_dir = name.unwrap_or(memorable_wordlist::snake_case(32));
            let project_dir_path = target_path.join(template.to_str()).join(project_dir);
            copy_dir_recursive(&template.get_templates_dir(), &project_dir_path).unwrap();
        }
        Actions::Ls { template } => {
            println!("Ls command for {template:?}");
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
