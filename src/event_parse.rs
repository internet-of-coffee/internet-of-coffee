//use std::path::Path;
use std::fs::File;
use std::io::{Read};
use std::mem;

#[derive(Debug)]
pub struct EventData {
    sec: u32,
    ms: u32,
    ev_type: u16,
    code: u16,
    value: i32,
}

#[derive(Debug)]
pub struct EventDevice {
    stream: File,
}

impl EventDevice {
    pub fn read(&mut self) -> Result<EventData, String> {
        println!("mem::size_of::<EventData>(): {}", mem::size_of::<EventData>());
        let mut buff: [u8;160] = [0u8;160];
        self.stream.read(&mut buff).unwrap();
//        match Ok(16u32) {
//            Ok(v) => {
//                println!("num_bytes: {}", v);
                //println!("data: {:?}", buff[0..v]);
//                for n in buff[0..v].iter() {
//                    println!("{} is a number!", n);
//                }
                Ok(EventData{sec:0, ms: 0,
                    ev_type: ((buff[8] as u16) << 8 | buff[9] as u16) as _,
                    code: ((buff[10] as u16) << 8 | buff[11] as u16) as _,
                    value: ((buff[12] as u32) << 24 | (buff[13] as u32) << 16 | (buff[14] as u32) << 8 | buff[15] as u32) as _})
//            },
//            Err(_) => Err("err".to_string()),
            //Err(e) => Err(format!("{:?}", e)),
//        }
    }
}

pub fn open_device(dev_nr: usize) -> Result<EventDevice, String> {
    match File::open(format!("/dev/input/event{}", dev_nr)) {
        Ok(s) => Ok(EventDevice{stream: s}),
        Err(e) => Err(format!("{:?}", e)),
    }
}