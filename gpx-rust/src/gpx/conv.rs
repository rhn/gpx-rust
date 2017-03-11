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

/// Defines conversions for GPX extensionsType
pub type Extensions = ::conv::XmlElement;
