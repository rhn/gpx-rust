extern crate chrono;
extern crate std;

use std::str::FromStr;

use parsers;
use ser::{ Serialize, SerializeCharElem, SerializeVia };


pub type Time = chrono::DateTime<chrono::FixedOffset>;
pub type DateTime = chrono::DateTime<chrono::FixedOffset>;
pub type NonNegativeInteger = u64;
pub type Decimal = String; // FIXME
pub type Degrees = String; // FIXME

pub mod par {
    extern crate xml as _xml;

    use std;
    use std::io;
    use std::str::FromStr;
    
    use xml::ElemStart;
    use par;
    use par::ParseVia;
    use par::parse_chars;
    use gpx::ElementError; // FIXME: move to par and concretize these types
    use xsd;
    use xsd::NonNegativeInteger;
    use gpx::par::{ _ElementError, AttributeValueError, FromAttribute }; // FIXME: move to par
    use xsd::conv;
    
    pub fn parse_int<T: std::io::Read, Error, EFree>
            (mut parser: &mut _xml::EventReader<T>, elem_start: ElemStart)
            -> Result<NonNegativeInteger, Error>
            where Error: par::ElementError<Free=EFree>,
                  EFree: par::ElementErrorFree + From<std::num::ParseIntError> {
        par::parse_chars(parser, elem_start,
                         |chars| NonNegativeInteger::from_str(chars))
    }

    pub fn parse_string<T: std::io::Read, Error, EFree>
            (mut parser: &mut _xml::EventReader<T>, elem_start: ElemStart)
            -> Result<String, Error>
            where Error: par::ElementError<Free=EFree>,
                  EFree: par::ElementErrorFree + From<std::num::ParseIntError> {
        par::parse_chars(parser, elem_start,
                         |chars| Ok::<_, EFree>(chars.into()))
    }
    
    pub fn parse_time<T: std::io::Read>
            (mut parser: &mut _xml::EventReader<T>, elem_start: ElemStart)
            -> Result<xsd::Time, ElementError> {
        parse_chars(parser, elem_start,
                    |chars| xsd::Time::parse_from_rfc3339(chars).map_err(_ElementError::from))
    }
    
    pub fn parse_decimal<T: std::io::Read>
            (mut parser: &mut _xml::EventReader<T>, elem_start: ElemStart)
            -> Result<xsd::Decimal, ElementError> {
        parse_chars(parser,
                    elem_start,
                    |chars| xsd::Decimal::from_str(chars).map_err(_ElementError::from))
    }
    
    impl ParseVia<String> for conv::String {
        fn parse_via<R: io::Read>(parser: &mut _xml::EventReader<R>, elem_start: ElemStart)
                -> Result<String, ElementError> {
            parse_chars(parser,
                        elem_start,
                        |chars| Ok::<_, _ElementError>(String::from(chars)))
        }
    }
    
    impl FromAttribute<String> for conv::String {
        fn from_attr(attr: &str) -> Result<String, AttributeValueError> {
            Ok(String::from(attr))
        }
    }
}

pub mod conv {
    pub struct String {}
}

impl SerializeCharElem for NonNegativeInteger {
    fn to_characters(&self) -> String { self.to_string() }
}

impl SerializeCharElem for DateTime {
    fn to_characters(&self) -> String { self.to_rfc3339() }
}
