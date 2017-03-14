//! Types defined in XSD spec.
//! Number types are not converted precisely to save on complexity and speed.
//! XSD defines numbers to have arbitrary precision and to save trailing zeroes, which is not required for basic purposes.

extern crate chrono;
extern crate std;

pub type Time = chrono::DateTime<chrono::FixedOffset>;
pub type DateTime = chrono::DateTime<chrono::FixedOffset>;

pub type NonNegativeInteger = u64;
pub type Integer = i64;
pub type GYear = i16;
pub type Decimal = f64;
pub type Uri = String;


pub mod par {
    //! Parsing impls
    extern crate xml as _xml;

    use std;
    use std::io;
    use std::str::FromStr;
    
    use xml;
    use xml::ElemStart;
    use par;
    use par::{ FromAttributeVia, ParseVia, ParseViaChar };
    use par::parse_chars;
    use par::{ Positioned, AttributeValueError };
    use xsd;
    use xsd::NonNegativeInteger;
    use gpx::par::Error; // FIXME: move to par
    use xsd::conv;
    
    // todo: move to par
    pub fn parse_int<T: std::io::Read, Error>
            (mut parser: &mut _xml::EventReader<T>, elem_start: ElemStart)
            -> Result<NonNegativeInteger, Positioned<Error>>
            where Error: From<std::num::ParseIntError> + From<xml::ElementError>
                         + From<_xml::reader::Error>{
        par::parse_chars(parser, elem_start,
                         |chars| NonNegativeInteger::from_str(chars))
    }

    // TODO: move to par
    pub fn parse_string<T: std::io::Read, Error>
            (mut parser: &mut _xml::EventReader<T>, elem_start: ElemStart)
            -> Result<String, Positioned<Error>>
            where Error: From<xml::ElementError> + From<_xml::reader::Error> {
        par::parse_chars(parser, elem_start,
                         |chars| Ok::<_, xml::ElementError>(chars.into()))
    }
    
    pub fn parse_time<T: std::io::Read>
            (mut parser: &mut _xml::EventReader<T>, elem_start: ElemStart)
            -> Result<xsd::Time, Positioned<Error>> {
        parse_chars(parser, elem_start,
                    |chars| xsd::Time::parse_from_rfc3339(chars).map_err(Error::from))
    }
    
    impl ParseVia<String> for conv::String {
        fn parse_via<R: io::Read>(parser: &mut _xml::EventReader<R>, elem_start: ElemStart)
                -> Result<String, Positioned<Error>> {
            parse_chars(parser,
                        elem_start,
                        |chars| Ok::<_, Error>(String::from(chars)))
        }
    }
    
    impl ParseViaChar<u16> for conv::Integer {
        fn from_char(s: &str) -> Result<u16, ::gpx::par::Error> {
            u16::from_str(s).map_err(::gpx::par::Error::from)
        }
    }
    
    impl ParseViaChar<u64> for conv::Integer {
        fn from_char(s: &str) -> Result<u64, ::gpx::par::Error> {
            u64::from_str(s).map_err(::gpx::par::Error::from)
        }
    }
    
    impl ParseViaChar<i16> for conv::Integer {
        fn from_char(s: &str) -> Result<i16, ::gpx::par::Error> {
            i16::from_str(s).map_err(::gpx::par::Error::from)
        }
    }
    
    impl ParseViaChar<f64> for conv::Decimal {
        fn from_char(s: &str) -> Result<f64, ::gpx::par::Error> {
            f64::from_str(s).map_err(::gpx::par::Error::from)
        }
    }
    
    impl ParseViaChar<f32> for conv::Decimal {
        fn from_char(s: &str) -> Result<f32, ::gpx::par::Error> {
            f32::from_str(s).map_err(::gpx::par::Error::from)
        }
    }
    
    impl FromAttributeVia<String> for conv::String {
        fn from_attribute(attr: &str) -> Result<String, AttributeValueError> {
            Ok(String::from(attr))
        }
    }
}

pub mod conv {
    //! conversion markers
    use std;
    pub struct String {}
    pub struct Decimal {}
    pub type Uri = String;
    pub struct Integer {}
    pub type NonNegativeInteger = Integer; // FIXME
    pub type GYear = Integer;
}

mod ser {
    //! Serialization impls
    use xsd;
    use xsd::conv;
    use ser::{ ToAttributeVia, SerializeCharElem, SerializeCharElemVia };
    
    use gpx::ser::AttributeValueError;
    
    impl SerializeCharElem for xsd::NonNegativeInteger {
        fn to_characters(&self) -> String { self.to_string() }
    }
    
    impl SerializeCharElemVia<f64> for xsd::conv::Decimal {
        fn to_characters(data: &f64) -> String { data.to_string() }
    }
    
    impl SerializeCharElemVia<f32> for xsd::conv::Decimal {
        fn to_characters(data: &f32) -> String { data.to_string() }
    }

    impl SerializeCharElemVia<u64> for xsd::conv::Integer {
        fn to_characters(data: &u64) -> String { data.to_string() }
    }

    impl SerializeCharElemVia<i16> for xsd::conv::Integer {
        fn to_characters(data: &i16) -> String { data.to_string() }
    }

    impl SerializeCharElem for xsd::DateTime {
        fn to_characters(&self) -> String { self.to_rfc3339() }
    }

    impl SerializeCharElemVia<String> for conv::String {
        fn to_characters(data: &String) -> String { data.clone() }
    }

    impl ToAttributeVia<String> for conv::String {
        fn to_attribute(data: &String) -> Result<String, AttributeValueError> {
            Ok(data.to_string())
        }
    }
}
