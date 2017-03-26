/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Contains functionaity useful for implementing XML parsers
extern crate xml as _xml;

use std::io;
use std::io::Read;
use std::fmt;
use std::error::Error as ErrorTrait;

use self::_xml::common::{ Position, TextPosition };
use self::_xml::reader::{ EventReader, XmlEvent };
use self::_xml::name::OwnedName;
use self::_xml::attribute::OwnedAttribute;

use xml;
use gpx::par::Error;

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

/// Problem with attributes serialization
#[derive(Debug)]
pub enum AttributeError<T: FormatError> {
    InvalidValue(T), // TODO: include attribute name
    /// This name is not allowed here
    Unexpected(OwnedName),
    // missing should be in build error, to give flexibility for fancy constraints
}

impl<T: FormatError> From<T> for AttributeError<T> {
    fn from(err: T) -> AttributeError<T> {
        AttributeError::InvalidValue(err)
    }
}

/// A string value cannot be parsed
///
/// Marks that this error can be used in AttributeError
pub trait FormatError where Self: fmt::Debug {} // TODO: enforce Error


/// Can parse complex element in XML stream into `Data` type.
///
/// The element may take any form.
/// Implement on converter types.
pub trait ParseVia<Data> {
    fn parse_via<R: io::Read>(parser: &mut EventReader<R>,
                              name: &OwnedName, attributes: &[OwnedAttribute])
        -> Result<Data, Positioned<Error>>;
}

/// Can parse simple element in XML stream into `Data` type.
///
/// The element must contain only character data.
/// `ParseVia` trait is automatically defined.
pub trait ParseViaChar<Data> {
    fn from_char(s: &str) -> Result<Data, ::gpx::par::Error>;
}

/// Implements basic event loop reading character data from inside
impl<T, Data> ParseVia<Data> for T where T: ParseViaChar<Data> {
    fn parse_via<R: io::Read>(parser: &mut EventReader<R>, 
                              end_name: &OwnedName, attributes: &[OwnedAttribute])
            -> Result<Data, Positioned<Error>> {
        let _ = attributes; // FIXME: error on present attributes
        let mut ret = String::new();
        loop {
            match parser.next() {
                Ok(XmlEvent::Characters(data)) => {
                    ret = data;
                }
                Ok(XmlEvent::EndElement { name }) => {
                    return if &name == end_name {
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

/// Helper, converts any parse error to the positioned error type. Not a closure to hopefully save performance
fn _with_pos<R: Read, Dst, Src: Into<Dst>>(reader: &EventReader<R>, src: Src) -> Positioned<Dst> {
    Positioned::with_position(src.into(), reader.position())
}

/// Can parse attribute into `Data` type.
///
/// Implement for `conv` types.
pub trait FromAttributeVia<Data> {
    type Error: FormatError;
    fn from_attribute(&str) -> Result<Data, Self::Error>;
}

pub trait ElementParse<E>
    where Self: Sized + ElementBuild,
          E: From<xml::ElementError> + From<Self::BuildError> + From<::par::AttributeError<E>>
             + From<_xml::reader::Error> + ::par::FormatError {
    // public iface
    fn new() -> Self;
    
    /// Parses the element and its subelements, returning ElementBuild::Element instance.
    fn parse<'a, R: Read>(mut self, name: &OwnedName, attributes: &[OwnedAttribute],
                          mut reader: &'a mut EventReader<R>)
            -> Result<Self::Element, Positioned<E>> {
        try!(self.parse_start(name, attributes).map_err(|e| _with_pos(reader, e)));
        loop {
            match try!(reader.next().map_err(|e| _with_pos(reader, e))) {
                XmlEvent::StartElement { name, attributes, namespace: _ } => {
                    try!(self.parse_element(&mut reader, &name, attributes.as_slice()));
                }
                XmlEvent::EndElement { name } => {
                    if &name == self.get_name() {
                        break;
                    }
                    return Err(_with_pos(reader, xml::ElementError::UnexpectedEnd));
                }
                XmlEvent::Characters(data) => {
                    try!(self.parse_characters(data).map_err(|e| _with_pos(reader, e)));
                }
                XmlEvent::Whitespace(s) => {
                    try!(self.parse_whitespace(s).map_err(|e| _with_pos(reader, e)));
                }
                e => return Err(_with_pos(reader, xml::ElementError::UnexpectedEvent(e)))
            }
        }
        self.build().map_err(|e| _with_pos(reader, e))
    }
    /// Parses the start event and attributes within it. Should be implemented, bu default ignores attributes.
    fn parse_start(&mut self, name: &OwnedName, attributes: &[OwnedAttribute])
            -> Result<(), ::par::AttributeError<E>> {
        let _ = (name, attributes);
        Ok(())
    }
    /// Parses sub-element.
    fn parse_element<'a, R: Read>(&mut self, reader: &'a mut EventReader<R>,
                                  name: &OwnedName, attributes: &[OwnedAttribute])
        -> Result<(), Positioned<E>>;
    /// Parses characters. By default ignores.
    fn parse_characters(&mut self, data: String) -> Result<(), E> {
        let _ = data;
        Ok(())
    }
    /// Parses whitespace (as defined by xml-rs Whitespace event). By Default ignores.
    fn parse_whitespace(&mut self, space: String)
            -> Result<(), E> {
        let _ = space;
        Ok(())
    }
    /// Return the name of this element.
    fn get_name(&self) -> &OwnedName;
}

pub trait ElementBuild {
    type Element;
    type BuildError;
    fn build(self) -> Result<Self::Element, Self::BuildError>;
}
