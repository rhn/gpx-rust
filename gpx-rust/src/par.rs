extern crate xml as _xml;

use std;
use std::io;
use std::str::FromStr;

use self::_xml::name::OwnedName;
use self::_xml::common::{ Position, TextPosition };
use self::_xml::reader::{ EventReader, XmlEvent };

use xml::{ ElementParse, ElementParser, XmlElement, ElemStart };
use gpx::par::_ElementError;
use gpx::ElementError as ElementErrorE;
use conv;


pub trait ElementErrorFree where Self: From<&'static str> + From<_xml::reader::Error> {}

pub trait ElementError where Self: Sized {
    type Free: ElementErrorFree;
    // TODO: remove
    fn from_free(err: Self::Free, position: TextPosition) -> Self {
        Self::with_position(err, position)
    }
    fn with_position(err: Self::Free, position: TextPosition) -> Self;
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
        -> Result<Data, ElementErrorE>;
}

impl ParseVia<XmlElement> for conv::XmlElement {
    fn parse_via<R: io::Read>(parser: &mut EventReader<R>, elem_start: ElemStart)
            -> Result<XmlElement, ElementErrorE> {
        ElementParser::new(parser).parse(elem_start)
    }
}

pub trait FromAttributeVia<Data> {
    fn from_attribute(&str) -> Result<Data, AttributeValueError>;
}

/// Raise whenever attribute value is out of bounds
#[derive(Debug)]
pub enum AttributeValueError {
    Str(&'static str),
    Error(Box<std::error::Error>),
}

pub fn parse_chars<R: std::io::Read, F, Res, E: ElementError, EInner>
    (mut parser: &mut EventReader<R>, elem_start: ElemStart, decode: F)
    -> Result<Res, E>
        where F: Fn(&str) -> Result<Res, EInner>,
              E::Free: From<EInner> {
    let mut ret = String::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::Characters(data)) => {
                ret = data;
            }
            Ok(XmlEvent::EndElement { name }) => {
                return if name == elem_start.name {
                    decode(&ret).map_err(|e| {
                        E::from_free(e.into(), parser.position())
                    })
                } else {
                    Err(E::from_free("Unexpected end".into(), parser.position()))
                }
            }
            Ok(XmlEvent::Whitespace(s)) => {
                println!("{:?}", s);
            }
            Ok(ev) => {
                println!("{:?}", ev);
                return Err(E::from_free("Unexpected event".into(), parser.position()));
            }
            Err(error) => {
                return Err(E::from_free(error.into(), parser.position()));
            }
        }
    }
}

pub fn parse_string<T: std::io::Read> (mut parser: &mut EventReader<T>, elem_start: ElemStart)
        -> Result<String, ElementErrorE> {
    parse_chars(parser,
                elem_start,
                |chars| Ok::<_, _ElementError>(chars.into()))
}

pub fn parse_u64<T: std::io::Read> (mut parser: &mut EventReader<T>, elem_start: ElemStart)
        -> Result<u64, ElementErrorE> {
    parse_chars(parser, elem_start,
                |chars| u64::from_str(chars).map_err(_ElementError::from))
}

// unused
pub fn parse_elem<T: std::io::Read>(parser: &mut EventReader<T>, elem_start: ElemStart)
        -> Result<XmlElement, ElementErrorE> {
    ElementParser::new(parser).parse(elem_start)
}
