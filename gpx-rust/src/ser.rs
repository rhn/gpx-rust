/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Serialization procedures for turning arbitrary data into XML documents
extern crate xml as _xml;

use std::fmt;
use std::io;
use std::borrow::Cow;

use self::_xml::common::XmlVersion;
use self::_xml::name::Name;
use self::_xml::namespace::Namespace;
use self::_xml::writer;
use self::_xml::writer::{ EmitterConfig, EventWriter, XmlEvent };

use conv;
use xml;

    
/// Value cannot be formatted to a valid string
pub trait FormatError where Self: fmt::Debug + 'static {}

/// Problems encountered while serializing
#[derive(Debug)]
pub enum Error {
    /// I/O and programming problems
    Writer(writer::Error),
    Value(Box<FormatError>), // TODO: save location and generalize beyond string
}

impl From<writer::Error> for Error {
    fn from(e: writer::Error) -> Self {
        Error::Writer(e)
    }
}

impl<E: FormatError + 'static> From<E> for Error {
    fn from(err: E) -> Self {
        Error::Value(Box::new(err) as Box<FormatError>)
    }
}

/// Serializes XML documents
pub trait SerializeDocument {
    /// Default serialization, pretty prints the XML file
    fn serialize<W: io::Write>(&self, sink: W) -> Result<(), Error> {
        self.serialize_with_config(EmitterConfig::new().line_separator("\n")
                                                .perform_indent(true),
                                   sink)
    }
    /// Convenience method to create a custom EventWriter based on passed config
    fn serialize_with_config<W: io::Write>(&self, config: EmitterConfig, sink: W)
            -> Result<(), Error> {
        self.serialize_with(&mut config.create_writer(sink))
    }
    /// Serialize the data into XML file
    fn serialize_with<W: io::Write>(&self, sink: &mut EventWriter<W>)
            -> Result<(), Error> {
        try!(sink.write(XmlEvent::StartDocument { version: XmlVersion::Version11,
                                                  encoding: None,
                                                  standalone: None }));
        self.serialize_root(sink)
    }
    /// Write root element inside the EventWriter
    fn serialize_root<W: io::Write>(&self, sink: &mut EventWriter<W>)
            -> Result<(), Error>;
}

/// Character type which can be obtained and saved
///
/// Serialize type as character data for use as attribute value or character node
/// Allows to use multiple data types as source, e.g. save xsd:Decimal from f32 or f64
// ?Sized allows the use of &str as input
pub trait ToCharsVia<Data: ?Sized> {
    type Error: FormatError; // For simplicity, there should be only one Error type for any Data type
    fn to_characters(value: &Data) -> Result<String, Self::Error>;
}

/// Implement on converters to do Conv::serialize_via(data, ...)
pub trait SerializeVia<Data: ?Sized> {
    fn serialize_via<W: io::Write>(data: &Data, sink: &mut EventWriter<W>, name: &str)
        -> Result<(), Error>;
}

/// Leverage char conversion capabilities
impl<T, Data: ?Sized> SerializeVia<Data> for T where T: ToCharsVia<Data>,
        T::Error: Into<Error> {
    fn serialize_via<W: io::Write>(data: &Data, sink: &mut EventWriter<W>, name: &str)
            -> Result<(), Error> {
        let elemname = Name::local(name);
        try!(sink.write(
            XmlEvent::StartElement { name: elemname.clone(),
                                     attributes: Cow::Owned(Vec::new()),
                                     namespace: Cow::Owned(Namespace::empty()) }
        ));
        try!(sink.write(XmlEvent::Characters(&try!(T::to_characters(data)))));
        try!(sink.write(XmlEvent::EndElement { name: Some(elemname) }));
        Ok(())
    }
}

/// Special handling of namespaces
impl SerializeVia<xml::XmlElement> for conv::XmlElement {
    fn serialize_via<W: io::Write>(data: &xml::XmlElement, sink: &mut EventWriter<W>, name: &str) 
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
                &xml::XmlNode::Text(ref s) => {
                    sink.write(XmlEvent::Characters(s)).map_err(Error::from)
                },
                &xml::XmlNode::Element(ref e) => conv::XmlElement::serialize_via(e, sink, name),
            });
        }
        try!(sink.write(XmlEvent::EndElement { name: Some(data.name.borrow()) }));
        Ok(())
    }
}

