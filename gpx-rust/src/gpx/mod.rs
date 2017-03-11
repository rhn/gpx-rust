//! GPX types

extern crate xml as _xml;
extern crate chrono;
extern crate geo;


use std;
use std::io;
use std::fmt;

use self::geo::Bbox;
use self::_xml::name::OwnedName;

use xml;
use xml::XmlElement;
use xsd;
use xsd::*;
use par::{ Positioned, ParserMessage, AttributeValueError };

mod conv;
mod ser_auto;
pub mod ser;
pub mod par;

/// Parses XML stream containing GPX data
pub use self::par::parse;

pub type Document = xml::Document<Gpx>;

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
enum Error {
    Str(&'static str),
    Chrono(chrono::format::ParseError),
    Io(io::Error),
    Xml(_xml::reader::Error),
    ParseValue(std::string::ParseError), // use "bad tag" instead
    BadAttributeValue(AttributeValueError), // use when value out of XML spec for any attribute
    BadAttribute(OwnedName, OwnedName),
    MalformedData(String),
    BadElement(Positioned<par::Error>),
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
    fn from_bad_attr_val(e: AttributeValueError) -> Self {
        Error::BadAttributeValue(e)
    }
}

impl From<Positioned<par::Error>> for Error {
    fn from(err: Positioned<par::Error>) -> Error {
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

#[derive(Debug)]
pub struct Gpx {
    pub version: One!(Version),
    pub creator: One!(String),
    pub metadata: Option!(Metadata),
    pub waypoints: Vec!(Waypoint),
    pub routes: Vec<Route>,
    pub tracks: Vec<Track>,
    pub extensions: Option<XmlElement>,
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum Version {
    V1_0,
    V1_1,
}

#[derive(XmlDebug)]
pub struct Metadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub author: Option<XmlElement>,
    pub copyright: Option<XmlElement>,
    pub links: Vec<Link>,
    pub time: Option<Time>,
    pub keywords: Option<String>,
    pub bounds: Option<Bounds>,
    pub extensions: Option<XmlElement>,
}

#[derive(Debug)]
pub struct Link {
    pub href: xsd::Uri,
    pub text: Option<String>,
    pub type_: Option<String>,
}

type Bounds = Bbox<f64>;

/// `<wpt>`, `<rtept>`, `<trkpt>` elements and `wptType`
#[derive(XmlDebug)]
pub struct Waypoint {
    pub location: Point,
    pub time: Option<xsd::DateTime>,
    pub mag_variation: Option<Degrees>,
    pub geoid_height: Option<xsd::Decimal>,
    pub name: Option<String>,
    pub comment: Option<String>,
    pub description: Option<String>,
    pub source: Option<String>,
    pub links: Vec<Link>,
    pub symbol: Option<String>,
    pub type_: Option<String>,
    pub fix: Option<Fix>,
    pub satellites: Option<xsd::NonNegativeInteger>,
    pub hdop: Option<xsd::Decimal>,
    pub pdop: Option<xsd::Decimal>,
    pub vdop: Option<xsd::Decimal>,
    pub dgps_age: Option<xsd::Decimal>,
    pub dgps_id: Option<String>,
    pub extensions: Option<XmlElement>,
}

/// WGS84 geographical coordinates
#[derive(XmlDebug)]
pub struct Point {
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: Option<f64>,
}

#[derive(Debug)]
pub enum Fix {
    None,
    _2D,
    _3D,
    DGPS,
    PPS
}

/// `<trk>` and `trkType`
#[derive(XmlDebug)]
pub struct Track {
    pub name: Option<String>,
    pub comment: Option<String>,
    pub description: Option<String>,
    pub source: Option<String>,
    pub links: Vec<Link>,
    pub number: Option<xsd::NonNegativeInteger>,
    pub type_: Option<String>,
    pub extensions: Option<XmlElement>,
    pub segments: Vec<TrackSegment>,
}

/// `<trkseg>` and `trksegType`
#[derive(Debug)]
pub struct TrackSegment {
    pub waypoints: Vec<Waypoint>,
    pub extensions: Option<XmlElement>,
}

/// `<rte>` and `rteType`
#[derive(Debug)]
pub struct Route {
    pub name: Option<String>,
    pub comment: Option<String>,
    pub description: Option<String>,
    pub source: Option<String>,
    pub links: Vec<Link>,
    pub number: Option<xsd::NonNegativeInteger>,
    pub type_: Option<String>,
    pub extensions: Option<XmlElement>,
    pub waypoints: Vec<Waypoint>,
}

pub type Degrees = String; // FIXME
