use std::{
    env,
    error::Error,
    fs::{self, read_dir},
    io,
    path::{Path, PathBuf},
    str::Lines,
    thread::panicking,
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

    match action {
        Action::New => {
            if template == Templates::TsNode.to_str() {
                let path = Templates::TsNode.get_local_template_dir();
                let git_ignore_str = git_ignore(path);
                get_all_in_dir(path, Some(&git_ignore_str));
            }
        }
        Action::Ls => {}
    };

    Ok(())
}

fn git_ignore(path: &Path) -> String {
    let file_result = fs::read_to_string(path.join(".gitignore"));
    let file = match file_result {
        Ok(file) => file,
        Err(_e) => panic!("cannot read to string .gitignore"),
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
