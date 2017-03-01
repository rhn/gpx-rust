extern crate xml as _xml;

use std;
use std::str::FromStr;

use self::_xml::name::OwnedName;
use self::_xml::common::{ Position, TextPosition };
use self::_xml::reader::{ EventReader, XmlEvent };

use xml::{ ElementParse, ElementParser, XmlElement, ElemStart };
use xsd;
use gpx::par::_ElementError;
use gpx::ElementError as ElementErrorE;


pub trait ElementErrorFree where Self: From<&'static str> + From<_xml::reader::Error> {}

pub trait ElementError {
    type Free: ElementErrorFree;
    fn from_free(err: Self::Free, position: TextPosition) -> Self;
}

/// Error classes in ElementParser must implement this
pub trait ParserMessage
        where Self: From<&'static str> {
    fn from_unexp_attr(elem_name: OwnedName, attr_name: OwnedName) -> Self;
    fn from_xml_error(_xml::reader::Error) -> Self;
    fn from_bad_attr_val(::gpx::par::AttributeValueError) -> Self;
}

pub fn parse_chars<T: std::io::Read, F, Res, E: ElementError, EInner>
    (mut parser: &mut EventReader<T>, elem_start: ElemStart, decode: F)
    -> Result<Res, E>
        where F: Fn(&str) -> Result<Res, EInner>,
              E::Free: From<EInner> {
    let mut ret = None;
    loop {
        match parser.next() {
            Ok(XmlEvent::Characters(data)) => {
                ret = Some(try!(decode(&data).map_err(|e| {
                    E::from_free(e.into(), parser.position())
                })));
            }
            Ok(XmlEvent::EndElement { name }) => {
                if name == elem_start.name {
                    return match ret {
                        Some(c) => Ok(c),
                        None => Err(E::from_free("Missing data".into(), parser.position()))
                    }
                }
                return Err(E::from_free("Unexpected end".into(), parser.position()));
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

pub fn parse_elem<T: std::io::Read>(parser: &mut EventReader<T>, elem_start: ElemStart)
        -> Result<XmlElement, ElementErrorE> {
    ElementParser::new(parser).parse_self(elem_start)//.map_err(ElementErrorE::from)
}
