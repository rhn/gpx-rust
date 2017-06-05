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

//! Converters for use with parsers and serializers

/// Defines conversions for GPX boundsType
pub struct Bounds {}

/// Defines conversions for GPX gpxType gpx::Gpx type
pub struct Gpx {}

/// Defines conversions between GPX latitudeType and String
pub struct Latitude {}

/// Defines conversions between GPX longitudeType and String
pub struct Longitude {}

/// Defines conversions for GPX linkType
pub struct Link {}

/// Defines conversions for GPX metadataType
pub struct Metadata {}

/// Defines conversions for GPX rteType
pub struct Rte {}

/// Defines conversions for GPX trkType
pub struct Trk {}

/// Defines conversions for GPX trksegType
pub struct Trkseg {}

/// Defines conversions between gpx::Version type and GPX version attribute
pub struct Version {}

/// Defines conversions for GPX fixType
pub struct Fix {}

/// Defines conversions for GPX degreesType
pub struct Degrees {}

/// Defines conversions for GPX extensionsType
pub type Extensions = ::xml::conv::Element;

/// Defines conversion for GPX dgpsStationType
pub struct DgpsStation {}

/// Defines conversion for GPX copyrightType
pub struct Copyright {}

/// Defines conversion for GPX personType
pub struct Person {}

/// Defines conversion between String and GPX emailType
pub struct Email {}

/// Defines conversion for GPX wptType
pub struct Wpt {}
