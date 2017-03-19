/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate gpx_rust;
extern crate clap;

use std::io::BufReader;
use std::fs::File;
use clap::{App, Arg};
use gpx_rust::xml::ParseXml;
use gpx_rust::gpx::{ Gpx, Parser, Error };


fn parse(filename: &str) -> Result<Gpx, Error> {
    let f = try!(File::open(filename).map_err(Error::Io));
    let f = BufReader::new(f);
    Parser::new(f).parse()
}
 
fn main() {
    let matches = App::new("Reader")
                      .arg(Arg::with_name("filename")
                              .required(true))
                      .get_matches();
    let out = parse(matches.value_of("filename").unwrap()).expect("fail");
    println!("{:?}", out);
}
