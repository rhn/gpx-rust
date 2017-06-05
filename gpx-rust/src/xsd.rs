/* This program and the accompanying materials are made available under the
 * terms of the Eclipse Public License v1.0 and the GNU General Public License
 * v3.0 or later which accompanies this distribution.
 * 
 *      The Eclipse Public License (EPL) v1.0 is available at
 *      http://www.eclipse.org/legal/epl-v10.html
 * 
 *      You should have received a copy of the GNU General Public License
 *      along with this program.  If not, see <http://www.gnu.org/licenses/>.
 * 
 * You may elect to redistribute this code under either of these licenses.     
 */

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
    
    impl FormatError for Error {}

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
        type Error = Error;
        fn from_attribute(attr: &str) -> Result<String, Self::Error> {
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

pub mod ser {
    //! Serialization impls
    use std;

    use xsd;
    use xsd::conv;
    use ser::ToCharsVia;
    use ser::FormatError;

    #[derive(Debug)]
    pub enum Error {}

    impl FormatError for Error {}
    
    type Result = std::result::Result<String, Error>;

    impl ToCharsVia<f64> for xsd::conv::Decimal {
        type Error = Error;
        fn to_characters(data: &f64) -> Result { Ok(data.to_string()) }
    }
    
    impl ToCharsVia<f32> for xsd::conv::Decimal {
        type Error = Error;
        fn to_characters(data: &f32) -> Result { Ok(data.to_string()) }
    }

    impl ToCharsVia<u64> for xsd::conv::Integer {
        type Error = Error;
        fn to_characters(data: &u64) -> Result { Ok(data.to_string()) }
    }
    
    impl ToCharsVia<u16> for xsd::conv::Integer {
        type Error = Error;
        fn to_characters(data: &u16) -> Result { Ok(data.to_string()) }
    }

    impl ToCharsVia<i16> for xsd::conv::Integer {
        type Error = Error;
        fn to_characters(data: &i16) -> Result { Ok(data.to_string()) }
    }

    impl ToCharsVia<xsd::DateTime> for xsd::conv::DateTime {
        type Error = Error;
        fn to_characters(data: &xsd::DateTime) -> Result { Ok(data.to_rfc3339()) }
    }

    impl ToCharsVia<String> for conv::String {
        type Error = Error;
        fn to_characters(data: &String) -> Result { Ok(data.clone()) }
    }

    impl ToCharsVia<str> for conv::String {
        type Error = Error;
        fn to_characters(data: &str) -> Result { Ok(data.into()) }
    }
}
