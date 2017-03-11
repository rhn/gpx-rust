//! Parsing of GPX files.
//!
//! Parses elements of XML files into Rust types

extern crate xml as _xml;
extern crate chrono;

use std;
use std::fmt;
use std::io;
use std::io::Read;
use std::str::FromStr;
use std::error::Error as ErrorTrait;
use self::_xml::common::Position;
use self::_xml::name::OwnedName;
use self::_xml::reader::{ XmlEvent, EventReader };

use xml;
use xml::{ DocumentParserData, XmlElement, ElemStart, ElementParser, ElementParse, ElementBuild };
use xsd;
use xsd::par::{ parse_time, parse_decimal };
use gpx;
use gpx::{ Document, Gpx, Bounds, Version, Waypoint, Fix, Metadata, Point, TrackSegment, Track, Route, Link, Degrees };
use gpx::conv;
use gpx::conv::{ Latitude, Longitude };
use ::par::{ FromAttribute, ParseVia, ParseViaChar, parse_string, parse_u64, parse_elem };
use ::par::{ Positioned, AttributeValueError };


include!(concat!(env!("OUT_DIR"), "/gpx_par_auto.rs"));


/// Describes a failure while parsing data
#[derive(Debug)]
pub enum Error {
    Str(&'static str),
    XmlEvent(_xml::reader::Error),
    BadInt(std::num::ParseIntError),
    BadFloat(std::num::ParseFloatError),
    BadString(std::string::ParseError),
    BadTime(chrono::ParseError),
    BadAttribute(xml::AttributeError),
    BadElement(xml::ElementError),
    BadShape(xml::BuildError),
    UnknownElement(OwnedName),
}

impl From<xml::AttributeError> for Error {
    fn from(err: xml::AttributeError) -> Error {
        Error::BadAttribute(err)
    }
}

impl From<xml::ElementError> for Error {
    fn from(err: xml::ElementError) -> Error {
        Error::BadElement(err)
    }
}

impl From<xml::BuildError> for Error {
    fn from(err: xml::BuildError) -> Error {
        Error::BadShape(err)
    }
}

impl From<_xml::reader::Error> for Error {
    fn from(err: _xml::reader::Error) -> Error {
        Error::XmlEvent(err)
    }
}

impl From<&'static str> for Error {
    fn from(msg: &'static str) -> Error {
        Error::Str(msg)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Error {
        Error::BadInt(err)
    }
}

impl From<std::string::ParseError> for Error {
    fn from(err: std::string::ParseError) -> Error {
        Error::BadString(err)
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(err: std::num::ParseFloatError) -> Error {
        Error::BadFloat(err)
    }
}

impl From<chrono::ParseError> for Error {
    fn from(err: chrono::ParseError) -> Error {
        Error::BadTime(err)
    }
}

/// FIXME: Remove this once xml::Error figured out
impl From<xml::Error> for Error {
    #[allow(unused_variables)]
    fn from(err: xml::Error) -> Error {
        Error::Str("BUG: xml::Error")
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Debug::fmt(self, fmt)
    }
}

impl ErrorTrait for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Str(_) => "Str (FIXME)",
            Error::XmlEvent(_) => "XmlEvent",
            Error::BadInt(_) => "Bad int",
            Error::BadFloat(_) => "Bad float",
            Error::BadString(_) => "Bad string",
            Error::BadTime(_) => "Bad time",
            Error::BadShape(_) => "Wrong elements number",
            Error::BadAttribute(_) => "Bad attribute",
            Error::BadElement(_) => "Bad element",
            Error::UnknownElement(_) => "Unknown element",
        }
    }
}

impl FromAttribute<f64> for Latitude {
    fn from_attr(attr: &str) -> Result<f64, AttributeValueError> {
        f64::from_str(attr).map_err(|e| { AttributeValueError::Error(Box::new(e)) })
    }
}

impl FromAttribute<f64> for Longitude {
    fn from_attr(attr: &str) -> Result<f64, AttributeValueError> {
        f64::from_str(attr).map_err(|e| { AttributeValueError::Error(Box::new(e)) })
    }
}

impl FromAttribute<Version> for conv::Version {
    fn from_attr(attr: &str) -> Result<Version, AttributeValueError> {
        match attr {
            "1.0" => Ok(Version::V1_0),
            "1.1" => Ok(Version::V1_1),
            _ => Err(AttributeValueError::Str("Unknown GPX version"))
        }
    }
}

impl<'a, T: Read> ElementBuild for BoundsParser<'a, T> {
    type Element = Bounds;
    type BuildError = xml::BuildError;
    fn build(self) -> Result<Self::Element, Self::BuildError> {
        Ok(Bounds { xmin: self.minlat.unwrap(),
                    ymin: self.minlon.unwrap(),
                    xmax: self.maxlat.unwrap(),
                    ymax: self.maxlon.unwrap() })
    }
}

impl ParseVia<Bounds> for conv::Bounds {
    fn parse_via<R: io::Read>(parser: &mut EventReader<R>, elem_start: ElemStart)
            -> Result<Bounds, Positioned<Error>> {
        BoundsParser::new(parser).parse(elem_start)
    }
}

impl ParseVia<Metadata> for conv::Metadata {
    fn parse_via<R: io::Read>(parser: &mut EventReader<R>, elem_start: ElemStart)
            -> Result<Metadata, Positioned<Error>> {
        MetadataParser::new(parser).parse(elem_start)
    }
}

impl ParseVia<Route> for conv::Rte {
    fn parse_via<R: io::Read>(parser: &mut EventReader<R>, elem_start: ElemStart)
            -> Result<Route, Positioned<Error>> {
        RteParser::new(parser).parse(elem_start)
    }
}

impl ParseVia<Track> for conv::Trk {
    fn parse_via<R: io::Read>(parser: &mut EventReader<R>, elem_start: ElemStart)
            -> Result<Track, Positioned<Error>> {
        TrkParser::new(parser).parse(elem_start)
    }
}

impl ParseVia<TrackSegment> for conv::Trkseg {
    fn parse_via<R: io::Read>(parser: &mut EventReader<R>, elem_start: ElemStart)
            -> Result<TrackSegment, Positioned<Error>> {
        TrackSegmentParser::new(parser).parse(elem_start)
    }
}

impl ParseVia<Link> for conv::Link {
    fn parse_via<R: io::Read>(parser: &mut EventReader<R>, elem_start: ElemStart)
            -> Result<Link, Positioned<Error>> {
        LinkParser::new(parser).parse(elem_start)
    }
}

impl ParseViaChar<Fix> for conv::Fix {
    fn from_char(s: &str) -> Result<Fix, Error> {
        Ok(match s {
            "none" => Fix::None,
            "2d" => Fix::_2D,
            "3d" => Fix::_3D,
            "dgps" => Fix::DGPS,
            "pps" => Fix::PPS,
            _ => return Err(Error::Str("Unknown fix kind")),
        })
    }
}

impl<'a, T: Read> ElementBuild for MetadataParser<'a, T> {
    type Element = Metadata;
    type BuildError = xml::BuildError;
    fn build(self) -> Result<Self::Element, Self::BuildError> {
        Ok(Metadata { name: self.name,
                      description: self.desc,
                      author: self.author,
                      copyright: self.copyright,
                      links: self.link,
                      time: self.time,
                      keywords: self.keywords,
                      bounds: self.bounds,
                      extensions: self.extensions })
    }
}

/// Waypoints need custom building because of the "location" field being composed of attributes and an element.
impl<'a, T: Read> ElementBuild for WaypointParser<'a, T> {
    type Element = Waypoint;
    type BuildError = xml::BuildError;
    fn build(self) -> Result<Self::Element, Self::BuildError> {
        Ok(Waypoint { location: Point { latitude: self.lat.unwrap(),
                                        longitude: self.lon.unwrap(),
                                        elevation: self.ele },
                      time: self.time,
                      mag_variation: self.magvar,
                      geoid_height: self.geoidheight,
                      name: self.name,
                      comment: self.cmt,
                      description: self.desc,
                      source: self.src,
                      links: self.link,
                      symbol: self.sym,
                      type_: self.type_,
                      fix: self.fix,
                      satellites: self.sat,
                      hdop: self.hdop,
                      pdop: self.pdop,
                      vdop: self.vdop,
                      dgps_age: self.ageofdgpsdata,
                      dgps_id: self.dgpsid,
                      extensions: self.extensions })
    }
}

/// Error describing a failure while parsing an XML stream
#[derive(Debug)]
pub enum DocumentError {
    ParserError(_xml::reader::Error),
    DocumentParserError(xml::DocumentParserError),
    BadData(Positioned<Error>),
    MissingGpx,
}

impl From<_xml::reader::Error> for DocumentError {
    fn from(err: _xml::reader::Error) -> DocumentError {
        DocumentError::ParserError(err)
    }
}

impl From<Positioned<Error>> for DocumentError {
    fn from(err: Positioned<Error>) -> DocumentError {
        DocumentError::BadData(err)
    }
}

impl From<xml::DocumentParserError> for DocumentError {
    fn from(err: xml::DocumentParserError) -> DocumentError {
        DocumentError::DocumentParserError(err)
    }
}

#[derive(Default)]
struct ParserData(Option<Gpx>);

impl DocumentParserData for ParserData {
    type Contents = Gpx;
    type Error = DocumentError;
    fn parse_element<R: Read>(&mut self, mut reader: &mut EventReader<R>, elem_start: ElemStart)
            -> Result<(), Positioned<Error>> {
        if let &mut ParserData(Some(_)) = self {
            return Err(Positioned::with_position("Duplicate GPX element".into(),
                                                 reader.position()));
        }
        self.0 = Some(try!(GpxElemParser::new(&mut reader).parse(elem_start)));
        Ok(())
    }
    fn build(self) -> Result<Gpx, Self::Error> {
        match self.0 {
            Some(gpx) => Ok(gpx),
            None => Err(DocumentError::MissingGpx)
        }
    }
}

/// Takes in GPX stream and returns an instance of `gpx::Document`.
///
/// ```
/// let f = File::open("foo").unwrap();
/// let xml_gpx = gpx::par::parse(f).unwrap();
/// ```
pub fn parse<R: Read>(source: R) -> Result<Document, DocumentError> {
    xml::parse_document::<R, ParserData>(source)
}
