extern crate chrono;
extern crate regex;
extern crate ansi_term;

mod gfx;
mod event_parse;
use std::env;
use std::path::Path;

use std::fs::File;
use std::io::{Read, Write};
use regex::Regex;
use ansi_term::Colour;

pub struct TtyReaderAndLogger {
    tty_usb: File,
    log_file: File,
    level_config: LevelConfig,
}

impl TtyReaderAndLogger {
    pub fn read_and_log(&mut self) -> Option<u32> {
        let mut data: [u8; 512] = [0u8; 512];
        let num_bytes = self.tty_usb.read(&mut data).unwrap();
        match std::str::from_utf8(&data[0..num_bytes]) {
            Ok(l) => {
                let txt = l.trim();
                if txt.len() == 0 {
                    None
                } else {
                    handle_value(txt, &self.level_config, &mut self.log_file)
                }
            }
            Err(e) => {
                // "Could not convert data from tty to UTF-8 string"
                println!("{}", Colour::Purple.paint(e.to_string()));
                None
            }
        }
    }
}

pub enum CoffeeLevel {
    HIGH,
    NORMAL,
    LOW,
}

#[derive(Clone, Copy)]
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


    let tty_usb = File::open(device_path.clone())
        .ok()
        .expect(format!("Could not open device {}", device_path).as_str());
    let log_file = File::create(logfile_path.clone())
        .ok()
        .expect(format!("Could not open file {} to log to", logfile_path).as_str());

    let mut event_dev = event_parse::open_device(2).unwrap();
    event_dev.read_name();
    loop {
        event_dev.read();
    }

    let lower_limit = env::args().nth(3).unwrap().parse::<u32>().unwrap();
    let upper_limit = env::args().nth(4).unwrap().parse::<u32>().unwrap();

    let path: &Path = Path::new("fonts/comicbd.ttf");
    gfx::run(path,
             TtyReaderAndLogger {
                 tty_usb: tty_usb,
                 log_file: log_file,
                 level_config: LevelConfig {
                     max: upper_limit,
                     min: lower_limit,
                 },
             });
}



fn handle_value(line: &str, level_config: &LevelConfig, log_file: &mut File) -> Option<u32> {
    if line.len() == 0 {
        None
    } else {
        let regex_pattern = r"\d+";
        let weight_matcher = Regex::new(regex_pattern).unwrap();
        let now = chrono::UTC::now();
        let data_str = format!("{}: {}\n",
                               now.format("%b %-d, %-I:%M:%S%.3f").to_string(),
                               line);
        let caps = weight_matcher.captures(line);
        let status_str = match caps {
            Some(c) => c.at(0).unwrap(),
            None => "",
        };

        let parse_res = match status_str.parse::<u32>() {
            Ok(r) => {
                (match select_level(r, level_config) {
                     CoffeeLevel::HIGH => Colour::Green.paint("HIGH"),
                     CoffeeLevel::NORMAL => Colour::Yellow.paint("NORMAL"),
                     CoffeeLevel::LOW => Colour::Red.paint("LOW"),
                 },
                 Some(r))
            }
            Err(_) => (Colour::Cyan.paint("UNKNOWN"), None),
        };

        println!("{}Coffee level: {}", data_str, parse_res.0);

        if parse_res.1 != None {
            let _ = log_file.write(data_str.into_bytes().as_slice());
            Some(parse_res.1.unwrap())
        } else {
            None
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
