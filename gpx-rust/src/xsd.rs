extern crate chrono;
extern crate xml as _xml;
extern crate std;

use std::str::FromStr;

use parsers::{ parse_chars, ElementError };
use parsers::ElementErrorFree;
use gpx::par::_ElementError;
use xml::ElemStart;
use ser::{ Serialize, SerializeCharElem };


pub type Time = chrono::DateTime<chrono::FixedOffset>;
pub type DateTime = chrono::DateTime<chrono::FixedOffset>;
pub type NonNegativeInteger = u64;
pub type Decimal = String; // FIXME
pub type Degrees = String; // FIXME

pub fn parse_int<T: std::io::Read, Error, EFree>
        (mut parser: &mut _xml::EventReader<T>, elem_start: ElemStart)
        -> Result<NonNegativeInteger, Error>
        where Error: ElementError<Free=EFree>,
              EFree: ElementErrorFree + From<std::num::ParseIntError> {
    parse_chars(parser, elem_start,
                |chars| NonNegativeInteger::from_str(chars)
    )
}

impl SerializeCharElem for Time {
    fn to_characters(&self) -> String { self.to_rfc3339() }
}

impl SerializeCharElem for NonNegativeInteger {
    fn to_characters(&self) -> String { self.to_string() }
}
