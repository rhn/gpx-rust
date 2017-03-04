use self::_xml::attribute::{ Attribute, OwnedAttribute };
use self::_xml::name::OwnedName;

use gpx::ser::ToAttributeVia;

include!(concat!(env!("OUT_DIR"), "/gpx_ser_auto.rs"));
