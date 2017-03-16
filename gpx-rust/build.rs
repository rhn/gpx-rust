//! Generates foo_auto.rs files containing impls
extern crate xml_parsergen;

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::fs::File;
use std::io;
use std::io::{ Write, BufWriter };

use xml_parsergen::{ ParserGen, ParserInfo, StructInfo, TypeConverter, TypeMap, ConvMap, gpx, prettify };
use xml_parsergen::xsd_types::{ Type, SimpleType, ComplexType };
use xml_parsergen::gpx::DEFAULT_GENERATOR;


struct SimpleImplInfo<'a> {
    type_name: String,
    type_: &'a SimpleType,
    elem: bool,
}

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

fn get_complex<'a>(types: &'a TypeMap, name: &str) -> &'a ComplexType {
    match *types.get(name).expect(&format!("Type {} undefined", name)) {
         Type::Complex(ref type_) => type_,
         _ => panic!("Type of {} is not Type::Complex", name),
    }
}

fn get_simple<'a>(types: &'a TypeMap, name: &str) -> &'a SimpleType {
    match *types.get(name).expect(&format!("Type {} undefined", name)) {
         Type::Simple(ref type_) => type_,
         _ => panic!("Type of {} is not Type::Simple", name),
    }
}

#[derive(Debug)]
enum Error {
    Var(env::VarError),
    Io(io::Error),
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
    let attr_convs: ConvMap = map!{
        "gpxType".into() => ("Gpx".into(), TypeConverter::UniversalClass("::gpx::conv::Gpx".into())),
        "boundsType".into() => ("Bounds".into(), TypeConverter::UniversalClass("::gpx::conv::Bounds".into())),
        "copyrightType".into() => ("Copyright".into(), TypeConverter::UniversalClass("::gpx::conv::Copyright".into())),
        "latitudeType".into() => ("f64".into(), TypeConverter::UniversalClass("gpx::conv::Latitude".into())),
        "longitudeType".into() => ("f64".into(), TypeConverter::UniversalClass("gpx::conv::Longitude".into())),
        "linkType".into() => ("Link".into(), TypeConverter::UniversalClass("::gpx::conv::Link".into())),
        "fixType".into() => ("Fix".into(), TypeConverter::UniversalClass("::gpx::conv::Fix".into())),
        "dgpsStationType".into() => ("u16".into(), TypeConverter::UniversalClass("::gpx::conv::DgpsStation".into())),
        "extensionsType".into() => ("XmlElement".into(), TypeConverter::UniversalClass("::gpx::conv::Extensions".into())), // FIXME: dedicated type?
        "personType".into() => ("Person".into(), TypeConverter::UniversalClass("::gpx::conv::Person".into())),
        "wptType".into() => ("Waypoint".into(), TypeConverter::ParserClass("WaypointParser".into())),
        "metadataType".into() => ("Metadata".into(), TypeConverter::UniversalClass("::gpx::conv::Metadata".into())),
        "trkType".into() => ("Track".into(), TypeConverter::UniversalClass("::gpx::conv::Trk".into())),
        "rteType".into() => ("Route".into(), TypeConverter::UniversalClass("::gpx::conv::Rte".into())),
        "trksegType".into() => ("TrackSegment".into(), TypeConverter::UniversalClass("::gpx::conv::Trkseg".into())),
        "emailType".into() => ("String".into(), TypeConverter::UniversalClass("::gpx::conv::Email".into())),
        "_gpx:version".into() => ("Version".into(), TypeConverter::UniversalClass("::gpx::conv::Version".into())),
        "xsd:decimal".into() => ("xsd::Decimal".into(), TypeConverter::UniversalClass("::xsd::conv::Decimal".into())),
        "xsd:dateTime".into() => ("xsd::DateTime".into(), TypeConverter::UniversalClass("::xsd::conv::DateTime".into())),
        "xsd:string".into() => ("String".into(), TypeConverter::UniversalClass("::xsd::conv::String".into())),
        "xsd:nonNegativeInteger".into() => ("u64".into(), TypeConverter::UniversalClass("::xsd::conv::NonNegativeInteger".into())),
        "degreesType".into() => ("f32".into(), TypeConverter::UniversalClass("::gpx::conv::Degrees".into())),
        "xsd:anyURI".into() => ("xsd::Uri".into(), TypeConverter::UniversalClass("::xsd::conv::Uri".into())),
        "xsd:integer".into() => ("i64".into(), TypeConverter::UniversalClass("::xsd::conv::Integer".into())),
        "xsd:gYear".into() => ("i16".into(), TypeConverter::UniversalClass("::xsd::conv::GYear".into())),
    };
    let parsers = vec![
        ParserInfo { name: "TrackSegmentParser".into(), type_: get_complex(&types, "trksegType") },
        ParserInfo { name: "MetadataParser".into(), type_: get_complex(&types, "metadataType") },
        ParserInfo { name: "WaypointParser".into(), type_: get_complex(&types, "wptType") },
        ParserInfo { name: "BoundsParser".into(), type_: get_complex(&types, "boundsType") },
        ParserInfo { name: "GpxElemParser".into(), type_: get_complex(&types, "gpxType") },
        ParserInfo { name: "RteParser".into(), type_: get_complex(&types, "rteType") },
        ParserInfo { name: "TrkParser".into(), type_: get_complex(&types, "trkType") },
        ParserInfo { name: "LinkParser".into(), type_: get_complex(&types, "linkType") },
        ParserInfo { name: "CopyrightParser".into(), type_: get_complex(&types, "copyrightType") },
        ParserInfo { name: "PersonParser".into(), type_: get_complex(&types, "personType") },
        ParserInfo { name: "EmailParser".into(), type_: get_complex(&types, "emailType") },
    ];

    let simple_impls = vec![
        SimpleImplInfo { type_name: "degreesType".into(), type_: get_simple(&types, "degreesType"),
                         elem: true },
        SimpleImplInfo { type_name: "dgpsStationType".into(), type_: get_simple(&types, "dgpsStationType"),
                         elem: true },
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
        StructInfo { name: "Link".into(),
                     type_name: "linkType".into(),
                     tags: HashMap::new() },
        StructInfo { name: "Gpx".into(),
                     type_name: "gpxType".into(),
                     tags: map! {
                         "wpt" => "waypoints",
                         "rte" => "routes",
                         "trk" => "tracks" } },
        StructInfo { name: "Copyright".into(),
                     type_name: "copyrightType".into(),
                     tags: HashMap::new() },
        StructInfo { name: "Person".into(),
                     type_name: "personType".into(),
                     tags: HashMap::new() },
    ];
    let builder_impls = ["RteParser", "TrkParser", "LinkParser", "GpxElemParser", "TrackSegmentParser", "CopyrightParser", "PersonParser"]
                        .iter().map(|name: &&'static str| {
        let type_ = parsers.iter()
                                .find(|pinfo| pinfo.name.as_str() == *name)
                                .expect(&format!("{} not in parser impls", *name))
                                .type_;
        let sinfo = structs.iter()
                           .find(|sinfo| get_complex(&types, sinfo.type_name.as_str()) as *const _ == type_ as *const _)
                           .expect(&format!("type of {} not in structs", *name));
        (*name,
         type_,
         sinfo)
    }).collect::<Vec<_>>();
    let serializers = ["Metadata", "Track", "TrackSegment", "Route", "Link", "Copyright", "Person"]
                      .iter().map(|name: &&'static str| {
        structs.iter()
               .find(|sinfo| sinfo.name == *name)
               .expect(&format!("Structure {} not defined", *name))
    }).collect::<Vec<_>>();
    let parser_impls_via = vec![
        ("CopyrightParser", attr_convs.get("copyrightType").expect("copyrighterr")),
        ("PersonParser", attr_convs.get("personType").expect("personerr")),
        ("EmailParser", attr_convs.get("emailType").expect("emailerr")),
    ];
    try!(write_file(&out_dir.join("gpx_auto.rs"), |f| {
        for item in &structs {
            try!(f.write(
                gpx::Generator::struct_def(&item.name, &item.tags,
                                           get_complex(&types, item.type_name.as_str()),
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
            try!(f.write(
                gpx::Generator::parser_impl(&item.name, item.type_, &attr_convs).as_bytes()
            ).map_err(Error::Io));
        }
        for &(ref name, ref conv_) in &parser_impls_via {
            try!(f.write(
                DEFAULT_GENERATOR.parse_impl_complex(name, conv_).as_bytes()
            ).map_err(Error::Io));
        }
        for item in &builder_impls {
            try!(f.write(
                gpx::Generator::build_impl(&item.0, item.1, item.2, &attr_convs).as_bytes()
            ).map_err(Error::Io));
        }
        for item in &simple_impls {
            try!(f.write(
                DEFAULT_GENERATOR.parse_impl(&item.type_name, item.type_,
                                             &attr_convs, &types).as_bytes()
            ).map_err(Error::Io));
        }
        Ok(())
    }));
    try!(write_file(&out_dir.join("gpx_ser_auto.rs"), |f| {
        for item in &serializers {
            try!(f.write(
                gpx::Generator::serializer_impl(&item.name, &item.tags,
                                                &item.type_name,
                                                get_complex(&types, item.type_name.as_str()),
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
