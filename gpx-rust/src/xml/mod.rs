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

use ::par::{ Positioned, ElementParse, ElementBuild };

pub mod conv;
pub mod par;
mod ser;

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
pub enum ElementError {
    UnexpectedEnd,
    UnexpectedEvent(XmlEvent),
}

pub type BuildError = par::BuildError;

impl From<&'static str> for Error {
    fn from(msg: &'static str) -> Error { Error::Str(msg) }
}

#[derive(Debug)]
pub enum Node {
    Text(String),
    Element(Element),
}

#[derive(Debug)]
pub struct Element {
    pub name: OwnedName,
    pub attributes: Vec<OwnedAttribute>,
    pub namespace: Namespace,
    pub nodes: Vec<Node>,
}

enum ParserState {
    PreStart,
    Inside,
    PostEnd,
}

pub type ElementParser<'a, T> = par::ElementParser<'a, T>;

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
struct ParserData(Vec<Node>);

impl DocumentParserData for ParserData {
    type Contents = Vec<Node>;
    type Error = DocumentError;
    fn parse_element<R: Read>(&mut self, mut reader: &mut EventReader<R>,
                              name: &OwnedName, attributes: &[OwnedAttribute])
            -> Result<(), DataError> {
        let elem = try!(ElementParser::new(&mut reader).parse(name, attributes));
        self.0.push(Node::Element(elem));
        Ok(())
    }
    fn build(self) -> Result<Self::Contents, Self::Error> {
        Ok(self.0)
    }
}

pub fn parse<R: Read>(source: R) -> Result<Document<Vec<Node>>, DocumentError> {
    parse_document::<R, ParserData>(source)
}
