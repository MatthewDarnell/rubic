use dotenv::dotenv;
use fern;
use chrono;

use log::{LevelFilter, Metadata};
pub use log::{debug, error, info, trace };





fn get_log_file() -> String {
    dotenv().ok();
    return match std::env::var("RUBIC_LOG_FILE") {
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
                            println!("Failed To Create Log File Path <{}> Permission Error?\n\tError: ({})\n Shutting Down.",
                                     built_path.as_str(),
                                     err.to_string()
                            );
                            panic!("No Log File");
                        }
                    };
                }
            }
            v
        },
        Err(_) => {
            //println!("RUBIC_LOG_FILE not found in env vars! Defaulting...");
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
                        panic!("No Log File!");
                    }
                };
            }
            default_path.push_str("rubic.log");
            //println!("Using Log File Path: <{}>", default_path.as_str());
            return default_path;
        }
    }
}
fn get_log_level() -> String {
    dotenv().ok();
    return match std::env::var("RUBIC_LOG_LEVEL") {
        Ok(v) => {
            match v.as_str() {
                "info" => v,
                "debug" => v,
                "error" => v,
                _ => {
                    //println!("Found Invalid RUBIC_LOG_LEVEL {}! Defaulting...", v.as_str());
                    "info".to_string()
                }
            }
        },
        Err(_) => {
            //println!("RUBIC_LOG_LEVEL not found in env vars! Defaulting...");
            let default_host: String = "127.0.0.1".to_string();
            //println!("Using RUBIC_LOG_LEVEL: <{}>", default_host.as_str());
            return default_host;
        }
    }
}


fn drop_rocket(meta: &Metadata) -> bool {
    let name = meta.target();
    if name.starts_with("rocket") || name.eq("_") {
        return false;
    }
    true
}

pub fn setup_logger() -> Result<(), fern::InitError> {
    let level = match get_log_level().as_str() {
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info
    };

    let mut options = std::fs::OpenOptions::new();
    options.create(true);
    options.append(true);
    
    
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(level)
        .filter(drop_rocket)
        .chain(options.open(get_log_file().as_str())?)
        .apply()?;
    Ok(())
}

pub fn info(value: &str) {
    info!("{}", value);
}

pub fn debug(value: &str) {
    debug!("{}", value);
}

pub fn error(value: &str) {
    error!("{}", value);
}


pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
