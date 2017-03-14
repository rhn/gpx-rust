extern crate xml as _xml;

use std;
use std::io;
use std::str::FromStr;
use std::fmt;
use std::error::Error as ErrorTrait;

use self::_xml::name::OwnedName;
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

pub trait FromAttribute<T> {
    fn from_attr(&str) -> Result<T, AttributeValueError>;
}


/// Error classes in ElementParser must implement this
pub trait ParserMessage
        where Self: From<&'static str> {
    fn from_unexp_attr(elem_name: OwnedName, attr_name: OwnedName) -> Self;
    fn from_xml_error(_xml::reader::Error) -> Self;
    fn from_bad_attr_val(AttributeValueError) -> Self;
}

/// Implement on converters to do Par::parse_via(data, ...)
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

pub trait ParseViaChar<Data> {
    fn from_char(s: &str) -> Result<Data, ::gpx::par::Error>;
}

impl<T, Data> ParseVia<Data> for T where T: ParseViaChar<Data> {
    fn parse_via<R: io::Read>(parser: &mut EventReader<R>, elem_start: ElemStart)
            -> Result<Data, Positioned<Error>> {
        parse_chars(parser, elem_start, |s| Self::from_char(s))
    }
}

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

pub fn parse_chars<R: std::io::Read, F, Res, E, EInner>
    (mut parser: &mut EventReader<R>, elem_start: ElemStart, decode: F)
    -> Result<Res, Positioned<E>>
        where F: Fn(&str) -> Result<Res, EInner>,
              E: From<xml::ElementError> + From<EInner> + From<_xml::reader::Error> {
    let mut ret = String::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::Characters(data)) => {
                ret = data;
            }
            Ok(XmlEvent::EndElement { name }) => {
                return if name == elem_start.name {
                    decode(&ret).map_err(|e| {
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

pub fn parse_string<T: std::io::Read> (mut parser: &mut EventReader<T>, elem_start: ElemStart)
        -> Result<String, Positioned<Error>> {
    parse_chars(parser,
                elem_start,
                |chars| Ok::<_, Error>(chars.into()))
}

pub fn parse_u64<T: std::io::Read> (mut parser: &mut EventReader<T>, elem_start: ElemStart)
        -> Result<u64, Positioned<Error>> {
    parse_chars(parser, elem_start,
                |chars| u64::from_str(chars).map_err(Error::from))
}

// unused
pub fn parse_elem<T: std::io::Read>(parser: &mut EventReader<T>, elem_start: ElemStart)
        -> Result<XmlElement, Positioned<Error>> {
    ElementParser::new(parser).parse(elem_start)
}
