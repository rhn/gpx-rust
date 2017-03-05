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
use xml::{ XmlElement, ElemStart, ElementParser, ElementParse, ElementBuild };
use xsd;
use xsd::par::{ parse_time, parse_decimal };
use gpx;
use gpx::{ Error, ElementError, Gpx, Bounds, GpxVersion, Waypoint, Fix, Metadata, Point, TrackSegment, Track, Route, Link, Degrees };
use gpx::conv;
use gpx::conv::{ Latitude, Longitude };
use ::par::{ ParseVia, parse_chars, parse_string, parse_u64, parse_elem, ParserMessage };
use ::par::{ ElementError as ElementErrorTrait, ElementErrorFree };

include!(concat!(env!("OUT_DIR"), "/gpx_par_auto.rs"));


#[derive(Debug)]
pub enum _ElementError {
    Str(&'static str),
    XmlEvent(_xml::reader::Error),
    BadInt(std::num::ParseIntError),
    BadString(std::string::ParseError),
    BadTime(chrono::ParseError),
    UnknownElement(OwnedName),
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

impl From<chrono::ParseError> for _ElementError {
    fn from(err: chrono::ParseError) -> _ElementError {
        _ElementError::BadTime(err)
    }
}

/// FIXME: Remove this once gpx::Error figured out
impl From<gpx::Error> for _ElementError {
    #[allow(unused_variables)]
    fn from(err: gpx::Error) -> _ElementError {
        _ElementError::Str("BUG: gpx::Error")
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
            _ElementError::BadString(_) => "Bad string",
            _ElementError::BadTime(_) => "Bad time",
            _ElementError::UnknownElement(_) => "Unknown element",
        }
    }
}

/// Raise whenever attribute value is out of bounds
#[derive(Debug)]
pub enum AttributeValueError {
    Str(&'static str),
    Error(Box<std::error::Error>),
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
    type Error = Error;
    fn build(self) -> Result<Self::Element, Self::Error> {
        Ok(Bounds { xmin: self.minlat.unwrap(),
                    ymin: self.minlon.unwrap(),
                    xmax: self.maxlat.unwrap(),
                    ymax: self.maxlon.unwrap() })
    }
}

impl ParseVia<Bounds> for conv::Bounds {
    fn parse_via<R: io::Read>(parser: &mut EventReader<R>, elem_start: ElemStart)
            -> Result<Bounds, ElementError> {
        BoundsParser::new(parser).parse_self(elem_start)
    }
}

impl ParseVia<Metadata> for conv::Metadata {
    fn parse_via<R: io::Read>(parser: &mut EventReader<R>, elem_start: ElemStart)
            -> Result<Metadata, ElementError> {
        MetadataParser::new(parser).parse_self(elem_start)
    }
}

impl ParseVia<Route> for conv::Rte {
    fn parse_via<R: io::Read>(parser: &mut EventReader<R>, elem_start: ElemStart)
            -> Result<Route, ElementError> {
        RteParser::new(parser).parse_self(elem_start)
    }
}

impl ParseVia<Track> for conv::Trk {
    fn parse_via<R: io::Read>(parser: &mut EventReader<R>, elem_start: ElemStart)
            -> Result<Track, ElementError> {
        TrkParser::new(parser).parse_self(elem_start)
    }
}

impl ParseVia<TrackSegment> for conv::Trkseg {
    fn parse_via<R: io::Read>(parser: &mut EventReader<R>, elem_start: ElemStart)
            -> Result<TrackSegment, ElementError> {
        TrackSegmentParser::new(parser).parse_self(elem_start)
    }
}

impl ParseVia<Link> for conv::Link {
    fn parse_via<R: io::Read>(parser: &mut EventReader<R>, elem_start: ElemStart)
            -> Result<Link, ElementError> {
        LinkParser::new(parser).parse_self(elem_start)
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
    type Error = Error;
    fn build(self) -> Result<Self::Element, Self::Error> {
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
    type Error = Error;
    fn build(self) -> Result<Self::Element, Self::Error> {
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
    type Error = Error;
    fn build(self) -> Result<Self::Element, Self::Error> {
        Ok(TrackSegment { waypoints: self.trkpt })
    }
}
