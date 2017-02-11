extern crate chrono;
extern crate xml as xml_;
extern crate std;

use std::str::FromStr;
use self::chrono::{ DateTime, FixedOffset };

use parsers::{ parse_chars, CharNodeError };
use xml::ElemStart;

pub type Time = DateTime<FixedOffset>;
pub type NonNegativeInteger = u64;


pub fn parse_int<T: std::io::Read, Error: CharNodeError + From<std::num::ParseIntError>>
        (mut parser: &mut xml_::EventReader<T>, elem_start: ElemStart)
        -> Result<NonNegativeInteger, Error> {
    parse_chars(parser, elem_start,
                |chars| NonNegativeInteger::from_str(chars)
    )
}
