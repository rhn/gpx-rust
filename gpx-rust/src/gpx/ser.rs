//! Serialization impls for GPX types

extern crate xml as _xml;

use std::io;
use std::borrow::Cow;
use self::_xml::common::XmlVersion;
use self::_xml::name::Name;
use self::_xml::namespace::{ Namespace, NS_NO_PREFIX };
use self::_xml::attribute::Attribute;
use self::_xml::writer;
use self::_xml::writer::{ XmlEvent, EventWriter };
use gpx::{ Gpx, GpxVersion, Metadata, Waypoint, Fix, Track, TrackSegment };
use ser::{ Serialize, SerializeAttr, SerializeCharElem };


const GPX_NS: &'static str = "http://www.topografix.com/GPX/1/1";


macro_rules! set_optional(
    ($sink:ident, $name:expr, $tag:expr) => {
        if let Some(ref item) = $name {
            try!(item.serialize_with($sink, $tag));
        }
    }
);

impl Serialize for Gpx {
    fn serialize_with<W: io::Write>(&self, sink: &mut EventWriter<W>, name: &str) -> writer::Result<()> {
        try!(sink.write(XmlEvent::StartDocument { version: XmlVersion::Version11,
                                                  encoding: None,
                                                  standalone: None }));
        let elemname = Name::local("gpx");
        let mut ns = Namespace::empty();
        ns.put(NS_NO_PREFIX, GPX_NS);
        let ns = ns;
        try!(sink.write(
            XmlEvent::StartElement {
                name: elemname.clone(),
                attributes: Cow::Owned(
                    vec![Attribute { name: Name::local("version"),
                                     value: GpxVersion::V1_1.to_attribute() },
                         Attribute { name: Name::local("creator"),
                                     value: &self.creator }]
                ),
                namespace: Cow::Owned(ns)
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
        // set_optional!(sink, self.mag_variation, "magvar");
        /*if let Some(ref item) = self.geoid_height {
            try!(item.serialize_with(sink, "geoidheight"));
        }*/
        set_optional!(sink, self.name, "name");
        // set_optional!(sink, self.comment, "cmt");
        // set_optional!(sink, self.description, "desc");
        // set_optional!(sink, self.source, "src");
        // set_optional!(sink, self.link, "link");
        // set_optional!(sink, self.symbol, "symbol");
        // set_optional!(sink, self.type_, "type");
        set_optional!(sink, self.fix, "fix");
        set_optional!(sink, self.satellites, "sat");
        // set_optional!(sink, self.hdop, "hdop");
        // set_optional!(sink, self.vdop, "vdop");
        // set_optional!(sink, self.pdop, "pdop");
        // set_optional!(sink, self.dgps_age, "ageofdgpsdata");
        // set_optional!(sink, self.dgps_id, "dgpsid");
        set_optional!(sink, self.extensions, "extensions");
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
