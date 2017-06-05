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

pub enum Type {
    Simple(SimpleType),
    Complex(ComplexType),
}

pub struct ComplexType {
    pub attributes: Vec<Attribute>,
    pub sequence: Vec<Element>,
}

pub struct SimpleType {
    pub base: String,
    pub min_inclusive: f64,
    pub max_inclusive: Option<f64>,
    pub max_exclusive: Option<f64>,
}

pub struct Element {
    pub name: String,
    pub type_: String,
    pub max_occurs: ElementMaxOccurs,
}

pub enum ElementMaxOccurs {
    Some(u64),
    Unbounded,
}

pub struct Attribute {
    pub name: String,
    pub type_: String,
    pub required: bool,
}
