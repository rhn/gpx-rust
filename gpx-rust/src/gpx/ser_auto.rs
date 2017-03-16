extern crate xml as _xml;

use std::borrow::Cow;

use self::_xml::attribute::{ Attribute, OwnedAttribute };
use self::_xml::name::{ Name, OwnedName };
use self::_xml::namespace::Namespace;
use self::_xml::writer::{ XmlEvent, EventWriter };

use ser::{ Serialize, SerError, SerializeVia, ToAttributeVia };
use gpx::*;

include!(concat!(env!("OUT_DIR"), "/gpx_ser_auto.rs"));
