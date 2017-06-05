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

use std::io::Read;
use std::error::Error as ErrorTrait;

use self::_xml::reader::EventReader;
use self::_xml::name::OwnedName;
use self::_xml::attribute::OwnedAttribute;

use par::{ ParseVia, Positioned, ElementParse, ElementBuild };

use xml;
use xml::conv;
use xml::{ Element, Node };

use gpx::par::Error;

#[derive(Debug)]
pub enum BuildError {
    Custom(Box<ErrorTrait>)
}

#[derive(Default)]
pub struct ElementParser {
    //name: Option<OwnedName>, // Using reference intentionally - this code does not need to interact with Name
    attributes: Vec<OwnedAttribute>,
    nodes: Vec<Node>,
}

impl ElementBuild for ElementParser {
    type Element = Element;
    type BuildError = BuildError;
    fn build(self) -> Result<Self::Element, Self::BuildError> {
        Ok(Element {
            //name: self.name.unwrap().to_owned(),
            attributes: self.attributes,
            nodes: self.nodes
        })
    }
}

impl ElementParse<::gpx::par::Error> for ElementParser {
    fn parse_start(&mut self, attributes: &[OwnedAttribute])
            -> Result<(), ::par::AttributeError<::gpx::par::Error>> {
        let _ = attributes; // FIXME: break if attributes present
        Ok(())
    }
    fn parse_element<'a, R: Read>(&mut self, reader: &'a mut EventReader<R>,
                                  name: &OwnedName, attributes: &[OwnedAttribute])
            -> Result<(), Positioned<::gpx::par::Error>> {
        let elem = try!(ElementParser::new().parse(name, attributes, reader));
        self.nodes.push(Node::Element(name.clone(), elem));
        Ok(())
    }
    fn parse_characters(&mut self, data: String) -> Result<(), ::gpx::par::Error> {
        self.nodes.push(Node::Text(data));
        Ok(())
    }
}

impl ParseVia<xml::Element> for conv::Element {
    fn parse_via<R: Read>(parser: &mut EventReader<R>,
                              name: &OwnedName, attributes: &[OwnedAttribute])
            -> Result<xml::Element, Positioned<Error>> {
        ElementParser::new().parse(name, attributes, parser)
    }
}
