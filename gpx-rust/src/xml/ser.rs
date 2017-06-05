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

extern crate xml as _xml;

use std::io::Write;
use std::borrow::Cow;

use self::_xml::name::OwnedName;
use self::_xml::writer::{ EventWriter, XmlEvent };

use ser::{ SerializeVia, Error };

use xml;
use xml::conv;

/// Special handling of namespaces
impl SerializeVia<xml::Element> for conv::Element {
    fn serialize_via<W: Write>(data: &xml::Element, sink: &mut EventWriter<W>, name: &OwnedName) 
            -> Result<(), Error> {
        try!(sink.write(
            XmlEvent::StartElement { name: name.borrow(),
                                     attributes: Cow::Borrowed(
                                         data.attributes
                                             .iter()
                                             .map(|a| { a.borrow() })
                                             .collect::<Vec<_>>()
                                             .as_slice()),
                                     namespace: Cow::Borrowed(&data.get_namespaces(name)) }
        ));
        for node in &data.nodes {
            try!(match node {
                &xml::Node::Text(ref s) => {
                    sink.write(XmlEvent::Characters(s)).map_err(Error::from)
                },
                &xml::Node::Element(ref name, ref e) => conv::Element::serialize_via(e, sink, name),
            });
        }
        try!(sink.write(XmlEvent::EndElement { name: Some(name.borrow()) }));
        Ok(())
    }
}
