extern crate xml;

use std::io;
use std::io::Read;
use self::xml::name::OwnedName;
use self::xml::attribute::OwnedAttribute;
use self::xml::namespace::Namespace;
use self::xml::reader::{ EventReader, XmlEvent };
use self::xml::common::XmlVersion;


#[derive(Debug)]
pub enum Error {
    Str(&'static str),
    Io(io::Error),
    Xml(xml::reader::Error),
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

#[derive(Debug)]
pub struct XmlDocument {
    pub version: XmlVersion,
    pub encoding: String,
    pub standalone: Option<bool>,
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

pub struct XmlParser<T: Read> {
    reader: EventReader<T>,
    info: Option<DocInfo>,
    nodes: Vec<XmlNode>,
}

pub struct DocInfo {
    version: XmlVersion,
    encoding: String,
    standalone: Option<bool>,
}

pub trait ElementBuild {
    type Element;
    type Error : From<Error> + From<&'static str>;
    fn build(self) -> Result<Self::Element, Self::Error>;
}

pub trait ElementParse<'a, T: Read> where Self: Sized + ElementBuild {
    // public iface
    fn new(reader: &'a mut EventReader<T>) -> Self;
    fn parse(mut self, elem_start: ElemStart) -> Result<Self::Element, Self::Error> {
        try!(self.parse_start(elem_start));
        loop {
            match try!(self.next()) {
                XmlEvent::StartElement { name, attributes, namespace } => {
                    try!(self.parse_element(ElemStart { name: name, attributes: attributes, namespace: namespace }));
                }
                XmlEvent::EndElement { name } => {
                    if &name == self.get_name() {
                        break;
                    }
                    return Err(Self::Error::from("Unexpected end"));
                }
                XmlEvent::Characters(data) => {
                    try!(self.parse_characters(data));
                }
                XmlEvent::Whitespace(s) => {
                    try!(self.parse_whitespace(s));}
                _ => {
                    return Err(Self::Error::from("Unexpected event"));
                }
            }
        }
        self.build()
    }
    fn parse_start(&mut self, elem_start: ElemStart) -> Result<(), Self::Error> {
        let _ = elem_start;
        Ok(())
    }
    fn parse_element(&mut self, elem_start: ElemStart)
            -> Result<(), Self::Error>;
    fn parse_characters(&mut self, data: String) -> Result<(), Self::Error> {
        let _ = data;
        Ok(())
    }
    fn parse_whitespace(&mut self, space: String) -> Result<(), Self::Error> {
        let _ = space;
        Ok(())
    }
    fn get_name(&self) -> &OwnedName;
    fn next(&mut self) -> Result<XmlEvent, Error>;
}

impl<'a, T: Read> ElementBuild for ElementParser<'a, T> {
    type Element = XmlElement;
    type Error = Error;
    fn build(self) -> Result<XmlElement, Error> {
        let elem_start = self.info.unwrap(); // this is a programming error if info is not present here
        Ok(XmlElement {
            name: elem_start.name,
            attributes: elem_start.attributes,
            namespace: elem_start.namespace,
            nodes: self.nodes
        })
    }
}

impl<'a, T: Read> ElementParse<'a, T> for ElementParser<'a, T> {
    fn new(reader: &'a mut EventReader<T>)
            -> ElementParser<'a, T> {
        ElementParser { reader: reader,
                        info: None,
                        nodes: Vec::new() }
    }
    fn parse_start(&mut self, elem_start: ElemStart) -> Result<(), Self::Error> {
        self.info = Some(elem_start);
        Ok(())
    }
    fn parse_element(&mut self, elem_start: ElemStart)
            -> Result<(), Self::Error> {
        let elem = try!(ElementParser::new(self.reader).parse(elem_start));
        self.nodes.push(XmlNode::Element(elem));
        Ok(())
    }
    fn parse_characters(&mut self, data: String) -> Result<(), Self::Error> {
        self.nodes.push(XmlNode::Text(data));
        Ok(())
    }
    fn get_name(&self) -> &OwnedName {
        match &self.info {
            &Some(ref i) => &i.name,
            &None => unreachable!(),
        }
    }
    fn next(&mut self) -> Result<XmlEvent, Error> {
        self.reader.next().map_err(Error::Xml)
    }
}

pub trait ParseXml<T: Read> where Self: Sized {
    type Document;
    type Error : From<Error> + From<&'static str>;
    // public iface
    fn new(source: T) -> Self;
    fn parse(mut self) -> Result<Self::Document, Self::Error> {
        let mut state = ParserState::PreStart;
        loop {
            let next = try!(self.next());
            state = match state {
                ParserState::PreStart => match next {
                    XmlEvent::StartDocument { version, encoding, standalone } => {
                        self.handle_info(DocInfo { version: version, encoding: encoding, standalone: standalone });
                        ParserState::Inside
                    }
                    ev => {
                        println!("{:?}", ev);
                        return Err(Self::Error::from("Event invalid for PreStart"));
                    }
                },
                ParserState::Inside => match next {
                    XmlEvent::StartElement { name, attributes, namespace } => {
                        let start = ElemStart { name: name,
                                                attributes: attributes,
                                                namespace: namespace };
                        try!(self.parse_element(start));
                        ParserState::Inside
                    }
                    // TODO: more events
                    XmlEvent::EndDocument => ParserState::PostEnd,
                    ev => {
                        println!("{:?}", ev);
                        return Err(Self::Error::from("Event invalid or unhandled while Inside"));
                    }
                },
                ParserState::PostEnd => { break; }
            }
        }
        self.build()
    }
    
    // internal
    fn next(&mut self) -> Result<XmlEvent, Error>;
    fn handle_info(&mut self, info: DocInfo) -> ();
    fn parse_element(&mut self, elem_start: ElemStart)
        -> Result<(), Self::Error>;
    fn build(self) -> Result<Self::Document, Self::Error>;
}

impl<T: Read> ParseXml<T> for XmlParser<T> {
    type Document = XmlDocument;
    type Error = Error;
    fn new(source: T) -> Self {
        XmlParser { reader: EventReader::new(source),
                    info: None,
                    nodes: Vec::new() }
    }
    fn next(&mut self) -> Result<XmlEvent, Error> {
        self.reader.next().map_err(Error::Xml)
    }
    fn handle_info(&mut self, info: DocInfo) -> () {
        self.info = Some(info)
    }
    fn parse_element(&mut self, elem_start: ElemStart)
            -> Result<(), Self::Error> {
        let elem = try!(ElementParser::new(&mut self.reader).parse(elem_start));
        self.nodes.push(XmlNode::Element(elem));
        Ok(())
    }
    fn build(self) -> Result<XmlDocument, Error> {
        let info = self.info.unwrap();
        Ok(XmlDocument { version: info.version,
                         encoding: info.encoding,
                         standalone: info.standalone,
                         nodes: self.nodes })
    }
}


pub enum WspMode {
    None,
    IndentLevel(u16),
}

impl WspMode {
    pub fn next(self) -> WspMode {
        match self {
            WspMode::None => WspMode::None,
            WspMode::IndentLevel(i) => WspMode::IndentLevel(i + 1),
        }
    }
    pub fn prev(self) -> WspMode {
        match self {
            WspMode::None => WspMode::None,
            WspMode::IndentLevel(i) => 
                if i <= 0 { panic!("Indent level cannot be lower than 0") }
                else { WspMode::IndentLevel(i + 1) }
        }
    }
}

pub trait Serialize {
    fn serialize<Stream: io::Write>(&self, out: Stream, whitespace: WspMode) -> io::Result<usize>; 
}
