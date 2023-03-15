use std::fs;
use std::io::Error as IoError;
use serde::{Deserialize, Serialize};
use toml;


#[derive(PartialEq, PartialOrd, Serialize, Deserialize, Debug)]
pub enum LogLevel{
    All,
    Info,
    Warning,
    Error,
}

#[derive(Serialize, Deserialize, Debug)]
struct ConfigTomlServer {
    port: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ConfigTomlPeriod {
    heartbeat_period: Option<u16>,
    ping_period: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ConfigTomlDebug {
    debug_level: Option<LogLevel>,
    debug_datagram_handler: Option<bool>,
    debug_ping_sender: Option<bool>,
    debug_data_handler: Option<bool>,
    debug_heartbeat_checker: Option<bool>,
    debug_topic_handler: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ConfigToml {
    server: Option<ConfigTomlServer>,
    debug: Option<ConfigTomlDebug>,
    period: Option<ConfigTomlPeriod>,
}

#[derive(Debug)]
pub struct Config {
    pub port: String,
    pub heart_beat_period: u16,
    pub ping_period: u16,
    pub debug_level: LogLevel,
    pub debug_datagram_handler: bool,
    pub debug_ping_sender: bool,
    pub debug_data_handler: bool,
    pub debug_heartbeat_checker: bool,
    pub debug_topic_handler: bool,
}

impl Config {
    pub fn new() -> Self {
        // 1 - List of path to config files
        let config_filepaths: [&str; 2] = [
            "./config.toml",
            "./src/config.toml",
        ];

        // 2 - Loop through each config file to get the first valid one
        let mut content: String = "".to_owned();

        for filepath in config_filepaths {
            let result: Result<String, IoError> = fs::read_to_string(filepath);

            if result.is_ok() {
                content = result.unwrap();
                break;
            }
        }

        // 3 - Extract the content to a toml object
        let config_toml: ConfigToml = match toml::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                println!("Failed to create ConfigToml Object out of config file. Error: {:?}", e);
                ConfigToml {
                    server: None,
                    period: None,
                    debug: None,
                }
            }
        };

        // 4 - Get every value into local variables

        // 4.1 - Server variables
        let port: u16 = match config_toml.server {
            Some(server) => {
                let port: u16 = server.port.unwrap_or_else(|| {
                    println!("Missing field port in table server.");
                    "unknown".to_owned()
                }).parse().unwrap();

                port
            }
            None => {
                println!("Missing table server.");
                3838 // Default value if none found
            }
        };

        // 4.2 - Period variables
        let (heartbeat_period, ping_period): (u16, u16) = match config_toml.period {
            Some(period) => {
                let hb_period = period.heartbeat_period.unwrap_or_else(|| {
                    println!("Missing field heartbeat_period in table period.");
                    5 // Default value if none found
                });
                let ping_period = period.ping_period.unwrap_or_else(|| {
                    println!("Missing field ping_period in table period.");
                    5 // Default value if none found
                });

                (hb_period, ping_period)
            }
            None => {
                println!("Missing table period.");
                (5, 5) // Default value if none found
            }
        };

        // 4.3 - Debug variables
        let (debug_level,
            debug_datagram_handler,
            debug_ping_sender,
            debug_data_handler,
            debug_heartbeat_checker,
            debug_topic_handler): (LogLevel, bool, bool, bool, bool, bool) = match config_toml.debug {
            Some(debug) => {
                let d_level: LogLevel = debug.debug_level.unwrap_or_else(|| {
                    println!("Missing field debug_level in table debug.");
                    LogLevel::All // Default value if none found
                });
                let d_ping = debug.debug_ping_sender.unwrap_or_else(|| {
                    println!("Missing field debug_ping_sender in table debug.");
                    true // Default value if none found
                });
                let d_datagram = debug.debug_datagram_handler.unwrap_or_else(|| {
                    println!("Missing field debug_datagram_handler in table debug.");
                    true // Default value if none found
                });
                let d_data = debug.debug_data_handler.unwrap_or_else(|| {
                    println!("Missing field debug_data_handler in table debug.");
                    true // Default value if none found
                });
                let d_heart = debug.debug_heartbeat_checker.unwrap_or_else(|| {
                    println!("Missing field debug_heartbeat_checker in table debug.");
                    true // Default value if none found
                });
                let d_topic = debug.debug_topic_handler.unwrap_or_else(|| {
                    println!("Missing field debug_topic_handler in table debug.");
                    true // Default value if none found
                });

                (d_level, d_datagram, d_ping, d_data, d_heart, d_topic)
            }
            None => {
                println!("Missing table debug.");
                (LogLevel::All, true, true, true, true, true) // Default value if none found
            }
        };


        Config {
            port: port.to_string(),
            heart_beat_period: heartbeat_period,
            ping_period,
            debug_level,
            debug_datagram_handler,
            debug_ping_sender,
            debug_data_handler,
            debug_heartbeat_checker,
            debug_topic_handler,
        }
    }
}
