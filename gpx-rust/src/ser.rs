extern crate xml as _xml;

use std::io;
use std::borrow::Cow;
use self::_xml::writer;
use self::_xml::writer::{ EmitterConfig, EventWriter, XmlEvent };

use xml;


pub trait Serialize {
    fn serialize<W: io::Write>(&self, sink: W) -> Result<(), io::Error> {
        let mut xw = EmitterConfig::new()
            .line_separator("\n")
            .perform_indent(true)
            .create_writer(sink);
        
        match self.serialize_with(&mut xw) {
            Err(writer::Error::Io(e)) => { Err(e) },
            Err(e) => panic!(format!("Bug: {:?}", e)),
            _ => Ok(())
        }
    }
    fn serialize_with<W: io::Write>(&self, sink: &mut EventWriter<W>) -> writer::Result<()>;
}

impl Serialize for xml::XmlElement {
    fn serialize_with<W: io::Write>(&self, sink: &mut EventWriter<W>) -> writer::Result<()> {
        try!(sink.write(
            XmlEvent::StartElement { name: self.name.borrow(),
                                     attributes: Cow::Borrowed(
                                        self.attributes
                                            .iter()
                                            .map(|a| { a.borrow() })
                                            .collect::<Vec<_>>()
                                            .as_slice()),
                                     namespace: Cow::Borrowed(&self.namespace) }
        ));
        for node in &self.nodes {
            try!(match node {
                &xml::XmlNode::Text(ref s) => sink.write(XmlEvent::Characters(s)),
                &xml::XmlNode::Element(ref e) => e.serialize_with(sink),
            })
        }
        sink.write(XmlEvent::EndElement { name: Some(self.name.borrow()) })
    }
}
