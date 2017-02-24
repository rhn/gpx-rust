#[macro_use]
extern crate quote;

mod xsd_types;

pub mod gpx;


use std::io;
use std::process::{ Command, ExitStatus };
use std::collections::HashMap;
use std::path::Path;

use xsd_types::Type;


pub type TagMap<'a> = HashMap<&'a str, &'a str>;
pub type AttrMap = HashMap<String, (String, String)>;

/// This is ugly
pub struct StructInfo<'a> {
    pub name: String,
    pub type_: &'a Type<'a>,
    pub tags: TagMap<'a>,
}

/// This is awful
pub struct ParserInfo<'a> {
    pub name: String,
    pub type_: &'a Type<'a>,
    pub attrs: AttrMap,
}


#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Command(ExitStatus),
}


pub trait ParserGen {
    fn header() -> &'static str;
    fn parser_cls(name: &str, data: &Type, types: &HashMap<String, (String, String)>) -> String;
    fn parser_impl(name: &str, data: &Type, types: &HashMap<String, (String, String)>) -> String;
    fn serializer_impl(cls_name: &str, tags: &TagMap, data: &Type) -> String;
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
