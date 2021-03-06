/* This program and the accompanying materials are made available under the
 * terms of the Eclipse Public License v1.0 and the GNU General Public License
 * v3.0 or later which accompanies this distribution.
 * 
 *      The Eclipse Public License (EPL) v1.0 is available at
 *      http://www.eclipse.org/legal/epl-v10.html
 * 
 *      You should have received a copy of the GNU General Public License
 *      along with this program.  If not, see <http://www.gnu.org/licenses/>.
 * 
 * You may elect to redistribute this code under either of these licenses.     
 */

//! Serialization procedures for turning arbitrary data into XML documents
extern crate xml as _xml;

use std::fmt;
use std::io;
use std::borrow::Cow;

use self::_xml::common::XmlVersion;
use self::_xml::name::OwnedName;
use self::_xml::namespace::Namespace;
use self::_xml::writer;
use self::_xml::writer::{ EmitterConfig, EventWriter, XmlEvent };

    
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

/// Can be serialized as a simple string
///
/// Serializes value of the type as character data for use as attribute value or character node
/// Allows to use multiple data types as source, e.g. save xsd:Decimal from f32 or f64
// ?Sized allows the use of &str as input
pub trait ToCharsVia<Data: ?Sized> {
    type Error: FormatError; // For simplicity, there should be only one Error type for any Data type
    fn to_characters(value: &Data) -> Result<String, Self::Error>;
}

/// Can be serialized as a regular XML element
pub trait SerializeVia<Data: ?Sized> {
    fn serialize_via<W: io::Write>(data: &Data, sink: &mut EventWriter<W>, name: &OwnedName)
        -> Result<(), Error>;
}

/// Leverage char conversion capabilities
impl<T, Data: ?Sized> SerializeVia<Data> for T where T: ToCharsVia<Data>,
        T::Error: Into<Error> {
    fn serialize_via<W: io::Write>(data: &Data, sink: &mut EventWriter<W>, name: &OwnedName)
            -> Result<(), Error> {
        let elemname = name.borrow();
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
