#[macro_use]
extern crate quote;

mod xsd_types;

pub mod gpx;


use std::collections::HashMap;

use xsd_types::XsdType;


pub type TagMap<'a> = HashMap<&'a str, &'a str>;

pub struct StructInfo<'a> {
    pub name: String,
    pub type_: &'a XsdType<'a>,
    pub tags: TagMap<'a>,
}

pub trait ParserGen {
    fn header() -> &'static str;
    fn serializer_impl(cls_name: &str, tags: &TagMap, data: &XsdType) -> String;
}
