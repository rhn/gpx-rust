#[macro_use]
extern crate quote;

mod xsd_types;

pub mod gpx;


use std::io;
use std::process::{ Command, ExitStatus };
use std::collections::HashMap;
use std::path::Path;

use xsd_types::XsdType;


pub type TagMap<'a> = HashMap<&'a str, &'a str>;

pub struct StructInfo<'a> {
    pub name: String,
    pub type_: &'a XsdType<'a>,
    pub tags: TagMap<'a>,
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Command(ExitStatus),
}


pub trait ParserGen {
    fn header() -> &'static str;
    fn serializer_impl(cls_name: &str, tags: &TagMap, data: &XsdType) -> String;
}

pub fn prettify(path: &Path) -> Result<(), Error> {
    let status = Command::new("rustfmt").arg(path)
                                        .spawn().map_err(Error::Io)?
                                        .wait().map_err(Error::Io)?;
    match status.success() {
        true => Ok(()),
        false => Err(Error::Command(status)),
    }
}
