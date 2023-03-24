use std::{
    error::Error,
    fs, panic,
    path::{Path, PathBuf},
    str::FromStr, collections::HashMap,
};

use rand::random;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(ValueEnum, Clone, Debug)]
enum Templates {
    TsNode,
    TsExpress,
}

impl Templates {
    fn get_local_template_dir(&self) -> PathBuf {
        match self {
            self::Templates::TsNode => Path::new("./templates/ts-node").to_path_buf(),
            self::Templates::TsExpress => Path::new("./templates/ts-express").to_path_buf(),
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
    let target_path = dirs::home_dir()
        .expect("Error while getting home dir")
        .join("playground");

    if !target_path.exists() {
        fs::create_dir(&target_path).expect("Error in creating the target dir")
    }

    let cli = Cli::parse();

    match cli.actions {
        Actions::New { template } => {
            let target_template_dir_path = target_path.join(template.to_str());

            if !target_template_dir_path.exists() || !target_template_dir_path.is_dir() {
                fs::create_dir(&target_template_dir_path).unwrap();
            }

            let mut random_folder = template.to_str().to_string();
            random_folder.push_str(&random::<u32>().to_string());

            if !target_template_dir_path.join(&random_folder).exists() {
                fs::create_dir(&target_template_dir_path.join(&random_folder))
                    .expect("failed to create a random folder in target template dir");
            }

            let local = template.get_local_template_dir();
            let all_paths_to_read = get_all_in_dir(&local);
            let all_paths_to_read = all_paths_to_read.iter().map(|x| x).collect::<Vec<_>>();



            let new_all_paths = all_paths_to_read
                .iter()
                .map(|path| {
                    let stripe = Path::new("./templates")
                        .join(template.to_str())
                        .to_path_buf();
                    path.strip_prefix(stripe).unwrap().to_path_buf()
                })
                .map(|striped| target_template_dir_path.join(&random_folder).join(striped))
                .collect::<Vec<_>>();

            let mut map: HashMap<&PathBuf, &PathBuf> = HashMap::new();
            
            for (key, value) in new_all_paths.iter().zip(all_paths_to_read) {
                map.insert(key, value);
            }

            new_all_paths.iter().for_each(|path| {
                if !path.parent().unwrap().exists() {
                    fs::create_dir_all(path.parent().unwrap()).unwrap();
                }

                fs::File::create(&path).unwrap();

                let original_path = map.get(path).unwrap();
                let contents = fs::read_to_string(original_path).unwrap();

                fs::write(&path, contents).unwrap();
            });

            println!("Project created")
        }
        Actions::Ls { template } => {
            println!("Ls command for {template:?}");
        }
    }

    Ok(())
}


fn read_gitignore(path: &PathBuf) -> String {
    let file = fs::read_to_string(path.join(".gitignore")).unwrap_or("".to_string());
    file.trim().to_string()
}

fn get_all_in_dir(path: &PathBuf) -> Vec<PathBuf> {
    let dir_contents = fs::read_dir(path).unwrap();
    let mut result: Vec<PathBuf> = vec![];

    let git_ignore = read_gitignore(&path);

    for item in dir_contents {
        let dir_entry = item.unwrap();
        let file_name = dir_entry.file_name().into_string().unwrap();

        if git_ignore.contains(&file_name) {
            continue;
        }

        let meta = dir_entry.metadata().unwrap();

        if meta.is_dir() {
            let more_results = get_all_in_dir(&path.join(&file_name));
            result.extend(more_results);
        } else {
            result.push(dir_entry.path());
        }
    }

    result
}
