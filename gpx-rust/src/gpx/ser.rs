//! Serialization impls for GPX types

extern crate fringe;
extern crate xml as _xml;

use std::io;
use std::borrow::Cow;
use self::_xml::common::XmlVersion;
use self::_xml::name::Name;
use self::_xml::namespace::Namespace;
use self::_xml::attribute::Attribute;
use self::_xml::writer;
use self::_xml::writer::{ XmlEvent, EventWriter };
use gpx::{ Gpx, GpxVersion, Metadata };
use ser::Serialize;


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

impl Serialize for Metadata {
    fn serialize_with<W: io::Write>(&self, sink: &mut EventWriter<W>, name: &str) -> writer::Result<()> {
        let elemname = Name::local(name);
        try!(sink.write(XmlEvent::StartElement {
            name: elemname.clone(),
            attributes: Cow::Owned(vec![]),
            namespace: Cow::Owned(Namespace::empty()),
        }));
        
        if let Some(ref item) = self.name {
            try!(item.serialize_with(sink, "name"));
        }
        if let Some(ref item) = self.desc {
            try!(item.serialize_with(sink, "desc"));
        }
        if let Some(ref item) = self.author {
            try!(item.serialize_with(sink, "author"));
        }
        if let Some(ref item) = self.copyright {
            try!(item.serialize_with(sink, "copyright"));
        }
        for item in &self.links {
            try!(item.serialize_with(sink, "link"));
        }
        if let Some(ref item) = self.time {
            try!(item.serialize_with(sink, "time"));
        }
        if let Some(ref item) = self.keywords {
            try!(item.serialize_with(sink, "keywords"));
        }
        if let Some(ref item) = self.bounds {
            try!(item.serialize_with(sink, "bounds"));
        }
        if let Some(ref item) = self.extensions {
            try!(item.serialize_with(sink, "extensions"));
        }
        sink.write(XmlEvent::EndElement { name: Some(elemname) })
    }
}
