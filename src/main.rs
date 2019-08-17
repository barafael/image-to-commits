extern crate chrono;
extern crate png;
extern crate resize;

use chrono::prelude::*;
use resize::Pixel::Gray8;
use resize::Type::Triangle;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufRead};
use std::path::Path;
use std::io::BufReader;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() == 2 {
        if args[1] == String::from("init") {
            init_stamp();
            return;
        } else {
            println!("todo: clap and usage");
            return;
        }
    } else if args.len() != 3 {
        return println!("Usage: {} in.png repo-url", args[0]);
    }

    let stamp = if let Ok(file) = File::open("init_timestamp.txt") {
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        let _len = reader.read_line(&mut line);
        match line.trim().parse::<i64>() {
            Ok(num) => num,
            Err(e) => {
                println!("timestamp file unreadable!");
                return;
            }
        }
    } else {
        println!("Timestamp file not found!");
        return;
    };

    println!("found stamp! {}", stamp);

    let mut year = resize_to_year(&args[1]);

    let index = nth_day_of_year(363, &year);
    //year[index] = 255;
    for pixel in &mut year {
        *pixel = (*pixel / 10) * 10;
    }

    let outfh = File::create("scaled.png").expect("Couldn't create tmp output file");
    let encoder = png::Encoder::new(outfh, 52u32, 7u32);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&year)
        .unwrap();
}

fn init_stamp() {
    let stamp = Local::now().timestamp();

    let path = Path::new("init_timestamp.txt");
    let display = path.display();
    let mut file = match File::create(&path) {
        Err(why) => panic!(
            "couldn't create timestamp file {}: {}",
            display,
            why.description()
        ),
        Ok(file) => file,
    };
    match file.write_all(format!("{}", stamp).as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why.description()),
        Ok(_) => println!("successfully wrote to {}", display),
    }
}

fn resize_to_year(filename: &str) -> Vec<u8> {
    let decoder = png::Decoder::new(File::open(filename).expect("Could not open file!"));
    let (info, mut reader) = decoder.read_info().expect("Could not read info!");
    dbg!(&info);
    let mut src = vec![0; info.buffer_size()];
    reader
        .next_frame(&mut src)
        .expect("Couldn't read image into buffer");

    let (w1, h1) = (info.width as usize, info.height as usize);
    let (w2, h2) = (52, 7);
    let mut dst = vec![0; w2 * h2];
    resize::resize(w1, h1, w2, h2, Gray8, Triangle, &src, &mut dst);
    dst
}

fn nth_day_of_year(day: usize, year: &Vec<u8>) -> usize {
    assert_eq!(52 * 7, year.len());
    assert!(day < year.len());
    let row = day / 7;
    let col = day % 7;
    row * 7 + col
}
