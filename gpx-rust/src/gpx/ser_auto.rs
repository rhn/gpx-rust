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

use std::borrow::Cow;

use self::_xml::attribute::{ Attribute, OwnedAttribute };
use self::_xml::name::OwnedName;
use self::_xml::namespace::Namespace;
use self::_xml::writer::{ XmlEvent, EventWriter };

use ser::{ Error, SerializeVia, ToCharsVia };
use gpx::*;

include!(concat!(env!("OUT_DIR"), "/gpx_ser_auto.rs"));
