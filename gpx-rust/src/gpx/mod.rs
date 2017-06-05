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

//! GPX types

extern crate xml as _xml;
extern crate chrono;
extern crate geo;

use std::io;
use self::geo::Bbox;

use xml;
use xsd;
use xsd::*;

mod conv;
mod ser_auto;
pub mod ser;
pub mod par;

/// Parses XML stream containing GPX data
pub use self::par::parse;

/// Xml document containing parsed GPX data
///
/// ```
/// let gpx = document.data;
/// ```
pub type Document = xml::Document<Gpx>;

/// `gpxType` contents
#[derive(XmlDebug)]
pub struct Gpx {
    pub version: Version,
    pub creator: String,
    pub metadata: Option<Metadata>,
    pub waypoints: Vec<Waypoint>,
    pub routes: Vec<Route>,
    pub tracks: Vec<Track>,
    pub extensions: Option<xml::Element>,
}

/// `<gpx version=...>` attribute values
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum Version {
    V1_0,
    V1_1,
}

/// `metadataType` contents
#[derive(XmlDebug)]
pub struct Metadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub author: Option<Person>,
    pub copyright: Option<Copyright>,
    pub links: Vec<Link>,
    pub time: Option<Time>,
    pub keywords: Option<String>,
    pub bounds: Option<Bounds>,
    pub extensions: Option<xml::Element>,
}

/// `personType` contents
#[derive(XmlDebug)]
pub struct Person {
    pub name: Option<String>,
    pub email: Option<String>,
    pub link: Option<Link>,
}

/// `copyrightType` contents
#[derive(XmlDebug)]
pub struct Copyright {
    pub author: String,
    pub year: Option<i16>,
    pub license: Option<xsd::Uri>,
}

/// `linkType` contents
#[derive(XmlDebug)]
pub struct Link {
    pub href: xsd::Uri,
    pub text: Option<String>,
    pub type_: Option<String>,
}

/// `boundsType` contents
pub type Bounds = Bbox<f64>;

/// `<wpt>`, `<rtept>`, `<trkpt>` elements and `wptType`
#[derive(XmlDebug)]
pub struct Waypoint {
    pub location: Point,
    pub time: Option<xsd::DateTime>,
    pub mag_variation: Option<Degrees>,
    pub geoid_height: Option<xsd::Decimal>,
    pub name: Option<String>,
    pub comment: Option<String>,
    pub description: Option<String>,
    pub source: Option<String>,
    pub links: Vec<Link>,
    pub symbol: Option<String>,
    pub type_: Option<String>,
    pub fix: Option<Fix>,
    pub satellites: Option<xsd::NonNegativeInteger>,
    pub hdop: Option<xsd::Decimal>,
    pub pdop: Option<xsd::Decimal>,
    pub vdop: Option<xsd::Decimal>,
    pub dgps_age: Option<xsd::Decimal>,
    pub dgps_id: Option<u16>,
    pub extensions: Option<xml::Element>,
}

/// WGS84 geographical coordinates
#[derive(XmlDebug)]
pub struct Point {
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: Option<f64>,
}

/// `<fix>` and `fixType`
#[derive(Debug)]
pub enum Fix {
    None,
    _2D,
    _3D,
    DGPS,
    PPS
}

/// `<trk>` and `trkType`
#[derive(XmlDebug)]
pub struct Track {
    pub name: Option<String>,
    pub comment: Option<String>,
    pub description: Option<String>,
    pub source: Option<String>,
    pub links: Vec<Link>,
    pub number: Option<xsd::NonNegativeInteger>,
    pub type_: Option<String>,
    pub extensions: Option<xml::Element>,
    pub segments: Vec<TrackSegment>,
}

/// `<trkseg>` and `trksegType`
#[derive(XmlDebug)]
pub struct TrackSegment {
    pub waypoints: Vec<Waypoint>,
    pub extensions: Option<xml::Element>,
}

/// `<rte>` and `rteType`
#[derive(XmlDebug)]
pub struct Route {
    pub name: Option<String>,
    pub comment: Option<String>,
    pub description: Option<String>,
    pub source: Option<String>,
    pub links: Vec<Link>,
    pub number: Option<xsd::NonNegativeInteger>,
    pub type_: Option<String>,
    pub extensions: Option<xml::Element>,
    pub waypoints: Vec<Waypoint>,
}

/// direction on the circle
pub type Degrees = f32;
