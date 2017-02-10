extern crate xml as _xml;
extern crate chrono;

use std;
use std::io;
use std::io::Read;
use std::str::FromStr;
use self::_xml::reader::{ EventReader, XmlEvent };
use self::_xml::name::OwnedName;
use xml;
use xml::{ ParseXml, DocInfo, XmlElement, ElemStart, ElementParser, ElementParse, ElementBuild };
use parsers::*;
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
macro_rules! ParserStart {
    ( $( $a:pat => $b:expr ),* ) => {
        fn parse_start(&mut self, elem_start: ElemStart)
                -> Result<(), Self::Error> {
            for attr in elem_start.attributes {
                match &(attr.name.local_name) as &str {
                    $( $a => $b ),*
                    _ => {
                        return Err(Self::Error::from_unexp_attr(elem_start.name, attr.name));
                    }
                }
            }
            self.elem_name = Some(elem_start.name);
            Ok(())
        }
    };
}

macro_rules! make_fn {
    ( fn, $parser:expr, $reader:expr, $elem_start:expr ) => {
        $parser($reader, $elem_start);
    };
    ( ElementParse, $parser:ty, $reader:expr, $elem_start:expr ) => {
        <$parser>::new($reader).parse($elem_start);
    };
}

macro_rules! make_tag {
    ( $T:ty, $self_:expr, $elem_start:expr, { $field:ident = Some, $ptype:tt, $parser:tt } ) => {
        $self_.$field = Some(try!(make_fn!($ptype, $parser, $self_.reader, $elem_start)));
    };
    ( $T:ty, $self_:expr, $elem_start:expr, { $field:ident = Vec, $ptype:tt, $parser:tt } ) => {
        $self_.$field.push(try!(make_fn!($ptype, $parser<$T>, $self_.reader, $elem_start)));
    };
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
    version: GpxVersion,
    creator: String,
    metadata: Option<Metadata>,
    waypoints: Vec<Waypoint>,
    tracks: Vec<Track>,
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum GpxVersion {
    V1_0,
    V1_1,
}

#[derive(XmlDebug)]
pub struct Metadata {
    name: Option<String>,
    desc: Option<String>,
    author: Option<XmlElement>,
    copyright: Option<XmlElement>,
    link: Vec<Link>,
    time: Option<Time>,
    keywords: Option<String>,
    bounds: Option<XmlElement>,
    extensions: Option<XmlElement>,
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

macro_attr! {
    #[derive(Parser!(
        TrkParser {
            "name" => { name = Some, fn, parse_string },
            "trkseg" => { segments = Vec, ElementParse, TrkSegParser },
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

impl<'a, T: Read> ElementParse<'a, T> for WptParser<'a, T> {
    fn new(reader: &'a mut EventReader<T>) -> Self {
        WptParser { reader: reader,
                    elem_name: None,
                    name: None,
                    lat: None, lon: None, ele: None,
                    time: None,
                    fix: None, sat: None }
    }
    fn parse_start(&mut self, elem_start: ElemStart) -> Result<(), Self::Error> {
        for attr in elem_start.attributes {
            match &(attr.name.local_name) as &str {
                "lat" => {
                    //TODO: check
                    self.lat = Some(attr.value);
                },
                "lon" => {
                    // TODO: check
                    self.lon = Some(attr.value);
                },
                _ => {
                    return Err(Self::Error::from_unexp_attr(elem_start.name, attr.name));
                }
            }
        }
        self.elem_name = Some(elem_start.name);
        Ok(())
    }
    fn parse_element(&mut self, elem_start: ElemStart)
            -> Result<(), Self::Error> {
        match &elem_start.name.local_name as &str {
            "time" => {
                self.time = Some(try!(parse_time::<T, Self::Error>(self.reader, elem_start)));
            }
            "ele" => {
                self.ele = Some(try!(parse_decimal(self.reader, elem_start)));
            }
            "name" => {
                self.name = Some(try!(parse_string(self.reader, elem_start)));
            }
            _ => {
                try!(ElementParser::new(self.reader).parse(elem_start));
            }
        }
        Ok(())
    }
    
    fn get_name(&self) -> &OwnedName {
        match &self.elem_name {
            &Some(ref i) => i,
            &None => unreachable!(),
        }
    }
    fn next(&mut self) -> Result<XmlEvent, self::xml::Error> {
        self.reader.next().map_err(self::xml::Error::Xml)
    }
}

fn parse_decimal<T: std::io::Read> (mut parser: &mut EventReader<T>, elem_start: ElemStart)
        -> Result<XmlDecimal, Error> {
    parse_chars(parser,
                elem_start,
                |chars| XmlDecimal::from_str(chars).map_err(Error::ParseValue))
}

pub fn parse_string<T: std::io::Read> (mut parser: &mut EventReader<T>, elem_start: ElemStart)
        -> Result<String, Error> {
    parse_chars(parser,
                elem_start,
                |chars| String::from_str(chars).map_err(Error::ParseValue))
}


struct GpxElemParser<'a, T: 'a + Read> {
    reader: &'a mut EventReader<T>,
    name: Option<OwnedName>,
    metadata: Option<Metadata>,
    waypoints: Vec<Waypoint>,
    tracks: Vec<Track>,
    version: Option<GpxVersion>,
    creator: Option<String>,
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

impl<'a, T: Read> ElementParse<'a, T> for GpxElemParser<'a, T> {
    fn new(reader: &'a mut EventReader<T>) -> GpxElemParser<'a, T> {
        GpxElemParser { reader: reader,
                        name: None,
                        metadata: None,
                        tracks: Vec::new(),
                        waypoints: Vec::new(),
                        version: None,
                        creator: None }
    }
    fn parse_start(&mut self, elem_start: ElemStart) -> Result<(), Self::Error> {
        for attr in elem_start.attributes {
            match &(attr.name.local_name) as &str {
                "version" => {
                    self.version = Some(match &(attr.value) as &str {
                        "1.0" => GpxVersion::V1_0,
                        "1.1" => GpxVersion::V1_1,
                        _ => { return Err(Error::Str("Unknown GPX version")); }
                    });
                },
                "creator" => {
                    self.creator = Some(attr.value);
                },
                _ => {}
            }
        }
        self.name = Some(elem_start.name);
        Ok(())
    }
    fn next(&mut self) -> Result<XmlEvent, self::xml::Error> {
        self.reader.next().map_err(self::xml::Error::Xml)
    }
    fn get_name(&self) -> &OwnedName {
        match &self.name {
            &Some(ref i) => i,
            &None => unreachable!(),
        }
    }
    fn parse_element(&mut self, elem_start: ElemStart) -> Result<(), Error> {
        match &(elem_start.name.local_name) as &str {
            "metadata" => {
                self.metadata = Some(
                    try!(MetadataParser::new(self.reader).parse(elem_start))
                );
            }
            "wpt" => {
                self.waypoints.push(
                    try!(WptParser::new(self.reader).parse(elem_start))
                );
            }
            "trk" => {
                self.tracks.push(
                    try!(TrkParser::new(self.reader).parse(elem_start))
                );
            }
            _ => {
                try!(ElementParser::new(self.reader).parse(elem_start));
            }
        }
        Ok(())
    }
}


struct MetadataParser<'a, T: 'a + Read> {
    reader: &'a mut EventReader<T>,
    elem_name: Option<OwnedName>,
    name: Option<String>,
    desc: Option<String>,
    author: Option<XmlElement>,
    copyright: Option<XmlElement>,
    link: Vec<XmlElement>,
    time: Option<Time>,
    keywords: Option<String>,
    bounds: Option<XmlElement>,
    extensions: Option<XmlElement>,  
}
/*
macro_rules! ElementParseImpl {
    ( $( $s: ident ), $( $element: ident ), $( $error: ident), $(
}*/
impl<'a, T: Read> ElementBuild for MetadataParser<'a, T> {
    type Element = Metadata;
    type Error = Error;
    fn build(self) -> Result<Self::Element, Self::Error> {
        Ok(Metadata { name: self.name,
                      desc: self.desc,
                      author: self.author,
                      copyright: self.copyright,
                      link: self.link,
                      time: self.time,
                      keywords: self.keywords,
                      bounds: self.bounds,
                      extensions: self.extensions })
    }
}

impl<'a, T: Read> ElementParse<'a, T> for MetadataParser<'a, T> {
    fn new(reader: &'a mut EventReader<T>) -> Self {
        MetadataParser { reader: reader,
                         elem_name: None,
                         name: None,
                         desc: None,
                         author: None,
                         copyright: None,
                         link: Vec::new(),
                         time: None,
                         keywords: None,
                         bounds: None,
                         extensions: None }
    }

    ParserStart!();

    fn parse_element(&mut self, elem_start: ElemStart)
            -> Result<(), Self::Error> {
        match &elem_start.name.local_name as &str {
            "time" => {
                self.time = Some(try!(parse_time::<T, Self::Error>(self.reader, elem_start)));
            }
            "name" => {
                self.name = Some(try!(parse_string(self.reader, elem_start)));
            }
            _ => {
                try!(ElementParser::new(self.reader).parse(elem_start));
            }
        }
        Ok(())
    }

    fn get_name(&self) -> &OwnedName {
        match &self.elem_name {
            &Some(ref i) => i,
            &None => unreachable!(),
        }
    }
    fn next(&mut self) -> Result<XmlEvent, self::xml::Error> {
        self.reader.next().map_err(self::xml::Error::Xml)
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
