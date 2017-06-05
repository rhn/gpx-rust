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

//! Reads a file and saves somewhere else
extern crate gpx_rust;
extern crate clap;

use std::process::exit;
use std::io::{ BufReader, BufWriter };
use std::fs::File;
use clap::{ App, Arg };

use gpx_rust::ser;
use gpx_rust::ser::SerializeDocument;
use gpx_rust::gpx;
use gpx_rust::gpx::{ Gpx, Document };


#[derive(Debug)]
enum ResaveError {
    Io(std::io::Error),
    Serialize(ser::Error),
}

#[derive(Debug)]
enum ParseError {
    Io(std::io::Error),
    Parse(gpx::par::DocumentError),
}

fn parse(filename: &str) -> Result<Document, ParseError> {
    let f = try!(File::open(filename).map_err(ParseError::Io));
    let f = BufReader::new(f);
    gpx::parse(f).map_err(ParseError::Parse)
}

fn save(filename: &str, data: Gpx) -> Result<(), ResaveError> {
    let f = try!(File::create(filename).map_err(ResaveError::Io));
    let f = BufWriter::new(f);
    data.serialize(f).map_err(ResaveError::Serialize)//, WspMode::IndentLevel(0)).map_err(ResaveError::Io));
}

fn main() {
    let matches = App::new("Reader")
                      .arg(Arg::with_name("source")
                              .required(true))
                      .arg(Arg::with_name("destination")
                              .required(true))
                      .get_matches();
    let data = match parse(matches.value_of("source").unwrap()) {
        Err(e) => {
            println!("Failed to load\n{:?}", e);
            exit(1);
        }
        Ok(doc) => doc.data
    };
    save(matches.value_of("destination").unwrap(), data).expect("Failed to save");
}
