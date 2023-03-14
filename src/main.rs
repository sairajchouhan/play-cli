use std::{
    env,
    error::Error,
    fs,
    panic,
    path::{Path, PathBuf},
    str::FromStr,
};

enum Action {
    New,
    Ls,
}

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

fn main() -> Result<(), Box<dyn Error>> {
    let action = env::args().nth(1);
    let template = env::args().nth(2);

    let action = match action {
        Some(item) => {
            if item == "new" {
                Action::New
            } else if item == "ls" {
                Action::Ls
            } else {
                panic!("now a valid action")
            }
        }
        None => panic!("First argument action is not passed"),
    };

    let template = match template {
        Some(item) => item,
        None => panic!("Second argument cannot be empty"),
    };

    let target_location = Path::new("./final");

    match action {
        Action::New => match Templates::from_str(&template) {
            Ok(template_enum) => {
                let path = template_enum.get_local_template_dir();
                let git_ignore_str = read_gitignore(path);
                let all_paths_to_read = get_all_in_dir(path, Some(git_ignore_str));
                let parent_path = target_location.join(template_enum.to_str());

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
                    let mut final_path = Path::new(target_location).join(template_enum.to_str());

                    for each_thing in path {
                        let temp_dir = final_path.join(each_thing);
                        if !temp_dir.exists() {
                            fs::create_dir(temp_dir).unwrap();
                            final_path.push(each_thing);
                        }
                    }

                    let path = x.to_str().unwrap();
                    let path = path.replace("./templates", "./final");
                    let final_path = Path::new(&path);

                    let content = fs::read_to_string(x).unwrap();
                    fs::File::create(final_path).expect("file creation failed");
                    fs::write(final_path, content).expect("file write failed");
                });
            }
            Err(_e) => {
                eprintln!("Invliad template")
            }
        },
        Action::Ls => {}
    };

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
