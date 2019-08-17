extern crate chrono;
extern crate png;
extern crate resize;
extern crate clap;
extern crate git2;

use std::fs::OpenOptions;
use git2::{Oid, Signature};
use std::path::Path;
use chrono::prelude::*;
use resize::Pixel::Gray8;
use resize::Type::Triangle;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufRead};
use std::io::BufReader;
use std::ops::Add;
use std::time::Duration;
use clap::{Arg, App, SubCommand};
use std::io::Write;
use git2::{Commit, ObjectType, Repository};

fn main() {
    let matches = App::new("image-to-commits")
        .version("0.1")
        .author("Rafael B. <rafael.bachmann.93@gmail.com>")
        .about("Generate a slow-moving image on the github contributions banner pixel by pixel over an entire year.")
        .arg(Arg::with_name("image")
            .short("i")
            .long("image")
            .value_name("image.png")
            .help("Sets an input image file. Must be grayscale png.")
            .required(true)
            .takes_value(true))
        .subcommand(SubCommand::with_name("init")
            .about("initializes directory with current timestamp.")
            .version("0.1")
            .author("Rafael B. <rafael.bachmann.93@gmail.com>")
            .arg(Arg::with_name("repo-url")
                     .short("r")
                     .long("repo-url")
                     .value_name("<url to your repo>")
                     .help("Sets the github repo to use")
                     .required(true)
                     .takes_value(true)))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("init") {
        if let Some(url) = matches.value_of("repo-url") {
            println!("Initializing timestamp! Repo url: {}", url);
            init_stamp();
        } else {
            eprintln!("Must supply a repo url to start with.");
            return;
        }
    }
    let image_file_name = if let Some(image) = matches.value_of("image") {
        image
    } else {
        println!("Must supply an image file!");
        return;
    };

    let repo_root = "../banner/";
    let repo = Repository::open(repo_root).expect("Couldn't open repository");
    println!("{} state={:?}", repo.path().display(), repo.state());
    let relative_path = Path::new("quotes.txt");
    {
        let file_path = Path::new(repo_root).join(relative_path);
        let mut file = OpenOptions::new().write(true).create(true).open(file_path).expect("Could not create quotes file!");
        file.write_all(get_quote().as_bytes()).unwrap();
    }
    let commit_id = add_and_commit(&repo, &relative_path, &get_commit_message())
        .expect("Couldn't add file to repo");
    println!("New commit: {}", commit_id);

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
        println!("Timestamp file not found. Init first!");
        return;
    };

    println!("found stamp! {}", stamp);

    let stamp_date = NaiveDateTime::from_timestamp(stamp, 0);
    println!( "stamp init date: {}", stamp_date);

    let one_day_after = NaiveDateTime::from_timestamp(stamp + 60 * 60 * 24, 0);
    dbg!(one_day_after);

    let mut year = resize_to_year(image_file_name);

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

fn add_and_commit(repo: &Repository, path: &Path, message: &str) -> Result<Oid, git2::Error> {
    let mut index = repo.index()?;
    index.add_path(path)?;
    let oid = index.write_tree()?;
    let signature = Signature::now("Rafael Bachmann", "rafael.bachmann.93@gmail.com")?;
    let parent_commit = find_last_commit(&repo)?;
    let tree = repo.find_tree(oid)?;
    repo.commit(Some("HEAD"),
                &signature,
                &signature,
                message,
                &tree,
                &[&parent_commit])
}

fn get_quote() -> String {
    String::from("Some stupid quote")
}

fn get_commit_message() -> String {
    String::from("Get some real commit messages from whatthecommit.com")
}

fn find_last_commit(repo: &Repository) -> Result<Commit, git2::Error> {
    let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
    obj.into_commit().map_err(|_| git2::Error::from_str("Couldn't find commit"))
}
