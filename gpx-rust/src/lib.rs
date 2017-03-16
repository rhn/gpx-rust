#[macro_use]
extern crate macro_attr;
#[macro_use]
extern crate gpx_debug;

#[macro_use]
mod parsers;

pub mod xml;
pub mod xsd;
pub mod gpx;
pub mod par;
pub mod ser;
mod conv;
