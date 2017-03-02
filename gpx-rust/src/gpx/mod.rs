extern crate xml as _xml;
extern crate chrono;
extern crate geo;


use std;
use std::io;
use std::io::{ Read };
use std::error::Error as ErrorTrait;
use std::str::FromStr;
use std::fmt;

use self::geo::Bbox;
use self::_xml::common::{ Position, TextPosition };
use self::_xml::reader::{ EventReader, XmlEvent };
use self::_xml::name::OwnedName;

use xml;
use xml::{ ParseXml, DocInfo, XmlElement, ElemStart, ElementParser, ElementParse, ElementBuild };
use par::ElementError as ElementErrorTrait;
use parsers::*;
use xsd;
use xsd::*;
use par::{ ParserMessage, parse_string };

pub mod conv;
mod ser_auto;
pub mod ser;
pub mod par;

use self::par::{ WaypointParser, MetadataParser, TrackSegmentParser, GpxElemParser, _ElementError };


trait EmptyInit {
    fn empty() -> Self;
}

impl<T> EmptyInit for Option<T> {
    fn empty() -> Self { None }
}

impl<T> EmptyInit for Vec<T> {
    fn empty() -> Self { Vec::new() }
}

#[derive(Debug)]
pub struct ElementError {
    pub error: par::_ElementError,
    pub position: TextPosition,
}

impl fmt::Display for ElementError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "Position {}: {}", self.position, self.error)
    }
}

impl ErrorTrait for ElementError {
    fn description(&self) -> &str {
        ""
    }
    fn cause(&self) -> Option<&std::error::Error> {
        Some(&self.error)
    }
}

impl ElementErrorTrait for ElementError {
    type Free = par::_ElementError;
    fn from_free(err: Self::Free, position: TextPosition) -> Self {
        ElementError { error: err, position: position }
    }
}

#[derive(Debug)]
pub enum Error {
    Str(&'static str),
    Chrono(chrono::format::ParseError),
    Io(io::Error),
    Xml(_xml::reader::Error),
    ParseValue(std::string::ParseError), // use "bad tag" instead
    BadAttributeValue(par::AttributeValueError), // use when value out of XML spec for any attribute
    BadAttribute(OwnedName, OwnedName),
    MalformedData(String),
    BadElement(ElementError),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Debug::fmt(self, fmt)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Str(_) => "Str (depr)",
            Error::Chrono(_) => "Chrono (depr)",
            Error::Io(_) => "Io (?)",
            Error::Xml(_) => "Xml (?)",
            Error::ParseValue(_) => "ParseValue (?)",
            Error::BadAttributeValue(_) => "Bad attribute value",
            Error::BadAttribute(_, _) => "Bad attribute (depr)",
            Error::MalformedData(_) => "Malformed data (depr)",
            Error::BadElement(_) => "Bad element (?)",
        }
    }
    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::Chrono(ref e) => Some(e),
            Error::Io(ref e) => Some(e),
            Error::Xml(ref e) => Some(e),
            Error::ParseValue(ref e) => Some(e),
            //Error::BadAttributeValue(e) => Some(&e),
            Error::BadElement(ref e) => Some(e),
            _ => None
        }
    }
}


impl ParserMessage for Error {
    fn from_unexp_attr(elem_name: OwnedName, attr_name: OwnedName) -> Self {
        Error::BadAttribute(elem_name, attr_name)
    }
    fn from_xml_error(e: _xml::reader::Error) -> Self {
        Error::Xml(e)
    }
    // Consider From
    fn from_bad_attr_val(e: par::AttributeValueError) -> Self {
        Error::BadAttributeValue(e)
    }
}

impl From<ElementError> for Error {
    fn from(err: ElementError) -> Error {
        Error::BadElement(err)
    }
}

impl From<chrono::ParseError> for Error {
    fn from(err: chrono::ParseError) -> Error {
        Error::Chrono(err)
    }
}

impl From<_xml::reader::Error> for Error {
    fn from(err: _xml::reader::Error) -> Error {
        Error::Xml(err)
    }
}

impl From<xml::Error> for Error {
    fn from(err: xml::Error) -> Error {
        match err {
            xml::Error::Str(s) => Error::Str(s),
            xml::Error::Io(e) => Error::Io(e),
            xml::Error::Xml(e) => Error::Xml(e),
        }
    }
}

impl From<&'static str> for Error {
    fn from(msg: &'static str) -> Error {
        Error::Str(msg)
    }
}

impl From<std::string::ParseError> for Error {
    fn from(err: std::string::ParseError) -> Error {
        Error::ParseValue(err)
    }
}

#[macro_export]
macro_rules! ElemParser {
    ( struct $name:ident {
        $( $i:ident : $t:ty, )*
      },
      $parser:ident {
        $( $tag:pat => $tagdata:tt, )*
      }
    ) => {
        #[derive(XmlDebug)]
        struct $name { $( $i: $t, )* }

        struct $parser<'a, T: 'a + Read> {
            reader: &'a mut EventReader<T>,
            elem_name: Option<OwnedName>,
            $( $i: $t, )*
        }
        
        impl<'a, T: Read> ElementParse<'a, T> for $parser<'a, T> {
            fn new(reader: &'a mut EventReader<T>) -> Self {
                $parser { reader: reader,
                          elem_name: None,
                          $( $i : <$t>::empty(), )* }
            }

            ParserStart!();

            fn parse_element(&mut self, elem_start: ElemStart)
                    -> Result<(), Self::Error> {
                match &elem_start.name.local_name as &str {
                    $( $tag => {
                        make_tag!(T, self, elem_start, $tagdata);
                    }),*
                    _ => {
                        try!(ElementParser::new(self.reader).parse(elem_start));
                    }
                };
                Ok(())
            }
            
            fn get_name(&self) -> &OwnedName {
                match &self.elem_name {
                    &Some(ref i) => i,
                    &None => panic!("Name was not set while parsing"),
                }
            }
            fn next(&mut self) -> Result<XmlEvent, xml::Error> {
                self.reader.next().map_err(xml::Error::Xml)
            }
        }
    }
}

#[derive(Debug)]
pub struct Gpx {
    version: One!(GpxVersion),
    creator: One!(String),
    metadata: Option!(Metadata),
    waypoints: Vec!(Waypoint),
    tracks: Vec!(Track),
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum GpxVersion {
    V1_0,
    V1_1,
}

macro_attr! {
    #[derive(XmlDebug)]
    pub struct Metadata {
        name: Option<String>,
        description: Option<String>,
        author: Option<XmlElement>,
        copyright: Option<XmlElement>,
        links: Vec<Link>,
        time: Option<Time>,
        keywords: Option<String>,
        bounds: Option<Bounds>,
        extensions: Option<XmlElement>,
    }
}

type Link = XmlElement;
type Bounds = Bbox<f64>;

#[derive(XmlDebug)]
pub struct Waypoint {
    location: Point,
    time: Option<xsd::DateTime>,
    mag_variation: Option<xsd::Degrees>,
    geoid_height: Option<xsd::Decimal>,
    name: Option<String>,
    comment: Option<String>,
    description: Option<String>,
    source: Option<String>,
    links: Vec<XmlElement>,
    symbol: Option<String>,
    type_: Option<String>,
    fix: Option<Fix>,
    satellites: Option<xsd::NonNegativeInteger>,
    hdop: Option<xsd::Decimal>,
    pdop: Option<xsd::Decimal>,
    vdop: Option<xsd::Decimal>,
    dgps_age: Option<xsd::Decimal>,
    dgps_id: Option<String>,
    extensions: Option<XmlElement>,
}

#[derive(XmlDebug)]
struct Point {
    latitude: f64,
    longitude: f64,
    elevation: Option<XmlDecimal>,
}

type XmlDecimal = String;

#[derive(Debug)]
enum Fix {
    None,
    _2D,
    _3D,
    DGPS,
    PPS
}

impl FromStr for Fix {
    type Err = _ElementError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "none" => Fix::None,
            "2d" => Fix::_2D,
            "3d" => Fix::_3D,
            "dgps" => Fix::DGPS,
            "pps" => Fix::PPS,
            _ => { return Err(_ElementError::Str("Unknown fix kind")); }
        })
    }
}

macro_attr! {
    #[derive(Parser!(
        TrkParser {
            attrs: {},
            tags: {
                "name" => { name = Some, fn, parse_string },
                "cmt" => { comment = Some, fn, parse_string },
                "desc" => { description = Some, fn, parse_string },
                "src" => { source = Some, fn, parse_string },
                "link" => { links = Vec, ElementParse, ElementParser },
                "number" => { number = Some, fn, parse_int },
                "type" => { type_ = Some, fn, parse_string },
                "extensions" => { extensions = Some, ElementParse, ElementParser },
                "trkseg" => { segments = Vec, ElementParse, TrackSegmentParser }
            }
        }
    ), ElementBuild!(TrkParser, Error), XmlDebug)]
    pub struct Track {
        name: Option<String>,
        comment: Option<String>,
        description: Option<String>,
        source: Option<String>,
        links: Vec<Link>,
        number: Option<xsd::NonNegativeInteger>,
        type_: Option<String>,
        extensions: Option<XmlElement>,
        segments: Vec<TrackSegment>,
    }
}

fn parse_int<T: std::io::Read> (mut parser: &mut EventReader<T>, elem_start: ElemStart)
        -> Result<NonNegativeInteger, ElementError> {
    xsd::par::parse_int(parser, elem_start)
}

#[derive(Debug)]
pub struct TrackSegment {
    waypoints: Vec<Waypoint>,
}

pub struct Parser<T: Read> {
    reader: EventReader<T>,
    gpx: Option<Gpx>,
}

impl<T: Read> ParseXml<T> for Parser<T> {
    type Document = Gpx;
    type Error = Error;
    fn new(source: T) -> Self {
        Parser { reader: EventReader::new(source),
                    gpx: None }
    }
    fn next(&mut self) -> Result<XmlEvent, self::xml::Error> {
        self.reader.next().map_err(self::xml::Error::Xml)
    }
    fn handle_info(&mut self, info: DocInfo) -> () {
        let _ = info;
    }
    
    fn parse_element(&mut self, elem_start: ElemStart)
            -> Result<(), Error> {
        if let Some(_) = self.gpx {
            return Err(Error::Str("GPX element already present"));
        }
        let gpx = try!(GpxElemParser::new(&mut self.reader)
                           .parse(elem_start));
        self.gpx = Some(gpx);
        Ok(())
    }
    fn build(self) -> Result<Gpx, Error> {
        match self.gpx {
            Some(gpx) => Ok(gpx),
            None => Err(Error::Str("No gpx element found")),
        }
    }
}
