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
use gpx::{ Gpx, GpxVersion, Metadata, Waypoint, Fix, Track, TrackSegment, Bounds };
use gpx::conv::{ Latitude, Longitude };
use gpx::conv;
use ser::{ SerError, Serialize, SerializeVia, SerializeAttr, SerializeCharElem };

const GPX_NS: &'static str = "http://www.topografix.com/GPX/1/1";


macro_rules! set_optional(
    ($sink:ident, $name:expr, $tag:expr) => {
        if let Some(ref item) = $name {
            try!(item.serialize_with($sink, $tag));
        }
    }
);

#[derive(Debug)]
pub enum AttributeValueError {
    LatitudeOutOfBounds(f64)
}

trait ToAttributeVia<Data> {
    fn to_attribute(&Data) -> Result<String, AttributeValueError>;
}

impl ToAttributeVia<f64> for Latitude {
    fn to_attribute(data: &f64) -> Result<String, AttributeValueError> {
        if *data >= 90.0 || *data < -90.0 {
            Err(AttributeValueError::LatitudeOutOfBounds(*data))
        } else {
            Ok(data.to_string())
        }
    }
}

impl ToAttributeVia<f64> for Longitude {
    fn to_attribute(data: &f64) -> Result<String, AttributeValueError> {
        if *data >= 180.0 || *data < -180.0 {
            Err(AttributeValueError::LatitudeOutOfBounds(*data))
        } else {
            Ok(data.to_string())
        }
    }
}

/// via XSD string type
/*impl SerializeVia<String> for XsdString {
    fn serialize_via<W: io::Write>(data: &String, sink: &mut EventWriter<W>, name: &str)
            -> Result<(), SerError> {
        try!(sink.write(XmlEvent::start_element(name)));
        try!(sink.write(XmlEvent::characters(data)));
        try!(sink.write(XmlEvent::EndElement { name: Some(name.into()) }));
        Ok(())
    }
}*/

impl SerializeVia<Bounds> for conv::Bounds {
    fn serialize_via<W: io::Write>(data: &Bounds, sink: &mut EventWriter<W>, name: &str)
            -> Result<(), SerError> {
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

impl Serialize for Gpx {
    fn serialize_with<W: io::Write>(&self, sink: &mut EventWriter<W>, name: &str)
            -> Result<(), SerError> {
        try!(sink.write(XmlEvent::StartDocument { version: XmlVersion::Version11,
                                                  encoding: None,
                                                  standalone: None }));
        let elemname = Name::local(name);
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
            try!(::gpx::conv::Metadata::serialize_via(meta, sink, "metadata"));
        }
        for item in &self.waypoints {
            try!(item.serialize_with(sink, "wpt"));
        }
        for item in &self.routes {
            try!(::gpx::conv::Rte::serialize_via(item, sink, "rte"));
        }
        for item in &self.tracks {
            try!(::gpx::conv::Trk::serialize_via(item, sink, "trk"));
        }
        
        try!(sink.write(XmlEvent::EndElement { name: Some(elemname) }));
        Ok(())
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
    fn serialize_with<W: io::Write>(&self, sink: &mut EventWriter<W>, name: &str)
            -> Result<(), SerError> {
        let elemname = Name::local(name);
        let lat = try!(Latitude::to_attribute(&self.location.latitude)
            .map_err(|e| SerError::ElementAttributeError("latitude", e)));
        let lon = try!(Longitude::to_attribute(&self.location.longitude)
            .map_err(|e| SerError::ElementAttributeError("longitude", e)));
        try!(sink.write(XmlEvent::StartElement {
            name: elemname.clone(),
            attributes: Cow::Owned(
                    vec![Attribute { name: Name::local("lat"),
                                     value: &lat },
                         Attribute { name: Name::local("lon"),
                                     value: &lon }]),
            namespace: Cow::Owned(Namespace::empty()),
        }));
        if let Some(ref item) = self.location.elevation {
            try!(item.serialize_with(sink, "ele"));
        }
        if let Some(ref item) = self.time {
            try!(item.serialize_with(sink, "time"));
        }
        set_optional!(sink, self.mag_variation, "magvar");
        set_optional!(sink, self.geoid_height, "geoidheight");
        set_optional!(sink, self.name, "name");
        set_optional!(sink, self.comment, "cmt");
        set_optional!(sink, self.description, "desc");
        set_optional!(sink, self.source, "src");
        for item in &self.links {
            try!(item.serialize_with(sink, "link"));
        }
        set_optional!(sink, self.symbol, "symbol");
        set_optional!(sink, self.type_, "type");
        set_optional!(sink, self.fix, "fix");
        set_optional!(sink, self.satellites, "sat");
        set_optional!(sink, self.hdop, "hdop");
        set_optional!(sink, self.vdop, "vdop");
        set_optional!(sink, self.pdop, "pdop");
        set_optional!(sink, self.dgps_age, "ageofdgpsdata");
        set_optional!(sink, self.dgps_id, "dgpsid");
        set_optional!(sink, self.extensions, "extensions");
        try!(sink.write(XmlEvent::EndElement { name: Some(elemname) }));
        Ok(())
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
