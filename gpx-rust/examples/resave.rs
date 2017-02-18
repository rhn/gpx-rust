//! Reads a file and saves somewhere else
extern crate gpx_rust;
extern crate clap;

use std::io::{ BufReader, BufWriter };
use std::fs::File;
use clap::{ App, Arg };
use gpx_rust::xml::{ ParseXml, WspMode };
use gpx_rust::ser::Serialize;
use gpx_rust::gpx::{ Gpx, Parser, Error };


#[derive(Debug)]
enum ResaveError {
    Io(std::io::Error)
}

fn parse(filename: &str) -> Result<Gpx, Error> {
    let f = try!(File::open(filename).map_err(Error::Io));
    let f = BufReader::new(f);
    Parser::new(f).parse()
}

fn save(filename: &str, data: Gpx) -> Result<(), ResaveError> {
    let f = try!(File::create(filename).map_err(ResaveError::Io));
    let f = BufWriter::new(f);
    data.serialize(f).map_err(ResaveError::Io)//, WspMode::IndentLevel(0)).map_err(ResaveError::Io));
}

fn main() {
    let matches = App::new("Reader")
                      .arg(Arg::with_name("source")
                              .required(true))
                      .arg(Arg::with_name("destination")
                              .required(true))
                      .get_matches();
    let data = parse(matches.value_of("source").unwrap()).expect("Failed to load");
    save(matches.value_of("destination").unwrap(), data).expect("Failed to save");
}
