#[macro_use]
extern crate macro_attr;
#[macro_use]
extern crate gpx_debug;

#[macro_use]
pub mod parsers;

pub mod xml;
pub mod xsd;
pub mod gpx;
mod generator;
pub mod ser;
