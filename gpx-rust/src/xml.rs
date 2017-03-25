/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! General parsing of XML documents

extern crate xml;

use std::io;
use std::io::Read;
use std::error::Error as ErrorTrait;

use self::xml::name::OwnedName;
use self::xml::attribute::OwnedAttribute;
use self::xml::namespace::Namespace;
use self::xml::reader::{ EventReader, XmlEvent };
use self::xml::common::{ XmlVersion, TextPosition, Position };

use par;
use par::Positioned;

type DataError = Positioned<::gpx::par::Error>;


#[derive(Debug)]
pub enum DocumentError {
    ParserError(xml::reader::Error),
    DocumentParserError(DocumentParserError),
    BadData(DataError)
}

impl From<xml::reader::Error> for DocumentError {
    fn from(err: xml::reader::Error) -> DocumentError {
        DocumentError::ParserError(err)
    }
}

impl From<DataError> for DocumentError {
    fn from(err: DataError) -> DocumentError {
        DocumentError::BadData(err)
    }
}

impl From<DocumentParserError> for DocumentError {
    fn from(err: DocumentParserError) -> DocumentError {
        DocumentError::DocumentParserError(err)
    }
}

#[derive(Debug)]
pub enum DocumentParserError {
    UnexpectedEventPreStart(XmlEvent),
    UnexpectedEventInside(XmlEvent)
}

#[derive(Debug)]
pub enum Error {
    Str(&'static str),
    Io(io::Error),
    Xml(xml::reader::Error),
}

#[derive(Debug)]
pub enum BuildError {
    Custom(Box<ErrorTrait>)
}

#[derive(Debug)]
pub enum ElementError {
    UnexpectedEnd,
    UnexpectedEvent(XmlEvent),
}

impl From<&'static str> for Error {
    fn from(msg: &'static str) -> Error { Error::Str(msg) }
}

#[derive(Debug)]
pub enum XmlNode {
    Text(String),
    Element(XmlElement),
}

#[derive(Debug)]
pub struct XmlElement {
    pub name: OwnedName,
    pub attributes: Vec<OwnedAttribute>,
    pub namespace: Namespace,
    pub nodes: Vec<XmlNode>,
}

enum ParserState {
    PreStart,
    Inside,
    PostEnd,
}

pub struct ElementParser<'a, T: 'a + Read> {
    reader: &'a mut EventReader<T>,
    name: Option<OwnedName>, // Using reference intentionally - this code does not need to interact with Name
    attributes: Vec<OwnedAttribute>,
    nodes: Vec<XmlNode>,
}

pub trait ElementBuild {
    type Element;
    type BuildError;
    fn build(self) -> Result<Self::Element, Self::BuildError>;
}

pub trait ElementParse<'a, R: Read, E>
    where Self: Sized + ElementBuild,
          E: From<ElementError> + From<Self::BuildError> + From<par::AttributeError<E>>
             + From<xml::reader::Error> + par::FormatError {
    // public iface
    fn new(reader: &'a mut EventReader<R>) -> Self;
    
    /// Parses the element and its subelements, returning ElementBuild::Element instance.
    fn parse(mut self, name: &OwnedName, attributes: &[OwnedAttribute])
            -> Result<Self::Element, Positioned<E>> {
        try!(self.parse_start(name, attributes).map_err(|e| self._with_pos(e)));
        loop {
            match try!(self.next().map_err(|e| self._with_pos(e))) {
                XmlEvent::StartElement { name, attributes, namespace: _ } => {
                    try!(self.parse_element(&name, attributes.as_slice()));
                }
                XmlEvent::EndElement { name } => {
                    if &name == self.get_name() {
                        break;
                    }
                    return Err(self._with_pos(ElementError::UnexpectedEnd));
                }
                XmlEvent::Characters(data) => {
                    try!(self.parse_characters(data).map_err(|e| self._with_pos(e)));
                }
                XmlEvent::Whitespace(s) => {
                    try!(self.parse_whitespace(s).map_err(|e| self._with_pos(e)));
                }
                e => return Err(self._with_pos(ElementError::UnexpectedEvent(e)))
            }
        }
        let pos = self.get_parser_position();
        self.build().map_err(|e| Positioned::with_position(e.into(), pos))
    }
    /// Helper, converts any parse error to the positioned error type. Not a closure to hopefully save performance
    fn _with_pos<Kind: Into<E>>(&self, kind: Kind) -> Positioned<E> {
        Positioned::with_position(kind.into(), self.get_parser_position())
    }
    /// Helper, equivalent to self.reader.position()
    fn get_parser_position(&self) -> TextPosition;
    /// Parses the start event and attributes within it. Should be implemented, bu default ignores attributes.
    fn parse_start(&mut self, name: &OwnedName, attributes: &[OwnedAttribute])
            -> Result<(), par::AttributeError<E>> {
        let _ = (name, attributes);
        Ok(())
    }
    /// Parses sub-element.
    fn parse_element(&mut self, name: &OwnedName, attributes: &[OwnedAttribute])
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
    /// Returns next event from the underlying parser.
    fn next(&mut self) -> Result<XmlEvent, xml::reader::Error>;
}

impl<'a, T: Read> ElementBuild for ElementParser<'a, T> {
    type Element = XmlElement;
    type BuildError = BuildError;
    fn build(self) -> Result<XmlElement, Self::BuildError> {
        Ok(XmlElement {
            name: self.name.unwrap().to_owned(),
            attributes: self.attributes,
            namespace: Namespace::empty(),
            nodes: self.nodes
        })
    }
}

impl<'a, T: Read> ElementParse<'a, T, ::gpx::par::Error> for ElementParser<'a, T> {
    fn new(reader: &'a mut EventReader<T>)
            -> ElementParser<'a, T> {
        ElementParser { reader: reader,
                        name: None,
                        attributes: Vec::new(),
                        nodes: Vec::new() }
    }
    fn parse_start(&mut self, name: &OwnedName, attributes: &[OwnedAttribute])
            -> Result<(), par::AttributeError<::gpx::par::Error>> {
        self.name = Some(name.clone());
        let _ = attributes; // FIXME: break if attributes present
        Ok(())
    }
    fn parse_element(&mut self, name: &OwnedName, attributes: &[OwnedAttribute])
            -> Result<(), Positioned<::gpx::par::Error>> {
        let elem = try!(ElementParser::new(self.reader).parse(name, attributes));
        self.nodes.push(XmlNode::Element(elem));
        Ok(())
    }
    fn parse_characters(&mut self, data: String) -> Result<(), ::gpx::par::Error> {
        self.nodes.push(XmlNode::Text(data));
        Ok(())
    }
    fn get_parser_position(&self) -> TextPosition {
        self.reader.position()
    }
    fn get_name(&self) -> &OwnedName {
        match &self.name {
            &Some(ref i) => i,
            &None => unreachable!(),
        }
    }
    fn next(&mut self) -> Result<XmlEvent, xml::reader::Error> {
        self.reader.next()
    }
}

/// Represents an XML document, including metadata
pub struct Document<T> {
    pub info: DocInfo,
    pub data: T,
}

/// Document metadata
pub struct DocInfo {
    pub version: XmlVersion,
    pub encoding: String,
    pub standalone: Option<bool>,
}

pub fn parse_document<R: Read, D: DocumentParserData>(source: R)
        -> Result<Document<D::Contents>, D::Error> {
    let mut reader = EventReader::new(source);
    let mut info = None;
    let mut contents = D::default();
    let mut state = ParserState::PreStart;
    loop {
        let next = try!(reader.next());
        state = match state {
            ParserState::PreStart => match next {
                XmlEvent::StartDocument { version, encoding, standalone } => {
                    info = Some(DocInfo { version: version,
                                          encoding: encoding,
                                          standalone: standalone });
                    ParserState::Inside
                }
                ev => return Err(DocumentParserError::UnexpectedEventPreStart(ev).into())
            },
            ParserState::Inside => match next {
                XmlEvent::StartElement { name, attributes, namespace: _ } => {
                    try!(contents.parse_element(&mut reader, &name, &attributes));
                    ParserState::Inside
                }
                // TODO: more events
                XmlEvent::EndDocument => ParserState::PostEnd,
                ev => return Err(DocumentParserError::UnexpectedEventInside(ev).into())
            },
            ParserState::PostEnd => { break; }
        }
    }
    Ok(Document {
        info: info.unwrap(),
        data: try!(contents.build())
    })
}

pub trait DocumentParserData where Self: Sized + Default {
    type Contents;
    type Error: From<xml::reader::Error> + From<DocumentParserError> + From<DataError>;
    // public iface
    fn parse_element<R: Read>(&mut self, reader: &mut EventReader<R>,
                              name: &OwnedName, attributes: &[OwnedAttribute])
            -> Result<(), DataError>;
    fn build(self) -> Result<Self::Contents, Self::Error>;
}

#[derive(Default)]
struct ParserData(Vec<XmlNode>);

impl DocumentParserData for ParserData {
    type Contents = Vec<XmlNode>;
    type Error = DocumentError;
    fn parse_element<R: Read>(&mut self, mut reader: &mut EventReader<R>,
                              name: &OwnedName, attributes: &[OwnedAttribute])
            -> Result<(), DataError> {
        let elem = try!(ElementParser::new(&mut reader).parse(name, attributes));
        self.0.push(XmlNode::Element(elem));
        Ok(())
    }
    fn build(self) -> Result<Self::Contents, Self::Error> {
        Ok(self.0)
    }
}

pub fn parse<R: Read>(source: R) -> Result<Document<Vec<XmlNode>>, DocumentError> {
    parse_document::<R, ParserData>(source)
}
