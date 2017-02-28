extern crate chrono;
extern crate xml as _xml;
extern crate std;

use std::str::FromStr;

use parsers;
use gpx::par::_ElementError;
use xml::ElemStart;
use ser::{ Serialize, SerializeCharElem, SerializeVia };


pub type Time = chrono::DateTime<chrono::FixedOffset>;
pub type DateTime = chrono::DateTime<chrono::FixedOffset>;
pub type NonNegativeInteger = u64;
pub type Decimal = String; // FIXME
pub type Degrees = String; // FIXME

pub fn parse_int<T: std::io::Read, Error, EFree>
        (mut parser: &mut _xml::EventReader<T>, elem_start: ElemStart)
        -> Result<NonNegativeInteger, Error>
        where Error: parsers::ElementError<Free=EFree>,
              EFree: parsers::ElementErrorFree + From<std::num::ParseIntError> {
    parsers::parse_chars(parser, elem_start,
                         |chars| NonNegativeInteger::from_str(chars))
}

pub fn parse_string<T: std::io::Read, Error, EFree>
        (mut parser: &mut _xml::EventReader<T>, elem_start: ElemStart)
        -> Result<String, Error>
        where Error: parsers::ElementError<Free=EFree>,
              EFree: parsers::ElementErrorFree + From<std::num::ParseIntError> {
    parsers::parse_chars(parser, elem_start,
                        |chars| Ok::<_, EFree>(chars.into()))
}

impl SerializeCharElem for NonNegativeInteger {
    fn to_characters(&self) -> String { self.to_string() }
}

impl SerializeCharElem for DateTime {
    fn to_characters(&self) -> String { self.to_rfc3339() }
}
