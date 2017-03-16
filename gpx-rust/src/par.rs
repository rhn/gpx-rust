//! Contains functionaity useful for implementing XML parsers
extern crate xml as _xml;

use std;
use std::io;
use std::fmt;
use std::error::Error as ErrorTrait;

use self::_xml::common::{ Position, TextPosition };
use self::_xml::reader::{ EventReader, XmlEvent };

use xml;
use xml::{ ElementParse, ElementParser, XmlElement, ElemStart };
use gpx::par::Error;
use conv;


/// Describes the position in the input stream for some data.
///
/// Used most extendively for errors.
#[derive(Debug)]
pub struct Positioned<Data> {
    pub data: Data,
    pub position: TextPosition,
}

impl<Data> Positioned<Data> {
    pub fn with_position(data: Data, position: TextPosition) -> Self {
        Positioned { data: data, position: position }
    }
}

impl<Data: fmt::Debug + fmt::Display> fmt::Display for Positioned<Data> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "Position {}: {}", self.position, self.data)
    }
}

impl<Data: ErrorTrait> ErrorTrait for Positioned<Data> {
    fn description(&self) -> &str {
        ""
    }
    fn cause(&self) -> Option<&ErrorTrait> {
        Some(&self.data)
    }
}

/// Parses complex element in XML stream into `Data` type.
///
/// The element may take any form.
/// Implement on converter types.
pub trait ParseVia<Data> {
    fn parse_via<R: io::Read>(parser: &mut EventReader<R>, elem_start: ElemStart)
        -> Result<Data, Positioned<Error>>;
}

impl ParseVia<XmlElement> for conv::XmlElement {
    fn parse_via<R: io::Read>(parser: &mut EventReader<R>, elem_start: ElemStart)
            -> Result<XmlElement, Positioned<Error>> {
        ElementParser::new(parser).parse(elem_start)
    }
}

/// Parses simple element in XML stream into `Data` type.
///
/// The element must contain only character data.
/// `ParseVia` trait is automatically defined.
pub trait ParseViaChar<Data> {
    fn from_char(s: &str) -> Result<Data, ::gpx::par::Error>;
}

/// Implements basic event loop reading character data from inside
impl<T, Data> ParseVia<Data> for T where T: ParseViaChar<Data> {
    fn parse_via<R: io::Read>(parser: &mut EventReader<R>, elem_start: ElemStart)
            -> Result<Data, Positioned<Error>> {
        let mut ret = String::new();
        loop {
            match parser.next() {
                Ok(XmlEvent::Characters(data)) => {
                    ret = data;
                }
                Ok(XmlEvent::EndElement { name }) => {
                    return if name == elem_start.name {
                        Self::from_char(&ret).map_err(|e| {
                            Positioned::with_position(e.into(), parser.position())
                        })
                    } else {
                        Err(Positioned::with_position(xml::ElementError::UnexpectedEnd.into(),
                                                      parser.position()))
                    }
                }
                Ok(XmlEvent::Whitespace(s)) => {
                    println!("{:?}", s);
                }
                Ok(ev) => {
                    return Err(Positioned::with_position(xml::ElementError::UnexpectedEvent(ev).into(),
                                                         parser.position()));
                }
                Err(error) => {
                    return Err(Positioned::with_position(error.into(), parser.position()));
                }
            }
        }
    }
}

/// Parses attribute into `Data` type.
///
/// Implement for `conv` types.
pub trait FromAttributeVia<Data> {
    fn from_attribute(&str) -> Result<Data, AttributeValueError>;
}

/// Raise whenever attribute value is out of bounds
///
/// TODO: follow this Box<> pattern to allow for carrying of namespace-specific errors
#[derive(Debug)]
pub enum AttributeValueError {
    Error(Box<std::error::Error>),
}
