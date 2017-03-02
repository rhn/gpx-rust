#[macro_use]
extern crate quote;

mod xsd_types;

pub mod gpx;


use std::io;
use std::process::{ Command, ExitStatus };
use std::collections::HashMap;
use std::path::Path;

use xsd_types::{ Type, Element, ElementMaxOccurs };


pub type TagMap<'a> = HashMap<&'a str, &'a str>;
pub type AttrMap = HashMap<String, (String, String)>;

/// This is ugly
pub struct StructInfo<'a> {
    pub name: String,
    pub type_: &'a Type,
    pub tags: TagMap<'a>,
}

/// This is awful
pub struct ParserInfo<'a> {
    pub name: String,
    pub type_: &'a Type,
}

pub enum TypeConverter {
    ParserClass(String),
    ParseFun(String),
    AttributeFun(String),
    UniversalClass(String),
}

impl<'a> From<&'a str> for TypeConverter {
    fn from(data: &str) -> TypeConverter {
        TypeConverter::ParseFun(data.into())
    }
}

pub struct UserType(String);

impl UserType {
    fn as_str(&self) -> &str { &self.0 }
    fn as_user_type(&self) -> &str { self.as_str() }
}

impl<'a> From<&'a str> for UserType {
    fn from(data: &'a str) -> UserType { UserType(data.into()) }
}

pub type TypeMap = HashMap<String, (UserType, TypeConverter)>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Command(ExitStatus),
}


pub trait ParserGen {
    fn header() -> &'static str;
    fn parser_cls(name: &str, data: &Type, type_convs: &TypeMap) -> String;
    fn parser_impl(name: &str, data: &Type, types: &TypeMap) -> String;
    //fn build_impl(cls_name: &str, data: &Type, tage: &TagMap) -> String;
    fn serializer_impl(cls_name: &str, tags: &TagMap, data: &Type,
                       type_convs: &HashMap<String, String>) -> String;
}

pub fn ident_safe(name: &str) -> &str {
    match name {
        "type" => "type_",
        n => n
    }
}

pub fn get_elem_fieldname(e: &Element) -> String {
    ident_safe(match e.max_occurs {
        ElementMaxOccurs::Some(0) => panic!("Element has 0 occurrences, can't derive name"),
        ElementMaxOccurs::Some(1) => e.name.clone(),
        _ => format!("{}s", e.name)
    }.as_str()).into()
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
