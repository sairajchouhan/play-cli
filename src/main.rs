use std::{
    env,
    error::Error,
    fs, panic,
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::{Parser, Subcommand, ValueEnum};

#[derive(ValueEnum, Clone, Debug)]
enum Templates {
    TsNode,
    TsExpress,
}

impl Templates {
    fn get_local_template_dir(&self) -> &Path {
        match self {
            self::Templates::TsNode => Path::new("./templates/ts-node"),
            self::Templates::TsExpress => Path::new("./templates/ts-express"),
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
    },
    Ls {
        template: Option<Templates>,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let target_dir = dirs::home_dir()
        .expect("Error while getting home dir")
        .join("playground");

    if !target_dir.exists() {
        fs::create_dir(&target_dir).expect("Error in creating the target dir")
    }

    let cli = Cli::parse();

    match cli.actions {
        Actions::New { template } => {
            let target_string = env::args()
                .nth(3)
                .unwrap_or(target_dir.to_str().unwrap().to_string());
            let target_location = Path::new(&target_string);
            let path = template.get_local_template_dir();
            let git_ignore_str = read_gitignore(path);
            let all_paths_to_read = get_all_in_dir(path, Some(git_ignore_str));
            let parent_path = target_location.join(template.to_str());

            if !target_location.exists() {
                fs::create_dir(&target_location).unwrap();
            }

            if target_location.is_dir() {
                if !parent_path.exists() || !parent_path.is_dir() {
                    fs::create_dir(&parent_path).unwrap();
                }

                if parent_path.is_file() {
                    panic!("The thing is already file");
                }
            }

            all_paths_to_read.iter().for_each(|x| {
                let mut path = x.to_str().unwrap().split("/").skip(3).collect::<Vec<_>>();
                path.pop();
                let mut final_path = Path::new(target_location).join(template.to_str());

                for each_thing in path {
                    let temp_dir = final_path.join(each_thing);
                    if !temp_dir.exists() {
                        fs::create_dir(temp_dir).unwrap();
                        final_path.push(each_thing);
                    }
                }

                let path = x.to_str().unwrap();
                let path = path.replace("./templates", target_location.to_str().unwrap());
                let final_path = Path::new(&path);

                let content = fs::read_to_string(x).unwrap();
                fs::File::create(final_path).expect("file creation failed");
                fs::write(final_path, content).expect("file write failed");
            });
            println!("Done writing")
        }
        Actions::Ls { template } => {
            println!("Ls command for {template:?}")
        }
    }

    Ok(())
}

fn read_gitignore(path: &Path) -> String {
    let file = fs::read_to_string(path.join(".gitignore")).unwrap_or("".to_string());
    file.trim().to_string()
}

fn get_all_in_dir(path: &Path, ignore: Option<String>) -> Vec<PathBuf> {
    let dir_contents = fs::read_dir(path).unwrap();
    let mut result: Vec<PathBuf> = vec![];
    let git_ignore = ignore.unwrap_or(String::from(""));

    for item in dir_contents {
        let dir_entry = item.unwrap();
        let file_name = dir_entry.file_name().into_string().unwrap();

        if git_ignore.contains(&file_name) {
            continue;
        }

        let meta = dir_entry.metadata().unwrap();

        if meta.is_dir() {
            let more_results = get_all_in_dir(&path.join(&file_name), None);
            result.extend(more_results);
        } else {
            result.push(dir_entry.path());
        }
    }

    result
}
