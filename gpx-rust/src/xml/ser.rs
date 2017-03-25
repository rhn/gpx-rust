/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate xml as _xml;

use std::io::Write;
use std::borrow::Cow;

use self::_xml::writer::{ EmitterConfig, EventWriter, XmlEvent };

use ser::{ SerializeVia, Error };

use xml;
use xml::conv;

/// Special handling of namespaces
impl SerializeVia<xml::Element> for conv::Element {
    fn serialize_via<W: Write>(data: &xml::Element, sink: &mut EventWriter<W>, name: &str) 
            -> Result<(), Error> {
        try!(sink.write(
            XmlEvent::StartElement { name: data.name.borrow(),
                                     attributes: Cow::Borrowed(
                                         data.attributes
                                             .iter()
                                             .map(|a| { a.borrow() })
                                             .collect::<Vec<_>>()
                                             .as_slice()),
                                     namespace: Cow::Borrowed(&data.namespace) }
        ));
        for node in &data.nodes {
            try!(match node {
                &xml::Node::Text(ref s) => {
                    sink.write(XmlEvent::Characters(s)).map_err(Error::from)
                },
                &xml::Node::Element(ref e) => conv::Element::serialize_via(e, sink, name),
            });
        }
        try!(sink.write(XmlEvent::EndElement { name: Some(data.name.borrow()) }));
        Ok(())
    }
}
