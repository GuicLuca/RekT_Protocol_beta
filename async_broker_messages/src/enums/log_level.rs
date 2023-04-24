use serde::{Deserialize, Serialize};
/**
 * LogLevel are used to filter log messages
 */
#[derive(PartialEq, PartialOrd, Serialize, Deserialize, Debug)]
pub enum LogLevel {
    All,
    Info,
    Warning,
    Error,
    Quiet,
}

/**
 * This function convert a LogLevel into a string
 *
 * @param loglevel: LogLevel, The source to convert
 *
 * @return String
 */
pub fn display_loglevel<'a>(loglevel: LogLevel) -> &'a str {
    match loglevel {
        LogLevel::All => "All",
        LogLevel::Info => "Info",
        LogLevel::Warning => "Warning",
        LogLevel::Error => "Error",
        LogLevel::Quiet => "Quiet",
    }
}