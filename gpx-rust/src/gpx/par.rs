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
use self::_xml::common::{ Position, TextPosition };
use self::_xml::name::OwnedName;
use self::_xml::reader::{ XmlEvent, EventReader };

use xml;
use xml::{ ParseXml, DocInfo, XmlElement, ElemStart, ElementParser, ElementParse, ElementBuild };
use xsd;
use xsd::par::{ parse_time, parse_decimal };
use gpx;
use gpx::{ Gpx, Bounds, GpxVersion, Waypoint, Fix, Metadata, Point, TrackSegment, Track, Route, Link, Degrees };
use gpx::conv;
use gpx::conv::{ Latitude, Longitude };
use ::par::{ ParseVia, parse_chars, parse_string, parse_u64, parse_elem };
use ::par::{ ElementError as ElementErrorTrait, ElementErrorFree, AttributeValueError };

include!(concat!(env!("OUT_DIR"), "/gpx_par_auto.rs"));


#[derive(Debug)]
pub struct ElementError {
    pub error: _ElementError,
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
    type Free = _ElementError;
    fn with_position(err: Self::Free, position: TextPosition) -> Self {
        ElementError { error: err, position: position }
    }
}



#[derive(Debug)]
pub enum _ElementError {
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

impl From<xml::AttributeError> for _ElementError {
    fn from(err: xml::AttributeError) -> _ElementError {
        _ElementError::BadAttribute(err)
    }
}

impl From<xml::ElementError> for _ElementError {
    fn from(err: xml::ElementError) -> _ElementError {
        _ElementError::BadElement(err)
    }
}

impl From<xml::BuildError> for _ElementError {
    fn from(err: xml::BuildError) -> _ElementError {
        _ElementError::BadShape(err)
    }
}

impl From<_xml::reader::Error> for _ElementError {
    fn from(err: _xml::reader::Error) -> _ElementError {
        _ElementError::XmlEvent(err)
    }
}

impl From<&'static str> for _ElementError {
    fn from(msg: &'static str) -> _ElementError {
        _ElementError::Str(msg)
    }
}

impl From<std::num::ParseIntError> for _ElementError {
    fn from(err: std::num::ParseIntError) -> _ElementError {
        _ElementError::BadInt(err)
    }
}

impl From<std::string::ParseError> for _ElementError {
    fn from(err: std::string::ParseError) -> _ElementError {
        _ElementError::BadString(err)
    }
}

impl From<std::num::ParseFloatError> for _ElementError {
    fn from(err: std::num::ParseFloatError) -> _ElementError {
        _ElementError::BadFloat(err)
    }
}

impl From<chrono::ParseError> for _ElementError {
    fn from(err: chrono::ParseError) -> _ElementError {
        _ElementError::BadTime(err)
    }
}

/// FIXME: Remove this once xml::Error figured out
impl From<xml::Error> for _ElementError {
    #[allow(unused_variables)]
    fn from(err: xml::Error) -> _ElementError {
        _ElementError::Str("BUG: xml::Error")
    }
}

impl ElementErrorFree for _ElementError {}

impl fmt::Display for _ElementError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Debug::fmt(self, fmt)
    }
}

impl ErrorTrait for _ElementError {
    fn description(&self) -> &str {
        match *self {
            _ElementError::Str(_) => "Str (FIXME)",
            _ElementError::XmlEvent(_) => "XmlEvent",
            _ElementError::BadInt(_) => "Bad int",
            _ElementError::BadFloat(_) => "Bad float",
            _ElementError::BadString(_) => "Bad string",
            _ElementError::BadTime(_) => "Bad time",
            _ElementError::BadShape(_) => "Wrong elements number",
            _ElementError::BadAttribute(_) => "Bad attribute",
            _ElementError::BadElement(_) => "Bad element",
            _ElementError::UnknownElement(_) => "Unknown element",
        }
    }
}

/// FIXME: move to general par.rs
pub trait FromAttribute<T> {
    fn from_attr(&str) -> Result<T, AttributeValueError>;
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

impl FromAttribute<GpxVersion> for conv::Version {
    fn from_attr(attr: &str) -> Result<GpxVersion, AttributeValueError> {
        match attr {
            "1.0" => Ok(GpxVersion::V1_0),
            "1.1" => Ok(GpxVersion::V1_1),
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
            -> Result<Bounds, ElementError> {
        BoundsParser::new(parser).parse(elem_start)
    }
}

impl ParseVia<Metadata> for conv::Metadata {
    fn parse_via<R: io::Read>(parser: &mut EventReader<R>, elem_start: ElemStart)
            -> Result<Metadata, ElementError> {
        MetadataParser::new(parser).parse(elem_start)
    }
}

impl ParseVia<Route> for conv::Rte {
    fn parse_via<R: io::Read>(parser: &mut EventReader<R>, elem_start: ElemStart)
            -> Result<Route, ElementError> {
        RteParser::new(parser).parse(elem_start)
    }
}

impl ParseVia<Track> for conv::Trk {
    fn parse_via<R: io::Read>(parser: &mut EventReader<R>, elem_start: ElemStart)
            -> Result<Track, ElementError> {
        TrkParser::new(parser).parse(elem_start)
    }
}

impl ParseVia<TrackSegment> for conv::Trkseg {
    fn parse_via<R: io::Read>(parser: &mut EventReader<R>, elem_start: ElemStart)
            -> Result<TrackSegment, ElementError> {
        TrackSegmentParser::new(parser).parse(elem_start)
    }
}

impl ParseVia<Link> for conv::Link {
    fn parse_via<R: io::Read>(parser: &mut EventReader<R>, elem_start: ElemStart)
            -> Result<Link, ElementError> {
        LinkParser::new(parser).parse(elem_start)
    }
}

fn parse_fix<T: std::io::Read> (mut parser: &mut EventReader<T>, elem_start: ElemStart)
        -> Result<Fix, ElementError> {
    parse_chars(parser, elem_start, Fix::from_str)
}

pub fn copy(value: &str) -> Result<String, AttributeValueError> {
    Ok(value.into())
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

impl<'a, T: Read> ElementBuild for TrackSegmentParser<'a, T> {
    type Element = TrackSegment;
    type BuildError = xml::BuildError;
    fn build(self) -> Result<Self::Element, Self::BuildError> {
        Ok(TrackSegment { waypoints: self.trkpt })
    }
}

/// Takes in GPX stream and returns an instance of `Gpx`.
///
/// ```
/// let f = File::open("foo").unwrap();
/// let gpx = DocumentParser::new(f).parse().unwrap();
/// ```
pub struct DocumentParser<T: Read> {
    reader: EventReader<T>,
    gpx: Option<Gpx>,
}

impl<T: Read> ParseXml<T> for DocumentParser<T> {
    type Document = Gpx;
    type Error = xml::DocumentError;
    fn new(source: T) -> Self {
        DocumentParser { reader: EventReader::new(source),
                         gpx: None }
    }
    fn next(&mut self) -> Result<XmlEvent, _xml::reader::Error> {
        self.reader.next()
    }
    fn handle_info(&mut self, info: DocInfo) -> () {
        let _ = info;
    }
    
    fn parse_element(&mut self, elem_start: ElemStart) -> Result<(), ElementError> {
        if let Some(_) = self.gpx {
            return Err(ElementError::with_position("Duplicate GPX".into(),
                                                   self.reader.position()));
        }
        let gpx = try!(GpxElemParser::new(&mut self.reader).parse(elem_start));
        self.gpx = Some(gpx);
        Ok(())
    }
    fn build(self) -> Result<Gpx, Self::Error> {
        match self.gpx {
            Some(gpx) => Ok(gpx),
            None => Err(ElementError::with_position("Missing GPX".into(),
                                                    self.reader.position()).into())
        }
    }
}
