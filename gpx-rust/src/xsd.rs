extern crate chrono;
extern crate xml as _xml;
extern crate std;

use std::io;
use std::str::FromStr;
use std::borrow::Cow;
use self::chrono::{ DateTime, FixedOffset };
use self::_xml::name::Name;
use self::_xml::namespace::Namespace;
use self::_xml::writer;
use self::_xml::writer::{ EventWriter, XmlEvent };

use parsers::{ parse_chars, CharNodeError };
use xml::ElemStart;
use ser::{ Serialize, SerializeCharElem };


pub type Time = DateTime<FixedOffset>;
pub type NonNegativeInteger = u64;


pub fn parse_int<T: std::io::Read, Error: CharNodeError + From<std::num::ParseIntError>>
        (mut parser: &mut _xml::EventReader<T>, elem_start: ElemStart)
        -> Result<NonNegativeInteger, Error> {
    parse_chars(parser, elem_start,
                |chars| NonNegativeInteger::from_str(chars)
    )
}

impl SerializeCharElem for Time {
    fn to_characters(&self) -> String { self.to_rfc3339() }
}
