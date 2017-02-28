//! Reads a file and saves somewhere else
extern crate gpx_rust;
extern crate clap;

use std::process::exit;
use std::io::{ BufReader, BufWriter };
use std::fs::File;
use clap::{ App, Arg };

use gpx_rust::xml::{ ParseXml };
use gpx_rust::ser::{ Serialize, SerError };
use gpx_rust::gpx::{ Gpx, Parser, Error };


#[derive(Debug)]
enum ResaveError {
    Io(std::io::Error),
    Serialize(SerError),
}

fn parse(filename: &str) -> Result<Gpx, Error> {
    let f = try!(File::open(filename).map_err(Error::Io));
    let f = BufReader::new(f);
    Parser::new(f).parse()
}

fn save(filename: &str, data: Gpx) -> Result<(), ResaveError> {
    let f = try!(File::create(filename).map_err(ResaveError::Io));
    let f = BufWriter::new(f);
    data.serialize(f, "").map_err(ResaveError::Serialize)//, WspMode::IndentLevel(0)).map_err(ResaveError::Io));
}

fn main() {
    let matches = App::new("Reader")
                      .arg(Arg::with_name("source")
                              .required(true))
                      .arg(Arg::with_name("destination")
                              .required(true))
                      .get_matches();
    let data = match parse(matches.value_of("source").unwrap()) {
        Err(e) => {
            println!("Failed to load\n{}", e);
            let mut e = &e as &std::error::Error;
            while e.cause().is_some() {
                e = e.cause().unwrap() as &std::error::Error;
                println!("{}", e);
            }
            exit(1);
        }
        Ok(d) => d
    };
    save(matches.value_of("destination").unwrap(), data).expect("Failed to save");
}
