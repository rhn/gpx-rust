extern crate gpx_rust;
extern crate clap;

use std::io::BufReader;
use std::fs::File;
use clap::{App, Arg};
use gpx_rust::xml::ParseXml;
use gpx_rust::gpx::{ Gpx, GpxParser, Error };


fn parse(filename: &str) -> Result<Gpx, Error> {
    let f = try!(File::open(filename).map_err(Error::Io));
    let f = BufReader::new(f);
    GpxParser::new(f).parse()
}
 
fn main() {
    let matches = App::new("Reader")
                      .arg(Arg::with_name("filename")
                              .required(true))
                      .get_matches();
    let out = parse(matches.value_of("filename").unwrap()).expect("fail");
    println!("{:?}", out);
}
