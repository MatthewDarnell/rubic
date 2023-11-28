use dotenv::dotenv;
use log::{debug, error, info, trace, warn, LevelFilter, SetLoggerError};
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};




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
            println!("RUBIC_LOG_FILE not found in env vars! Defaulting...");
            let mut default_path: String = match std::env::consts::OS {
                "windows" => {
                    let mut home_path: String = std::env::var("USERPROFILE")
                        .expect("Weird Error! Failed to get Env Var %USERPROFILE% on Windows!");
                    home_path.push_str("/.rubic/");
                    home_path
                },
                _ => "~/.rubic/".to_string()
            };
            println!("Defaulting to: {}", default_path);
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
            println!("Using Log File Path: <{}>", default_path.as_str());
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
                    println!("Found Invalid RUBIC_LOG_LEVEL {}! Defaulting...", v.as_str());
                    "info".to_string()
                }
            }
        },
        Err(_) => {
            println!("RUBIC_LOG_LEVEL not found in env vars! Defaulting...");
            let default_host: String = "127.0.0.1".to_string();
            println!("Using RUBIC_LOG_LEVEL: <{}>", default_host.as_str());
            return default_host;
        }
    }
}

pub fn debug(value: &str) -> Result<(), SetLoggerError> {
    let level = match get_log_level().as_str() {
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info
    };
    let file_path = get_log_file();
    // Logging to log file.
    let logfile = FileAppender::builder()
        // Pattern: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build(file_path.as_str())
        .expect("Error Building Log File!");


    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(
            Root::builder()
                .appender("logfile")
                .build(level),
        )
        .expect("Error Building Log File Config!");
    let _handle = log4rs::init_config(config)?;
    debug!("{}", value);
    Ok(())
}

pub fn error(value: &str) -> Result<(), SetLoggerError> {
    let level = match get_log_level().as_str() {
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info
    };
    let file_path = get_log_file();
    // Logging to log file.
    let logfile = FileAppender::builder()
        // Pattern: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build(file_path.as_str())
        .expect("Error Building Log File!");


    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(
            Root::builder()
                .appender("logfile")
                .build(level),
        )
        .expect("Error Building Log File Config!");
    let _handle = log4rs::init_config(config)?;
    error!("{}", value);
    Ok(())
}

pub fn info(value: &str) -> Result<(), SetLoggerError> {
    let level = match get_log_level().as_str() {
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info
    };
    let file_path = get_log_file();
    // Logging to log file.
    let logfile = FileAppender::builder()
        // Pattern: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build(file_path.as_str())
        .expect("Error Building Log File!");


    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(
            Root::builder()
                .appender("logfile")
                .build(level),
        )
        .expect("Error Building Log File Config!");
    let _handle = log4rs::init_config(config)?;
    info!("{}", value);
    Ok(())
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
