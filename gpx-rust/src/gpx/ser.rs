//! Serialization impls for GPX types

extern crate fringe;

use std::borrow::Cow;
use generator::{Generator, make_gen};
use gpx::*;


impl Gpx {
    pub fn serialize<W: std::io::Write>(&self, sink: W) -> Result<(), io::Error> {
        let mut xw = _xml::writer::EmitterConfig::new()
            .line_separator("\n")
            .perform_indent(true)
            .create_writer(sink);
        for ev in self.events() {
            match xw.write(ev) {
                Err(_xml::writer::Error::Io(e)) => { return Err(e) },
                Err(e) => panic!(format!("Programming error: {:?}", e)),
                _ => ()
            }
        }
        Ok(())
    }
    fn events<'a>(&'a self) -> Generator<_xml::writer::XmlEvent<'a>> {
        make_gen(move |ctx| {
            ctx.suspend(_xml::writer::XmlEvent::StartDocument { version: _xml::common::XmlVersion::Version11,
                                                  encoding: None,
                                                  standalone: None });
            let elemname = _xml::name::Name::local("gpx");
            let gpxver = GpxVersion::V1_1;
            let ver = gpxver.to_attribute();
            let attrs = vec![_xml::attribute::Attribute { name: _xml::name::Name::local("version"),
                                                          value: ver },
                             _xml::attribute::Attribute { name: _xml::name::Name::local("creator"),
                                                          value: &self.creator },];
            ctx.suspend(_xml::writer::XmlEvent::StartElement { name: elemname.clone(),
                                                               attributes: Cow::Owned(attrs),
                                                               namespace: Cow::Owned(_xml::namespace::Namespace::empty()) });
            ctx.suspend(_xml::writer::XmlEvent::EndElement { name: Some(elemname) });
        })
    }
}
