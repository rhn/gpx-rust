extern crate xml as _xml;

use std::io;
use self::_xml::writer;
use self::_xml::writer::{ EmitterConfig, XmlEvent };
use generator::Generator;


pub trait Serialize {
    fn serialize<W: io::Write>(&self, sink: W) -> Result<(), io::Error> {
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
    fn events<'a>(&'a self) -> Generator<XmlEvent<'a>>;
}
