extern crate xml as _xml;

use std;
use std::io::Read;
use std::str::FromStr;
use std::error::Error as StdError;
use self::_xml::common::Position;
use self::_xml::name::OwnedName;
use self::_xml::reader::{ XmlEvent, EventReader };

use xml;
use xml::{ ElemStart, ElementParser, ElementParse, ElementBuild };
use gpx::{ Error, ElementError, Bounds, GpxVersion };
use gpx::ser;
use gpx::conv::{ Latitude, Longitude };
use ::par::ParserMessage;


/// Raise whenever attribute value is out of bounds
#[derive(Debug)]
pub enum AttributeValueError {
    Str(&'static str),
    Error(Box<std::error::Error>),
}

pub trait FromAttribute<T> {
    fn from_attr(&str) -> Result<T, AttributeValueError>;
}

impl FromAttribute<f64> for Latitude {
    fn from_attr(attr: &str) -> Result<f64, AttributeValueError> {
        f64::from_str(attr).map_err(|e| { AttributeValueError::Error(Box::new(e)) })
    }
}


pub struct BoundsParser<'a, T: 'a + Read> {
    reader: &'a mut EventReader<T>,
    elem_name: Option<OwnedName>,
    minlat: Option<f64>,
    minlon: Option<f64>,
    maxlat: Option<f64>,
    maxlon: Option<f64>,
}

impl<'a, T: Read> ElementParse<'a, T> for BoundsParser<'a, T> {
    fn new(reader: &'a mut EventReader<T>) -> Self {
        BoundsParser {
            reader: reader,
            elem_name: None,
            minlat: None,
            minlon: None,
            maxlat: None,
            maxlon: None,
        }
    }
    ParserStart!(
        "minlat" => { minlat , Latitude::from_attr },
        "minlon" => { minlon , Longitude::from_attr },
        "maxlat" => { maxlat , Latitude::from_attr },
        "maxlon" => { maxlon , Longitude::from_attr }
    );
    _ParserImplBody!(
        tags: {}
    );
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

pub fn parse_gpx_version(value: &str) -> Result<GpxVersion, AttributeValueError> {
    match value {
        "1.0" => Ok(GpxVersion::V1_0),
        "1.1" => Ok(GpxVersion::V1_1),
        _ => Err(AttributeValueError::Str("Unknown GPX version"))
    }
}

pub fn copy(value: &str) -> Result<String, AttributeValueError> {
    Ok(value.into())
}
