extern crate dotenv_codegen;

use dotenv::dotenv;
pub mod sqlite;


pub fn get_db_path() -> String {
    dotenv().ok();
    return match std::env::var("RUBIC_DB") {
        Ok(v) => {
            let p = std::path::Path::new(v.as_str());
            if !p.is_file() {
                let s: Vec<&str> = v.as_str().split("/").collect();
                let mut built_path: String = String::from("");
                for (index, part) in s.iter().enumerate() {
                    if index < s.len() - 1 {
                        built_path.push_str(part);
                        built_path.push_str("/");
                    }
                }
                let path_without_file = std::path::Path::new(built_path.as_str());
                if !path_without_file.exists() {
                    println!("Path <{}> Does NOT Exist. Creating!", built_path.as_str());
                    match std::fs::create_dir(path_without_file) {
                        Ok(_) => {},
                        Err(err) => {
                            println!("Failed To Create Db Path <{}> Permission Error?\n\tError: ({})\n Shutting Down.",
                                     built_path.as_str(),
                                     err.to_string()
                            );
                            panic!("No DB");
                        }
                    };
                }
            }
            v
        },
        Err(_) => {
            //println!("RUBIC_DB not found in env vars! Defaulting...");
            let mut default_path: String = match std::env::consts::OS {
                "windows" => {
                    let mut home_path: String = std::env::var("USERPROFILE")
                        .expect("Weird Error! Failed to get Env Var %USERPROFILE% on Windows!");
                    home_path.push_str("/.rubic/");
                    home_path
                },
                _ => {
                    match home::home_dir() {
                        Some(path) => {
                            let mut value: String =  path.as_path().to_str().unwrap().to_string();
                            value += "/.rubic/";
                            value
                        },
                        None => panic!("Impossible to get your home dir!"),
                    }
                    //"~/.rubic/".to_string()
                }
            };
            if !std::path::Path::new(default_path.as_str()).exists() {
                println!("Path <{}> Does NOT Exist. Creating!", default_path.as_str());
                match std::fs::create_dir(std::path::Path::new(default_path.as_str())) {
                    Ok(_) => {},
                    Err(err) => {
                        println!("Failed To Create Db Path <{}> Permission Error?\n\tError: ({})\n Shutting Down.",
                                 default_path.as_str(),
                                 err.to_string()
                        );
                        panic!("No DB");
                    }
                };
            }
            default_path.push_str("rubic.sqlite");
            return default_path;
        }
    }
}
