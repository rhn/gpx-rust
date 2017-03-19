/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Serialization impls for GPX types

extern crate xml as _xml;

use std::fmt;
use std::io;
use std::borrow::Cow;
use std::error::Error as ErrorTrait;
use self::_xml::name::Name;
use self::_xml::namespace::{ Namespace, NS_NO_PREFIX };
use self::_xml::attribute::Attribute;
use self::_xml::writer::{ XmlEvent, EventWriter };

use xsd;
use gpx::{ Gpx, Version, Waypoint, Fix, Bounds };
use gpx::conv::{ Latitude, Longitude };
use gpx::conv;
use ser;
use ser::FormatError;
use ser::{ SerializeDocument, SerializeVia, SerializeCharElemVia, ToAttributeVia };

const GPX_NS: &'static str = "http://www.topografix.com/GPX/1/1";


macro_rules! set_optional(
    ($sink:ident, $name:expr, $tag:expr, $type_:path) => {
        if let Some(ref item) = $name {
            try!(<$type_>::serialize_via(item, $sink, $tag));
        }
    }
);

#[derive(Debug)]
pub enum Error {
    InvalidEmail,
    DecimalOutOfBounds(f64)
}

impl FormatError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Debug::fmt(self, fmt)
    }
}

impl ErrorTrait for Error {
    fn description(&self) -> &str {
        match *self {
            Error::InvalidEmail => "Email must contain exactly one @ sign",
            Error::DecimalOutOfBounds(_) => "Decimal value is outside of allowed range",
        }
    }
}

impl ToAttributeVia<f64> for Latitude {
    fn to_attribute(data: &f64) -> Result<String, Box<FormatError>> {
        if *data >= 90.0 || *data < -90.0 {
            Err(Box::new(Error::DecimalOutOfBounds(*data)) as Box<FormatError>)
        } else {
            Ok(data.to_string())
        }
    }
}

impl ToAttributeVia<f64> for Longitude {
    fn to_attribute(data: &f64) -> Result<String, Box<FormatError>> {
        if *data >= 180.0 || *data < -180.0 {
            Err(Box::new(Error::DecimalOutOfBounds(*data)) as Box<FormatError>)
        } else {
            Ok(data.to_string())
        }
    }
}

impl SerializeVia<Bounds> for conv::Bounds {
    fn serialize_via<W: io::Write>(data: &Bounds, sink: &mut EventWriter<W>, name: &str)
            -> Result<(), ser::Error> {
        let name = Name::local(name);
        try!(sink.write(
            XmlEvent::StartElement {
                name: name,
                attributes: Cow::Owned(
                // FIXME: turn to_string() into Latitude/Longitude conv
                    vec![Attribute { name: Name::local("minlat"),
                                     value: &data.xmin.to_string() },
                         Attribute { name: Name::local("minlon"),
                                     value: &data.ymin.to_string() },
                         Attribute { name: Name::local("maxlat"),
                                     value: &data.xmax.to_string() },
                         Attribute { name: Name::local("maxlon"),
                                     value: &data.ymax.to_string() }]
                ),
                namespace: Cow::Owned(Namespace::empty())
            }
        ));
        try!(sink.write(XmlEvent::EndElement { name: Some(name) }));    
        Ok(())
    }
}

impl SerializeDocument for Gpx {
    fn serialize_root<W: io::Write>(&self, sink: &mut EventWriter<W>)
            -> Result<(), ser::Error> {
        conv::Gpx::serialize_via(self, sink, "gpx")
    }
}

/// Gpx needs custom serialization because it needs to carry the GPX namespace and version number
impl SerializeVia<Gpx> for conv::Gpx {
    fn serialize_via<W: io::Write>(data: &Gpx, sink: &mut EventWriter<W>, name: &str)
            -> Result<(), ser:: Error> {
        let elemname = Name::local(name);
        let mut ns = Namespace::empty();
        ns.put(NS_NO_PREFIX, GPX_NS);
        let ns = ns;
        try!(sink.write(
            XmlEvent::StartElement {
                name: elemname.clone(),
                attributes: Cow::Owned(
                    vec![Attribute { name: Name::local("version"),
                                     value: Version::V1_1.to_attribute() },
                         Attribute { name: Name::local("creator"),
                                     value: &data.creator }]
                ),
                namespace: Cow::Owned(ns)
            }
        ));
        if let Some(ref meta) = data.metadata {
            try!(::gpx::conv::Metadata::serialize_via(meta, sink, "metadata"));
        }
        for item in &data.waypoints {
            try!(::gpx::conv::Wpt::serialize_via(item, sink, "wpt"));
        }
        for item in &data.routes {
            try!(::gpx::conv::Rte::serialize_via(item, sink, "rte"));
        }
        for item in &data.tracks {
            try!(::gpx::conv::Trk::serialize_via(item, sink, "trk"));
        }
        if let Some(ref ext) = data.extensions {
            try!(::gpx::conv::Extensions::serialize_via(ext, sink, "extensions"));
        }
        try!(sink.write(XmlEvent::EndElement { name: Some(elemname) }));
        Ok(())
    }
}

impl SerializeVia<String> for conv::Email {
    fn serialize_via<W: io::Write>(data: &String, sink: &mut EventWriter<W>, name: &str)
           -> Result<(), ser::Error> {
        let split = data.split("@").collect::<Vec<_>>();
        if split.len() != 2 {
            return Err(ser::Error::Value(Box::new(Error::InvalidEmail)));
        }
        let (id, domain) = (split[0], split[1]);
        
        let elemname = Name::local(name);
        try!(sink.write(XmlEvent::StartElement {
            name: elemname.clone(),
            attributes: Cow::Owned(Vec::new()),
            namespace: Cow::Owned(Namespace::empty()),
        }));

        try!(::xsd::conv::String::serialize_via(id, sink, "id"));
        try!(::xsd::conv::String::serialize_via(domain, sink, "domain"));

        try!(sink.write(XmlEvent::EndElement { name: Some(elemname) }));
        Ok(())
    }
}


impl Version {
    fn to_attribute(&self) -> &'static str {
        match self {
            &Version::V1_0 => "1.0",
            &Version::V1_1 => "1.1",
        }
    }
}

/// Custom serialization beeded because of the location field
impl SerializeVia<Waypoint> for conv::Wpt {
    fn serialize_via<W: io::Write>(data: &Waypoint, sink: &mut EventWriter<W>, name: &str)
            -> Result<(), ser::Error> {
        let elemname = Name::local(name);
        let lat = try!(Latitude::to_attribute(&data.location.latitude)
            .map_err(ser::Error::Value));
        let lon = try!(Longitude::to_attribute(&data.location.longitude)
            .map_err(ser::Error::Value));
        try!(sink.write(XmlEvent::StartElement {
            name: elemname.clone(),
            attributes: Cow::Owned(
                    vec![Attribute { name: Name::local("lat"),
                                     value: &lat },
                         Attribute { name: Name::local("lon"),
                                     value: &lon }]),
            namespace: Cow::Owned(Namespace::empty()),
        }));
        if let Some(ref item) = data.location.elevation {
            try!(xsd::conv::Decimal::serialize_via(item, sink, "ele"));
        }
        set_optional!(sink, data.time, "time", xsd::conv::DateTime);
        if let Some(ref item) = data.mag_variation {
            try!(xsd::conv::Decimal::serialize_via(item, sink, "magvar"));
        }
        if let Some(ref item) = data.geoid_height {
            try!(xsd::conv::Decimal::serialize_via(item, sink, "magvar"));
        }
        set_optional!(sink, data.name, "name", xsd::conv::String);
        set_optional!(sink, data.comment, "cmt", xsd::conv::String);
        set_optional!(sink, data.description, "desc", xsd::conv::String);
        set_optional!(sink, data.source, "src", xsd::conv::String);
        for item in &data.links {
            try!(conv::Link::serialize_via(item, sink, "link"));
        }
        set_optional!(sink, data.symbol, "sym", xsd::conv::String);
        set_optional!(sink, data.type_, "type", xsd::conv::String);
        set_optional!(sink, data.fix, "fix", conv::Fix);
        set_optional!(sink, data.satellites, "sat", xsd::conv::NonNegativeInteger);
        set_optional!(sink, data.hdop, "hdop", xsd::conv::Decimal);
        set_optional!(sink, data.vdop, "vdop", xsd::conv::Decimal);
        set_optional!(sink, data.pdop, "pdop", xsd::conv::Decimal);
        set_optional!(sink, data.dgps_age, "ageofdgpsdata", xsd::conv::Decimal);
        set_optional!(sink, data.dgps_id, "dgpsid", conv::DgpsStation);
        set_optional!(sink, data.extensions, "extensions", conv::Extensions);
        try!(sink.write(XmlEvent::EndElement { name: Some(elemname) }));
        Ok(())
    }
}

impl SerializeCharElemVia<Fix> for conv::Fix {
    fn to_characters(data: &Fix) -> String {
        match data {
            &Fix::None => "none",
            &Fix::_2D => "2d",
            &Fix::_3D => "3d",
            &Fix::DGPS => "dgps",
            &Fix::PPS => "pps"
        }.into()
    }
}


impl SerializeCharElemVia<u16> for conv::DgpsStation {
    #[allow(unused_comparisons)]
    fn to_characters(data: &u16) -> String {
        if 0 > *data {
            /*Err(::gpx::par::Error::TooSmall {
                limit: 0.into(),
                value: value.into(),
            })*/
            panic!("Too small");
        } else if *data > 1024 {
            /*Err(::gpx::par::Error::TooLarge {
                limit: 1024.into(),
                value: value.into(),
            })*/
            panic!("Too large");
        } else {
            <::xsd::conv::Integer as SerializeCharElemVia<u16>>::to_characters(data)
        }
    }
}
