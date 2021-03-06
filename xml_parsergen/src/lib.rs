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

#[macro_use]
extern crate quote;

pub mod xsd_types;

pub mod gpx;


use std::io;
use std::process::{ Command, ExitStatus };
use std::collections::HashMap;
use std::path::Path;

use xsd_types::{ Type, SimpleType, ComplexType, Element, ElementMaxOccurs };

pub type TypeMap<'a> = HashMap<&'a str, Type>;
pub type TagMap<'a> = HashMap<&'a str, &'a str>;
pub type AttrMap = HashMap<String, (String, String)>;

/// This is ugly
pub struct StructInfo<'a> {
    pub name: String,
    pub type_name: String,
    pub tags: TagMap<'a>,
}

/// This is awful
pub struct ParserInfo<'a> {
    pub name: String,
    pub type_: &'a ComplexType,
}

#[derive(Debug)]
pub struct UserType(String);

impl UserType {
    fn as_str(&self) -> &str { &self.0 }
    fn as_user_type(&self) -> &str { self.as_str() }
}

impl<'a> From<&'a str> for UserType {
    fn from(data: &'a str) -> UserType { UserType(data.into()) }
}

pub type ConvMap = HashMap<String, (UserType, UserType)>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Command(ExitStatus),
}


pub trait ParserGen {
    fn data_struct_type(&self, name: &str, tags: &TagMap, data: &ComplexType,
                        type_convs: &ConvMap) -> String;
    fn parser_type(&self, name: &str, data: &ComplexType, type_convs: &ConvMap) -> String;
    fn parser_impl(&self, name: &str, data: &ComplexType, convs: &ConvMap) -> String;
    fn parse_impl(&self, type_name: &str, data: &SimpleType, convs: &ConvMap, types_: &TypeMap)
        -> String;
    fn parse_impl_complex(&self, parser_name: &str, conv_entry: &(UserType, UserType))
        -> String;
    fn build_impl(cls_name: &str, data: &ComplexType, struct_info: &StructInfo, convs: &ConvMap)
        -> String;
    fn serializer_impl(cls_name: &str, tags: &TagMap,
                       type_name: &str, data: &ComplexType, type_convs: &ConvMap) -> String;
}

pub fn ident_safe(name: &str) -> &str {
    match name {
        "type" => "type_",
        n => n
    }
}

pub fn get_elem_fieldname(e: &Element) -> String {
    ident_safe(match e.max_occurs {
        ElementMaxOccurs::Some(0) => panic!("Element has 0 occurrences, can't derive name"),
        ElementMaxOccurs::Some(1) => e.name.clone(),
        _ => format!("{}s", e.name)
    }.as_str()).into()
}

pub fn prettify(path: &Path) -> Result<(), Error> {
    let status = Command::new("rustfmt").arg(path)
                                        .spawn().map_err(Error::Io)?
                                        .wait().map_err(Error::Io)?;
    match status.success() {
        true => Ok(()),
        false => Err(Error::Command(status)),
    }
}
