//! Serialization impls for GPX types

extern crate fringe;
extern crate xml as _xml;

use std::borrow::Cow;
use self::_xml::common::XmlVersion;
use self::_xml::name::Name;
use self::_xml::namespace::Namespace;
use self::_xml::attribute::Attribute;
use self::_xml::writer::XmlEvent;
use generator::{ Generator, make_gen };
use gpx::{ Gpx, GpxVersion, Metadata };
use ser::Serialize;


impl Serialize for Gpx {
    fn events<'a>(&'a self) -> Generator<XmlEvent<'a>> {
        make_gen(move |ctx| {
            ctx.suspend(XmlEvent::StartDocument { version: XmlVersion::Version11,
                                                  encoding: None,
                                                  standalone: None });
            let elemname = Name::local("gpx");
            ctx.suspend(
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
            );
            
            if let Some(ref meta) = self.metadata {
                for ev in meta.events() {
                    ctx.suspend(ev);
                }
            }
            ctx.suspend(XmlEvent::EndElement { name: Some(elemname) });
        })
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
    fn events<'a>(&'a self) -> Generator<XmlEvent<'a>> {
        make_gen(move |ctx| {
            let elemname = Name::local("metadata");
            ctx.suspend(XmlEvent::StartElement {
                name: elemname.clone(),
                attributes: Cow::Owned(vec![]),
                namespace: Cow::Owned(Namespace::empty()),
            });
            /*
            if let Some(ref item) = self.name {
                for ev in item.events() {
                    ctx.suspend(ev);
                }
            }
            if let Some(ref item) = self.desc {
                for ev in item.events() {
                    ctx.suspend(ev);
                }
            }
            if let Some(ref item) = self.author {
                for ev in item.events() {
                    ctx.suspend(ev);
                }
            }
            if let Some(ref item) = self.copyright {
                for ev in item.events() {
                    ctx.suspend(ev);
                }
            }
            for item in self.links {
                for ev in item.events() {
                    ctx.suspend(ev);
                }
            }*/
            if let Some(ref item) = self.time {
                for ev in item.events() {
                    ctx.suspend(ev);
                }
            }/*
            if let Some(ref item) = self.keywords {
                for ev in item.events() {
                    ctx.suspend(ev);
                }
            }
            if let Some(ref item) = self.bounds {
                for ev in item.events() {
                    ctx.suspend(ev);
                }
            }
            if let Some(ref item) = self.extensions {
                for ev in item.events() {
                    ctx.suspend(ev);
                }
            }
            */
            ctx.suspend(XmlEvent::EndElement { name: Some(elemname) });
        })
    }
}
