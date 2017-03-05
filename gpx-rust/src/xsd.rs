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
    
    impl FromAttribute<xsd::Uri> for conv::Uri {
        fn from_attr(attr: &str) -> Result<xsd::Uri, AttributeValueError> {
            Ok(xsd::Uri::from(attr))
        }
    }
}

pub mod conv {
    //! conversion markers
    use std;
    pub type String = std::string::String;
    pub type Decimal = f64;
    pub struct Uri {}
}

mod ser {
    //! Serialization impls
    use xsd;
    use xsd::conv;
    use ser::SerializeCharElem;
    
    use gpx::ser::{ ToAttributeVia, AttributeValueError };
    
    impl SerializeCharElem for xsd::NonNegativeInteger {
        fn to_characters(&self) -> String { self.to_string() }
    }
    
    impl SerializeCharElem for xsd::Decimal {
        fn to_characters(&self) -> String { self.to_string() }
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
