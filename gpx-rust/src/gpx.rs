extern crate xml as _xml;
extern crate chrono;

use std;
use std::io;
use std::io::Read;
use std::error::Error as Error_;
use std::str::FromStr;
use self::_xml::reader::{ EventReader, XmlEvent };
use self::_xml::name::OwnedName;
use xml;
use xml::{ ParseXml, DocInfo, XmlElement, ElemStart, ElementParser, ElementParse, ElementBuild };
use parsers::*;
use xsd;
use xsd::*;


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
pub enum Error {
    Str(&'static str),
    Chrono(chrono::format::ParseError),
    Io(io::Error),
    Xml(_xml::reader::Error),
    ParseValue(std::string::ParseError), // use "bad tag" instead
    BadAttribute(OwnedName, OwnedName),
    MalformedData(String),
}

impl From<chrono::ParseError> for Error {
    fn from(err: chrono::ParseError) -> Error {
        Error::Chrono(err)
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

macro_attr! {
    #[derive(Debug, ParserExp!(GpxElemParser {
        attrs: { "version" => { version, parse_gpx_version },
                 "creator" => { creator, Ok<String, Error> }},
        tags: { "metadata" => { metadata = Some, ElementParse, MetadataParser },
                "wpt" => { waypoints = Vec, ElementParse, WptParser },
                "trk" => { tracks = Vec, ElementParse, TrkParser }
            }
    }))]
    pub struct Gpx {
        version: One!(GpxVersion),
        creator: One!(String),
        metadata: Option!(Metadata),
        waypoints: Vec!(Waypoint),
        tracks: Vec!(Track),
    }
}

fn parse_gpx_version(value: String) -> Result<GpxVersion, Error> {
    match &value as &str {
        "1.0" => Ok(GpxVersion::V1_0),
        "1.1" => Ok(GpxVersion::V1_1),
        _ => Err(Error::Str("Unknown GPX version"))
    }
}

impl<'a, T: Read> ElementBuild for GpxElemParser<'a, T> {
    type Element = Gpx;
    type Error = Error;
    fn build(self) -> Result<Self::Element, Self::Error> {
        Ok(Gpx { version: self.version.expect("Version uninitialized"),
                 creator: self.creator.expect("Creator uninitialized"),
                 metadata: self.metadata,
                 waypoints: self.waypoints,
                 tracks: self.tracks })
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum GpxVersion {
    V1_0,
    V1_1,
}

macro_attr! {
    #[derive(XmlDebug,
    Parser!(MetadataParser {
        attrs: {},
        tags: { "author" => { author = Some, ElementParse, ElementParser },
                "copyright" => { copyright = Some, ElementParse, ElementParser },
                "link" => { links = Vec, ElementParse, ElementParser },
                "time" => { time = Some, fn, parse_time },
                "keywords" => { keywords = Some, fn, parse_string },
                "bounds" => { bounds = Some, ElementParse, ElementParser },
                "extensions" => { extensions = Some, ElementParse, ElementParser },}
    }))]
    pub struct Metadata {
        name: Option<String>,
        desc: Option<String>,
        author: Option<XmlElement>,
        copyright: Option<XmlElement>,
        links: Vec<Link>,
        time: Option<Time>,
        keywords: Option<String>,
        bounds: Option<XmlElement>,
        extensions: Option<XmlElement>,
    }
}

type Link = XmlElement;

#[derive(XmlDebug)]
pub struct Waypoint {
    location: LosslessPoint,
    time: Option<Time>,
    fix: Option<Fix>,
    satellites: Option<u16>,
    name: Option<String>,
}

struct WptParser<'a, T: 'a + Read> {
    reader: &'a mut EventReader<T>,
    elem_name: Option<OwnedName>,
    lat: Option<XmlDecimal>,
    lon: Option<XmlDecimal>,
    time: Option<Time>,
    fix: Option<Fix>,
    ele: Option<XmlDecimal>,
    sat: Option<u16>,
    name: Option<String>,
}

impl<'a, T: Read> ElementParse<'a, T> for WptParser<'a, T> {
    fn new(reader: &'a mut EventReader<T>) -> Self {
        WptParser { reader: reader,
                    elem_name: None,
                    name: None,
                    lat: None, lon: None, ele: None,
                    time: None,
                    fix: None, sat: None }
    }
    _ParserImplBody!(
        attrs: { "lat" => { lat, parse_dec },
                 "lon" => { lon, parse_dec }
        },
        tags: {
            "time" => { time = Some, fn, parse_time },
            "fix" => { fix = Some, fn, parse_fix },
            "ele" => { ele = Some, fn, parse_decimal },
            "sat" => { sat = Some, fn, parse_u16 },
            "name" => { name = Some, fn, parse_string },
        }
    );
}

fn parse_dec(value: String) -> Result<XmlDecimal, Error> {
    Ok(value)
}

impl<'a, T: Read> ElementBuild for WptParser<'a, T> {
    type Element = Waypoint;
    type Error = Error;
    fn build(self) -> Result<Self::Element, Self::Error> {
        Ok(Waypoint { location: LosslessPoint { latitude: self.lat.unwrap(),
                                                longitude: self.lon.unwrap(),
                                                elevation: self.ele },
                      time: self.time,
                      fix: self.fix,
                      satellites: self.sat,
                      name: self.name })
    }
}

#[derive(XmlDebug)]
struct LosslessPoint {
    latitude: XmlDecimal,
    longitude: XmlDecimal,
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
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "none" => Fix::None,
            "2d" => Fix::_2D,
            "3d" => Fix::_3D,
            "dgps" => Fix::DGPS,
            "pps" => Fix::PPS,
            _ => { return Err(Error::Str("Unknown fix kind")); }
        })
    }
}

macro_attr! {
    #[derive(Parser!(
        TrkParser {
            attrs: {},
            tags: {
                "name" => { name = Some, fn, parse_string },
                "trkseg" => { segments = Vec, ElementParse, TrkSegParser },
            }
        }
    ), XmlDebug)]
    pub struct Track {
        name: Option<String>,
        cmt: Option<String>,
        desc: Option<String>,
        src: Option<String>,
        link: Vec<Link>,
        number: Option<XmlNumber>,
        type_: Option<String>,
        extensions: Option<XmlElement>,
        segments: Vec<TrackSegment>,
    }
}

impl<'a, T: Read> ElementBuild for TrkParser<'a, T> {
    type Element = Track;
    type Error = Error;
    fn build(self) -> Result<Self::Element, Self::Error> {
        Ok(Track { name: self.name,
                   cmt: self.cmt,
                   desc: self.desc,
                   src: self.src,
                   link: self.link,
                   number: self.number,
                   type_: self.type_,
                   extensions: self.extensions,
                   segments: self.segments })
    }
}

type XmlNumber = String;

ElemParser!(
struct TrackSegment {
    waypoints: Vec<Waypoint>,
},
TrkSegParser {
    "trkpt" => { waypoints = Vec, ElementParse, WptParser },
}
);

impl<'a, T: Read> ElementBuild for TrkSegParser<'a, T> {
    type Element = TrackSegment;
    type Error = Error;
    fn build(self) -> Result<Self::Element, Self::Error> {
        Ok(TrackSegment { waypoints: self.waypoints })
    }
}

impl ParserMessage for Error {
    fn from_unexp_attr(elem_name: OwnedName, attr_name: OwnedName) -> Self {
        Error::BadAttribute(elem_name, attr_name)
    }
    fn from_xml_error(e: _xml::reader::Error) -> Self {
        Error::Xml(e)
    }
}

fn parse_fix<T: std::io::Read> (mut parser: &mut EventReader<T>, elem_start: ElemStart)
        -> Result<Fix, Error> {
    parse_chars(parser, elem_start, Fix::from_str)
}

fn parse_u16<T: std::io::Read> (mut parser: &mut EventReader<T>, elem_start: ElemStart)
        -> Result<u16, Error> {
    parse_chars(parser, elem_start,
                |chars| u16::from_str(chars).map_err(
                    |e| Error::MalformedData(String::from(e.description()))
                )
    )
}

fn parse_decimal<T: std::io::Read> (mut parser: &mut EventReader<T>, elem_start: ElemStart)
        -> Result<XmlDecimal, Error> {
    parse_chars(parser,
                elem_start,
                |chars| XmlDecimal::from_str(chars).map_err(Error::ParseValue))
}

pub fn parse_time<T: std::io::Read>
        (mut parser: &mut EventReader<T>, elem_start: ElemStart)
        -> Result<xsd::Time, Error> {
    parse_chars(parser, elem_start,
                |chars| xsd::Time::parse_from_rfc3339(chars).map_err(Error::from))
}

pub fn parse_string<T: std::io::Read> (mut parser: &mut EventReader<T>, elem_start: ElemStart)
        -> Result<String, Error> {
    parse_chars(parser,
                elem_start,
                |chars| String::from_str(chars).map_err(Error::ParseValue))
}

impl<'a, T: Read> ElementBuild for MetadataParser<'a, T> {
    type Element = Metadata;
    type Error = Error;
    fn build(self) -> Result<Self::Element, Self::Error> {
        Ok(Metadata { name: self.name,
                      desc: self.desc,
                      author: self.author,
                      copyright: self.copyright,
                      links: self.links,
                      time: self.time,
                      keywords: self.keywords,
                      bounds: self.bounds,
                      extensions: self.extensions })
    }
}

pub struct GpxParser<T: Read> {
    reader: EventReader<T>,
    gpx: Option<Gpx>,
}

impl<T: Read> ParseXml<T> for GpxParser<T> {
    type Document = Gpx;
    type Error = Error;
    fn new(source: T) -> Self {
        GpxParser { reader: EventReader::new(source),
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
