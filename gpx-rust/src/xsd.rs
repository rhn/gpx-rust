//! Types defined in XSD spec.
//! Number types are not converted precisely to save on complexity and speed.
//! XSD defines numbers to have arbitrary precision and to save trailing zeroes, which is not required for basic purposes.

extern crate chrono;
extern crate std;

pub type Time = chrono::DateTime<chrono::FixedOffset>;
pub type DateTime = chrono::DateTime<chrono::FixedOffset>;

pub type NonNegativeInteger = u64;
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
    
    impl FromAttributeVia<xsd::Uri> for conv::Uri {
        fn from_attribute(attr: &str) -> Result<xsd::Uri, AttributeValueError> {
            Ok(xsd::Uri::from(attr))
        }
    }
}

pub mod conv {
    //! conversion markers
    use std;
    pub type String = std::string::String;
    pub struct Decimal {}
    pub struct Uri {}
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

    impl SerializeCharElem for xsd::DateTime {
        fn to_characters(&self) -> String { self.to_rfc3339() }
    }
    
    impl ToAttributeVia<xsd::Uri> for conv::Uri {
        fn to_attribute(data: &xsd::Uri) -> Result<String, AttributeValueError> {
            Ok(data.to_string())
        }
    }
}
