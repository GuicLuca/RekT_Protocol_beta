use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::{stdin, Write};
use std::net::TcpStream;

use regex::Regex;

pub fn send_bytes(mut stream: &TcpStream, bytes: &[u8]) -> bool {
    let size = stream.write(bytes);
    if size.is_err() {
        println!("{}", size.unwrap_err());
        return false;
    }
    let mut percentage = 0;
    if bytes.len() > 0 {
        percentage = (size.unwrap() / bytes.len()) * 100;
    }
    println!("{percentage}% bytes sent");
    return true;
}

pub fn string_to_hash(str: &String) -> u64 {
    let mut hasher = DefaultHasher::new();
    str.hash(&mut hasher);
    return hasher.finish();
}


pub fn get_cli_input(prompt: &str, err_msg: &str, string_match: Option<&Vec<String>>, regex: Option<&Regex>, must_be_i16: bool) -> String {
    let mut is_valid = false;
    let mut value = String::new();

    while !is_valid {
        value = String::from("");
        println!("{prompt}");
        stdin().read_line(&mut value).expect("Could not read user input");
        value = value.trim().parse().unwrap();

        if must_be_i16 && value.parse::<i16>().is_ok() || string_match.is_some() && string_match.unwrap().contains(&value) || regex.is_some() && regex.unwrap().is_match(&value) {
            is_valid = true
        } else {
            println!("{err_msg}");
        }
    }

    return value;
}

pub fn get_bytes_from_slice(buffer: &[u8], from: usize, to: usize) -> Vec<u8> {
    let mut vec: Vec<u8> = Default::default();
    if to < from {
        panic!("from is greater than to");
    }
    vec.copy_from_slice(&buffer[from..to]);
    return vec;
}