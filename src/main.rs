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
                let all_paths_to_read = get_all_in_dir(path, Some(&git_ignore_str));
                let parent_path = target_location.join(Templates::TsNode.to_str());

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
                    let content = fs::read_to_string(x);
                    let content = match content {
                        Ok(stuff) => stuff,
                        Err(_e) => panic!("could not read file"),
                    };

                    let path = match x.to_str() {
                        Some(path) => String::from(path),
                        None => panic!("just panic"),
                    };
                    let path = path.replace("./templates", "./final");
                    let final_path = Path::new(&path);
                    let file = fs::File::create(final_path);

                    match file {
                        Ok(_) => (),
                        Err(e) => {
                            eprintln!("{:?}", e);
                            panic!("File creation falied")
                        }
                    }

                    let write_res = fs::write(final_path, content);

                    match write_res {
                        Ok(_) => (),
                        Err(err) => {
                            println!("{}", err);
                            panic!("Could not write file")
                        }
                    }
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
    let file_result = fs::read_to_string(path.join(".gitignore"));
    let file = match file_result {
        Ok(file) => file,
        Err(_e) => {
            println!("No .gitignore file found");
            "".to_string()
        }
    };

    file.trim().to_string()
}

fn get_all_in_dir(path: &Path, ignore: Option<&String>) -> Vec<PathBuf> {
    let read_dir = fs::read_dir(path);
    let mut result: Vec<PathBuf> = vec![];

    if let Ok(dir_contents) = read_dir {
        for item in dir_contents {
            if let Ok(dir_entry) = item {
                let file_name_res = dir_entry.file_name().into_string();

                let file_name = match file_name_res {
                    Ok(file_name) => file_name,
                    Err(e) => {
                        eprintln!("{:?}", e);
                        panic!("OS string into string gone wrong")
                    }
                };

                if let Some(git_ignore) = ignore {
                    if git_ignore.contains(&file_name) {
                        continue;
                    }
                }

                let result_meta = dir_entry.metadata();
                match result_meta {
                    Ok(meta) => {
                        if meta.is_dir() {
                            get_all_in_dir(&dir_entry.path(), None);
                        } else {
                            result.push(dir_entry.path());
                        }
                    }
                    Err(_e) => {
                        panic!("meta for result not found")
                    }
                }
                println!("{:?}", dir_entry)
            } else {
                panic!("else not a valid direntry")
            }
        }
    } else {
        panic!("else block: read_dir got mad")
    }

    result
}
