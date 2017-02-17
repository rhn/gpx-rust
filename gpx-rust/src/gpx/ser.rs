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
use self::_xml::writer::{ EmitterConfig, XmlEvent };
use generator::{ Generator, make_gen };
use gpx::{ Gpx, GpxVersion };


impl Gpx {
    pub fn serialize<W: io::Write>(&self, sink: W) -> Result<(), io::Error> {
        let mut xw = EmitterConfig::new()
            .line_separator("\n")
            .perform_indent(true)
            .create_writer(sink);
        for ev in self.events() {
            match xw.write(ev) {
                Err(writer::Error::Io(e)) => { return Err(e) },
                Err(e) => panic!(format!("Programming error: {:?}", e)),
                _ => ()
            }
        }
        Ok(())
    }
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
            ctx.suspend(XmlEvent::EndElement { name: Some(elemname) });
        })
    }
}
