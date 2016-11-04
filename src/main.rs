/*extern crate iron;

use iron::prelude::*;
use iron::status;*/

extern crate chrono;
extern crate regex;
extern crate ansi_term;

mod gfx;
use std::env;
use std::path::Path;

use std::fs::File;
use std::io::{Read, Write};
use regex::Regex;
use ansi_term::Colour;

pub enum CoffeeLevel {
    HIGH,
    NORMAL,
    LOW,
}

pub struct LevelConfig {
    max: u32,
    min: u32,
}


fn main() {
    if env::args().count() != 5 {
        println!("Usage: {} <device> <logfile> <lower_limit_in_grams> <upper_limit_in_grams>",
                 env::args().nth(0).unwrap());
        std::process::exit(1);
    }

    let device_path = env::args().nth(1).unwrap();
    let logfile_path = format!("{}{}",
                               env::args().nth(2).unwrap(),
                               chrono::UTC::now().format("%F@%H-%M-%S"));


    let mut tty_usb = File::open(device_path.clone())
        .ok().expect(format!("Could not open device {}", device_path).as_str());
    let mut log_file = File::create(logfile_path.clone())
        .ok().expect(format!("Could not open file {} to log to", logfile_path).as_str());


    let lower_limit = env::args().nth(3).unwrap().parse::<u32>().unwrap();
    let upper_limit = env::args().nth(4).unwrap().parse::<u32>().unwrap();

    main_loop(&mut tty_usb, &mut log_file, lower_limit, upper_limit);

    /*Iron::new(|_: &mut Request| {
        Ok(Response::with((status::Ok, "Hello World2!")))
    }).http("localhost:3000").unwrap();*/
}

fn main_loop(tty_usb: &mut File, log_file: &mut File, lower_limit: u32, upper_limit: u32) {
    let path: &Path = Path::new("fonts/comicbd.ttf");
    gfx::run(path, &LevelConfig { max: upper_limit, min: lower_limit }, tty_usb, log_file);
}

pub fn read_and_log(tty_usb: &mut File, mut log_file: &mut File, level_config: &LevelConfig) -> Option<u32> {
    let mut data: [u8; 512] = [0u8; 512];
    let num_bytes = tty_usb.read(&mut data).unwrap();
    match std::str::from_utf8(&data[0..num_bytes]) {
        Ok(l) => Some(handle_value(l.trim(), level_config, &mut log_file)),
        Err(e) => {
            // "Could not convert data from tty to UTF-8 string"
            println!("{}", Colour::Purple.paint(e.to_string()));
            None
        },
    }
}

fn handle_value(line: &str, level_config: &LevelConfig, log_file: &mut File) -> u32 {
    if line.len() == 0 { 0 } else {
        let regex_pattern = r"\d+";
        let weight_matcher = Regex::new(regex_pattern).unwrap();
        let now = chrono::UTC::now();
        let data_str = format!("{}: {}\n", now.format("%b %-d, %-I:%M:%S%.3f").to_string(), line);
        let caps = weight_matcher.captures(line);
        let status_str = match caps {
            Some(c) => c.at(0).unwrap(),
            None => "",
        };

        let parse_res = match status_str.parse::<u32>() {
            Ok(r) =>
                (match select_level(r, level_config) {
                    CoffeeLevel::HIGH => Colour::Green.paint("HIGH"),
                    CoffeeLevel::NORMAL => Colour::Yellow.paint("NORMAL"),
                    CoffeeLevel::LOW => Colour::Red.paint("LOW"),
                }, Some(r)),
            Err(_) => (Colour::Cyan.paint("UNKNOWN"), None),
        };

        println!("{}Coffee level: {}", data_str, parse_res.0);

        if parse_res.1 != None {
            let _ = log_file.write(data_str.into_bytes().as_slice());
            parse_res.1.unwrap()
        } else {
            0
        }
    }
}

pub fn select_level(weight: u32, config: &LevelConfig) -> CoffeeLevel {
    let padding = ((config.max - config.min) as f32 * 0.2f32) as u32;
    match weight {
        w if w > config.max - padding => CoffeeLevel::HIGH,
        w if w < config.min + padding => CoffeeLevel::LOW,
        _ => CoffeeLevel::NORMAL,
    }
}
