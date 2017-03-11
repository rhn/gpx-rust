extern crate xml;

use std::io;
use std::io::Read;
use self::xml::name::OwnedName;
use self::xml::attribute::OwnedAttribute;
use self::xml::namespace::Namespace;
use self::xml::reader::{ EventReader, XmlEvent };
use self::xml::common::{ XmlVersion, TextPosition, Position };

use par::PositionedError;

#[derive(Debug)]
pub enum DocumentError {
    ParserError(xml::reader::Error),
    DocumentParserError(DocumentParserError),
    BadData(::gpx::par::ElementError)
}

impl From<xml::reader::Error> for DocumentError {
    fn from(err: xml::reader::Error) -> DocumentError {
        DocumentError::ParserError(err)
    }
}

impl From<::gpx::par::ElementError> for DocumentError {
    fn from(err: ::gpx::par::ElementError) -> DocumentError {
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
pub struct BuildError {}

#[derive(Debug)]
pub enum AttributeError {
    InvalidValue(::par::AttributeValueError),
    Unexpected(OwnedName)
}

impl From<::par::AttributeValueError> for AttributeError {
    fn from(err: ::par::AttributeValueError) -> AttributeError {
        AttributeError::InvalidValue(err)
    }
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
    info: Option<ElemStart>,
    nodes: Vec<XmlNode>,
}

pub struct ElemStart {
    pub name: OwnedName,
    pub attributes: Vec<OwnedAttribute>,
    pub namespace: Namespace,
}

pub trait ElementBuild {
    type Element;
    type BuildError;
    fn build(self) -> Result<Self::Element, Self::BuildError>;
}

pub trait ElementParse<'a, R: Read, E>
    where Self: Sized + ElementBuild,
          E: From<AttributeError> + From<ElementError> + From<Self::BuildError>
             + From<xml::reader::Error> {
    // public iface
    fn new(reader: &'a mut EventReader<R>) -> Self;
    
    /// Parses the element and its subelements, returning ElementBuild::Element instance.
    fn parse(mut self, elem_start: ElemStart) -> Result<Self::Element, PositionedError<E>> {
        try!(self.parse_start(elem_start).map_err(|e| self._with_pos(e)));
        loop {
            match try!(self.next().map_err(|e| self._with_pos(e))) {
                XmlEvent::StartElement { name, attributes, namespace } => {
                    try!(self.parse_element(ElemStart { name: name,
                                                        attributes: attributes,
                                                        namespace: namespace }));
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
        self.build().map_err(|e| PositionedError::with_position(e.into(), pos))
    }
    /// Helper, converts any parse error to the positioned error type. Not a closure to hopefully save performance
    fn _with_pos<Kind: Into<E>>(&self, kind: Kind) -> PositionedError<E> {
        PositionedError::with_position(kind.into(), self.get_parser_position())
    }
    /// Helper, equivalent to self.reader.position()
    fn get_parser_position(&self) -> TextPosition;
    /// Helper, remove
    //fn parse_self(self, elem_start: ElemStart) -> Result<Self::Element, self::Error> {
      //  self.parse(elem_start)
    //}
    /// Parses the start event and attributes within it. Should be implemented, bu default ignores attributes.
    fn parse_start(&mut self, elem_start: ElemStart) -> Result<(), AttributeError> {
        let _ = elem_start;
        Ok(())
    }
    /// Parses sub-element.
    fn parse_element(&mut self, elem_start: ElemStart) -> Result<(), PositionedError<E>>;
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
        let elem_start = self.info.unwrap(); // this is a programming error if info is not present here
        Ok(XmlElement {
            name: elem_start.name,
            attributes: elem_start.attributes,
            namespace: elem_start.namespace,
            nodes: self.nodes
        })
    }
}

impl<'a, T: Read> ElementParse<'a, T, ::gpx::par::_ElementError> for ElementParser<'a, T> {
    fn new(reader: &'a mut EventReader<T>)
            -> ElementParser<'a, T> {
        ElementParser { reader: reader,
                        info: None,
                        nodes: Vec::new() }
    }
    fn parse_start(&mut self, elem_start: ElemStart) -> Result<(), AttributeError> {
        self.info = Some(elem_start);
        Ok(())
    }
    fn parse_element(&mut self, elem_start: ElemStart) -> Result<(), ::gpx::par::Error> {
        let elem = try!(ElementParser::new(self.reader).parse(elem_start));
        self.nodes.push(XmlNode::Element(elem));
        Ok(())
    }
    fn parse_characters(&mut self, data: String) -> Result<(), ::gpx::par::_ElementError> {
        self.nodes.push(XmlNode::Text(data));
        Ok(())
    }
    fn get_parser_position(&self) -> TextPosition {
        self.reader.position()
    }
    fn get_name(&self) -> &OwnedName {
        match &self.info {
            &Some(ref i) => &i.name,
            &None => unreachable!(),
        }
    }
    fn next(&mut self) -> Result<XmlEvent, xml::reader::Error> {
        self.reader.next()
    }
}

pub struct Document<T> {
    pub info: DocInfo,
    pub data: T,
}

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
                XmlEvent::StartElement { name, attributes, namespace } => {
                    let start = ElemStart { name: name,
                                            attributes: attributes,
                                            namespace: namespace };
                    try!(contents.parse_element(&mut reader, start));
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
    type Error: From<xml::reader::Error> + From<DocumentParserError> + From<::gpx::par::ElementError>;
    // public iface
    fn parse_element<R: Read>(&mut self, reader: &mut EventReader<R>, elem_start: ElemStart)
            -> Result<(), ::gpx::par::ElementError>;
    fn build(self) -> Result<Self::Contents, Self::Error>;
}

#[derive(Default)]
struct ParserData(Vec<XmlNode>);

impl DocumentParserData for ParserData {
    type Contents = Vec<XmlNode>;
    type Error = DocumentError;
    fn parse_element<R: Read>(&mut self, mut reader: &mut EventReader<R>, elem_start: ElemStart)
            -> Result<(), ::gpx::par::ElementError> {
        let elem = try!(ElementParser::new(&mut reader).parse(elem_start));
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
