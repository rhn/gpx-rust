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

pub struct XsdType<'a> {
    pub sequence: Vec<XsdElement<'a>>,
}

pub struct XsdElement<'a> {
    pub name: String,
    pub type_: XsdElementType<'a>,
    pub max_occurs: XsdElementMaxOccurs,
}

pub enum XsdElementType<'a> {
    Name(String),
    Type_(&'a XsdType<'a>)
}

pub enum XsdElementMaxOccurs {
    Some(u64),
    Unbounded,
}
