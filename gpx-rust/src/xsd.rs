/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

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
    extern crate chrono;
    
    use std::str::FromStr;
    use std::num::ParseIntError;
    use std::num::ParseFloatError;
    
    use par::{ FromAttributeVia, ParseViaChar };
    use par::FormatError;
    use xsd;
    use xsd::conv;
    
    #[derive(Debug)]
    pub enum Error {
        BadInt(ParseIntError),
        BadFloat(ParseFloatError),
        BadTime(chrono::ParseError),
    }
    
    impl From<ParseIntError> for Error {
        fn from(err: ParseIntError) -> Error {
            Error::BadInt(err)
        }
    }
    
    impl From<ParseFloatError> for Error {
        fn from(err: ParseFloatError) -> Error {
            Error::BadFloat(err)
        }
    }
    
    impl ParseViaChar<String> for conv::String {
        fn from_char(chars: &str) -> Result<String, ::gpx::par::Error> {
            Ok(String::from(chars))
        }
    }
    
    impl ParseViaChar<u16> for conv::Integer {
        fn from_char(s: &str) -> Result<u16, ::gpx::par::Error> {
            u16::from_str(s).map_err(|e| Error::from(e).into())
        }
    }
    
    impl ParseViaChar<u64> for conv::Integer {
        fn from_char(s: &str) -> Result<u64, ::gpx::par::Error> {
            u64::from_str(s).map_err(|e| Error::from(e).into())
        }
    }
    
    impl ParseViaChar<i16> for conv::Integer {
        fn from_char(s: &str) -> Result<i16, ::gpx::par::Error> {
            i16::from_str(s).map_err(|e| Error::from(e).into())
        }
    }
    
    impl ParseViaChar<f64> for conv::Decimal {
        fn from_char(s: &str) -> Result<f64, ::gpx::par::Error> {
            f64::from_str(s).map_err(|e| Error::BadFloat(e).into())
        }
    }
    
    impl ParseViaChar<f32> for conv::Decimal {
        fn from_char(s: &str) -> Result<f32, ::gpx::par::Error> {
            f32::from_str(s).map_err(|e| Error::BadFloat(e).into())
        }
    }
    
    impl ParseViaChar<xsd::DateTime> for conv::DateTime {
        fn from_char(chars: &str) -> Result<xsd::DateTime, ::gpx::par::Error> {
            xsd::DateTime::parse_from_rfc3339(chars).map_err(|e| Error::BadTime(e).into())
        }
    }
    
    impl FromAttributeVia<String> for conv::String {
        fn from_attribute(attr: &str) -> Result<String, Box<FormatError>> {
            Ok(String::from(attr))
        }
    }
}

pub mod conv {
    //! conversion markers
    pub struct String {}
    pub struct Decimal {}
    pub type Uri = String;
    pub struct Integer {}
    pub type NonNegativeInteger = Integer; // FIXME
    pub type GYear = Integer;
    pub struct DateTime {}
}

mod ser {
    //! Serialization impls
    use xsd;
    use xsd::conv;
    use ser::{ ToAttributeVia, SerializeCharElemVia };
    use ser::FormatError;

    impl SerializeCharElemVia<f64> for xsd::conv::Decimal {
        fn to_characters(data: &f64) -> String { data.to_string() }
    }
    
    impl SerializeCharElemVia<f32> for xsd::conv::Decimal {
        fn to_characters(data: &f32) -> String { data.to_string() }
    }

    impl SerializeCharElemVia<u64> for xsd::conv::Integer {
        fn to_characters(data: &u64) -> String { data.to_string() }
    }
    
    impl SerializeCharElemVia<u16> for xsd::conv::Integer {
        fn to_characters(data: &u16) -> String { data.to_string() }
    }

    impl SerializeCharElemVia<i16> for xsd::conv::Integer {
        fn to_characters(data: &i16) -> String { data.to_string() }
    }

    impl SerializeCharElemVia<xsd::DateTime> for xsd::conv::DateTime {
        fn to_characters(data: &xsd::DateTime) -> String { data.to_rfc3339() }
    }

    impl SerializeCharElemVia<String> for conv::String {
        fn to_characters(data: &String) -> String { data.clone() }
    }

    impl SerializeCharElemVia<str> for conv::String {
        fn to_characters(data: &str) -> String { data.into() }
    }

    impl ToAttributeVia<String> for conv::String {
        fn to_attribute(data: &String) -> Result<String, Box<FormatError>> {
            Ok(data.to_string())
        }
    }
}
