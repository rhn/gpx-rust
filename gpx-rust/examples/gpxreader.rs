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

extern crate gpx_rust;
extern crate clap;

use std::io::BufReader;
use std::fs::File;
use clap::{App, Arg};
use gpx_rust::xml::ParseXml;
use gpx_rust::gpx::{ Gpx, Parser, Error };


fn parse(filename: &str) -> Result<Gpx, Error> {
    let f = try!(File::open(filename).map_err(Error::Io));
    let f = BufReader::new(f);
    Parser::new(f).parse()
}
 
fn main() {
    let matches = App::new("Reader")
                      .arg(Arg::with_name("filename")
                              .required(true))
                      .get_matches();
    let out = parse(matches.value_of("filename").unwrap()).expect("fail");
    println!("{:?}", out);
}
