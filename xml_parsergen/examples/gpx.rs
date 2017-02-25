extern crate clap;
extern crate xml_parsergen;

use std::collections::HashMap;
use std::io;
use std::io::{ Write, BufWriter };
use std::fs::File;
use std::path::Path;
use clap::{ App, Arg };

use xml_parsergen::{ ParserGen, StructInfo, ParserInfo, gpx, prettify, AttrMap };


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

/// type_convs - type->converter class mapping. So far elements only, user-provided, may be missing
fn save(filename: &str, attr_type_convs: AttrMap, type_convs: HashMap<String, String>,
                        serializers: Vec<StructInfo>,
                        types: Vec<ParserInfo>) -> Result<(), io::Error> {
    let f = try!(File::create(filename));
    let mut f = BufWriter::new(f);

    try!(f.write(gpx::Generator::header().as_bytes()));
    
    for item in types {
        try!(f.write(
            gpx::Generator::parser_cls(&item.name, item.type_, &attr_type_convs).as_bytes()
        ));
        try!(f.write(
            gpx::Generator::parser_impl(&item.name, item.type_, &attr_type_convs).as_bytes()
        )); 
    }
    for item in serializers {
        try!(f.write(
            gpx::Generator::serializer_impl(&item.name, &item.tags, item.type_, &type_convs).as_bytes()
        ));
    }
    Ok(())
}

fn main() {
    let matches = App::new("codegen")
                      .arg(Arg::with_name("destination")
                              .required(true))
                      .get_matches();

    let types = gpx::get_types();
    let structs = vec![
        StructInfo { name: "Metadata".into(), type_: types.get("metadataType").unwrap(), tags: map! { } },
        StructInfo { name: "Track".into(), type_: types.get("trkType").unwrap(),
                     tags: map! {
                          "cmt" => "comment",
                          "desc" => "description",
                          "src" => "source",
                          "link" => "links",
                          "type" => "type_",
                          "trkseg" => "segments"} },
        StructInfo { name: "TrackSegment".into(), type_: types.get("trksegType").unwrap(),
                     tags: map! { "trkpt" => "waypoints" } },
    ];
    let attr_convs = map!{ "latitudeType".into() => ("f64".into(), "Latitude::from_attr".into()),
                           "longitudeType".into() => ("f64".into(), "Longitude::from_attr".into()) };
    let elem_convs = map!{ "boundsType".into() => "gpx::conv::Bounds".into() };
    let parsers = vec![
        ParserInfo { name: "BoundsParser".into(), type_: types.get("boundsType").unwrap() },
        ParserInfo { name: "WaypointParser".into(), type_: types.get("wptType").unwrap() },
    ];
    let dest = matches.value_of("destination").unwrap();
    save(dest, attr_convs, elem_convs, structs, parsers).expect("Failed to save");
    prettify(Path::new(dest.into())).expect("Failed to prettify");
}
