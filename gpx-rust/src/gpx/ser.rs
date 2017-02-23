//! Serialization impls for GPX types

extern crate xml as _xml;

use std::io;
use std::borrow::Cow;
use self::_xml::common::XmlVersion;
use self::_xml::name::Name;
use self::_xml::namespace::Namespace;
use self::_xml::attribute::Attribute;
use self::_xml::writer;
use self::_xml::writer::{ XmlEvent, EventWriter };
use gpx::{ Gpx, GpxVersion, Metadata, Waypoint, Fix, Track, TrackSegment };
use ser::{ Serialize, SerializeAttr, SerializeCharElem };


impl Serialize for Gpx {
    fn serialize_with<W: io::Write>(&self, sink: &mut EventWriter<W>, name: &str) -> writer::Result<()> {
        try!(sink.write(XmlEvent::StartDocument { version: XmlVersion::Version11,
                                                  encoding: None,
                                                  standalone: None }));
        let elemname = Name::local("gpx");
        try!(sink.write(
            XmlEvent::StartElement {
                name: elemname.clone(),
                attributes: Cow::Owned(
                    vec![Attribute { name: Name::local("version"),
                                     value: GpxVersion::V1_1.to_attribute() },
                         Attribute { name: Name::local("creator"),
                                     value: &self.creator }]
                ),
                namespace: Cow::Owned(Namespace::empty())
            }
        ));
        if let Some(ref meta) = self.metadata {
            try!(meta.serialize_with(sink, "metadata"));
        }
        for item in &self.waypoints {
            try!(item.serialize_with(sink, "wpt"));
        }
        for item in &self.tracks {
            try!(item.serialize_with(sink, "trk"));
        }
        
        sink.write(XmlEvent::EndElement { name: Some(elemname) })
    }
}


impl GpxVersion {
    fn to_attribute(&self) -> &'static str {
        match self {
            &GpxVersion::V1_0 => "1.0",//String::from("1.0"),
            &GpxVersion::V1_1 => "1.1",//String::from("1.1")
        }
    }
}

impl Serialize for Waypoint {
    fn serialize_with<W: io::Write>(&self, sink: &mut EventWriter<W>, name: &str) -> writer::Result<()> {
        let elemname = Name::local(name);
        try!(sink.write(XmlEvent::StartElement {
            name: elemname.clone(),
            attributes: Cow::Owned(
                    vec![Attribute { name: Name::local("lat"),
                                     value: self.location.latitude.to_attribute() },
                         Attribute { name: Name::local("lon"),
                                     value: self.location.longitude.to_attribute() }]),
            namespace: Cow::Owned(Namespace::empty()),
        }));
        if let Some(ref item) = self.location.elevation {
            try!(item.serialize_with(sink, "ele"));
        }
        if let Some(ref item) = self.time {
            try!(item.serialize_with(sink, "time"));
        }
        if let Some(ref item) = self.fix {
            try!(item.serialize_with(sink, "fix"));
        }
        if let Some(ref item) = self.satellites {
            try!(item.serialize_with(sink, "sat"));
        }
        if let Some(ref item) = self.name {
            try!(item.serialize_with(sink, "name"));
        }
        sink.write(XmlEvent::EndElement { name: Some(elemname) })
    }
}

impl SerializeCharElem for Fix {
    fn to_characters(&self) -> String {
        match self {
            &Fix::None => "none",
            &Fix::_2D => "2d",
            &Fix::_3D => "3d",
            &Fix::DGPS => "dgps",
            &Fix::PPS => "pps"
        }.into()
    }
}
