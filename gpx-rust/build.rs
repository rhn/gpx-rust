//! Generates foo_auto.rs files containing impls
extern crate xml_parsergen;

use std::env;
use std::path::PathBuf;
use std::fs::File;
use std::io;
use std::io::{ Write, BufWriter };

use xml_parsergen::{ ParserGen, ParserInfo, StructInfo, TypeConverter, TypeMap, gpx, prettify };


macro_rules! map(
    { $($key:expr => $value:expr),* $(,)* } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )*
            m
        }
     };
);


#[derive(Debug)]
enum Error {
    Var(env::VarError),
    Io(io::Error)
}

fn write_file<F: FnOnce(&mut BufWriter<File>) -> Result<(), Error>>(filename: &PathBuf, inner: F)
        -> Result<(), Error> {
    { // to drop and flush & close f before prettifying
        let f = try!(File::create(filename.clone()).map_err(Error::Io));
        let mut f = BufWriter::new(f);
        
        try!(inner(&mut f));
    }
    Ok(match prettify(filename) {
        Ok(_) => {},
        Err(e) => println!("warning=prettifying failed with {:?}", e),
    })
}

fn process() -> Result<(), Error> {
    let out_dir = PathBuf::from(try!(env::var("OUT_DIR").map_err(Error::Var)));

    let types = gpx::get_types();
    let attr_convs: TypeMap = map!{
        "boundsType".into() => ("Bounds".into(), TypeConverter::UniversalClass("::gpx::conv::Bounds".into())),
        "copyrightType".into() => ("XmlElement".into(), TypeConverter::ParseFun("parse_elem".into())), // FIXME
        "latitudeType".into() => ("f64".into(), TypeConverter::AttributeFun("gpx::conv::Latitude::from_attr".into())),
        "longitudeType".into() => ("f64".into(), TypeConverter::AttributeFun("gpx::conv::Longitude::from_attr".into())),
        "linkType".into() => ("XmlElement".into(), "parse_elem".into()),
        "fixType".into() => ("Fix".into(), "parse_fix".into()),
        "dgpsStationType".into() => ("String".into(), "parse_string".into()), // FIXME
        "extensionsType".into() => ("XmlElement".into(), "parse_elem".into()), // FIXME: dedicated type?
        "personType".into() => ("XmlElement".into(), TypeConverter::ParseFun("parse_elem".into())), // FIXME
        "wptType".into() => ("Waypoint".into(), TypeConverter::ParserClass("WaypointParser".into())),
        "metadataType".into() => ("Metadata".into(), TypeConverter::UniversalClass("::gpx::conv::Metadata".into())),
        "trkType".into() => ("Track".into(), TypeConverter::UniversalClass("::gpx::conv::Trk".into())),
        "rteType".into() => ("Route".into(), TypeConverter::UniversalClass("::gpx::conv::Rte".into())),
        "trksegType".into() => ("TrackSegment".into(), TypeConverter::UniversalClass("::gpx::conv::Trkseg".into())),
        "_gpx:version".into() => ("GpxVersion".into(), TypeConverter::UniversalClass("::gpx::conv::Version".into())),
        "xsd:decimal".into() => ("xsd::Decimal".into(), "parse_decimal".into()),
        "xsd:dateTime".into() => ("xsd::DateTime".into(), "parse_time".into()),
        "xsd:string".into() => ("String".into(), TypeConverter::UniversalClass("::xsd::conv::String".into())),
        "xsd:nonNegativeInteger".into() => ("xsd::NonNegativeInteger".into(), "parse_u64".into()),
        "xsd:degreesType".into() => ("xsd::Degrees".into(), "parse_string".into()),
    };
    let parsers = vec![
        ParserInfo { name: "TrackSegmentParser".into(), type_: types.get("trksegType").unwrap() },
        ParserInfo { name: "MetadataParser".into(), type_: types.get("metadataType").unwrap() },
        ParserInfo { name: "WaypointParser".into(), type_: types.get("wptType").unwrap() },
        ParserInfo { name: "BoundsParser".into(), type_: types.get("boundsType").unwrap() },
        ParserInfo { name: "GpxElemParser".into(), type_: types.get("gpxType").unwrap() },
        ParserInfo { name: "RteParser".into(), type_: types.get("rteType").unwrap() },
        ParserInfo { name: "TrkParser".into(), type_: types.get("trkType").unwrap() },
    ];
    let parser_impls = vec![
        ParserInfo { name: "TrackSegmentParser".into(), type_: types.get("trksegType").unwrap() },
        ParserInfo { name: "MetadataParser".into(), type_: types.get("metadataType").unwrap() },
        ParserInfo { name: "WaypointParser".into(), type_: types.get("wptType").unwrap() },
        ParserInfo { name: "BoundsParser".into(), type_: types.get("boundsType").unwrap() },
        ParserInfo { name: "GpxElemParser".into(), type_: types.get("gpxType").unwrap() },
        ParserInfo { name: "RteParser".into(), type_: types.get("rteType").unwrap() },
        ParserInfo { name: "TrkParser".into(), type_: types.get("trkType").unwrap() },
    ];
    let structs = vec![
        StructInfo { name: "Metadata".into(),
                     type_name: "metadataType".into(),
                     tags: map! { "desc" => "description" } },
        StructInfo { name: "Track".into(), type_name: "trkType".into(),
                     tags: map! {
                         "cmt" => "comment",
                         "desc" => "description",
                         "src" => "source",
                         "link" => "links",
                         "type" => "type_",
                         "trkseg" => "segments" } },
        StructInfo { name: "TrackSegment".into(), type_name: "trksegType".into(),
                     tags: map! { "trkpt" => "waypoints" } },
        StructInfo { name: "Route".into(), type_name: "rteType".into(),
                     tags: map! { 
                         "cmt" => "comment",
                         "desc" => "description",
                         "src" => "source",
                         "link" => "links",
                         "type" => "type_",
                         "rtept" => "waypoints" } },
    ];
    let builder_impls = ["RteParser", "TrkParser"].iter().map(|name: &&'static str| {
        let type_ = parser_impls.iter().find(|pinfo| pinfo.name.as_str() == *name).unwrap().type_;
        let sinfo = structs.iter()
                           .find(|sinfo| types.get(sinfo.type_name.as_str()).unwrap() as *const _ == type_ as *const _)
                           .unwrap();
        (*name,
         type_,
         sinfo)
    }).collect::<Vec<_>>();

    try!(write_file(&out_dir.join("gpx_auto.rs"), |f| {
        for item in &structs {
            try!(f.write(
                gpx::Generator::struct_def(&item.name, &item.tags,
                    types.get(item.type_name.as_str()).unwrap(),
                    &attr_convs).as_bytes()
            ).map_err(Error::Io));
        }
        Ok(())
    }));
    try!(write_file(&out_dir.join("gpx_par_auto.rs"), |f| {
        for item in &parsers {
            try!(f.write(
                gpx::Generator::parser_cls(&item.name, item.type_, &attr_convs).as_bytes()
            ).map_err(Error::Io));
        }
        for item in &parser_impls {
            try!(f.write(
                gpx::Generator::parser_impl(&item.name, item.type_, &attr_convs).as_bytes()
            ).map_err(Error::Io));
        }
        for item in &builder_impls {
            try!(f.write(
                gpx::Generator::build_impl(&item.0, item.1, item.2, &attr_convs).as_bytes()
            ).map_err(Error::Io));
        }
        Ok(())
    }));
    try!(write_file(&out_dir.join("gpx_ser_auto.rs"), |f| {
        try!(f.write(gpx::Generator::header().as_bytes()).map_err(Error::Io));
        for item in &structs {
            try!(f.write(
                gpx::Generator::serializer_impl(&item.name, &item.tags,
                                                &item.type_name,
                                                types.get(item.type_name.as_str()).unwrap(),
                                                &attr_convs).as_bytes()
            ).map_err(Error::Io));
        }
        Ok(())
    }));
    Ok(())
}

fn main() {
    match process() {
        Ok(_) => (),
        Err(e) => println!("{:?}", e),
    }
}
