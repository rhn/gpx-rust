//! Generates foo_auto.rs files containing impls
extern crate xml_parsergen;

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::fs::File;
use std::io;
use std::io::{ Write, BufWriter };

use xml_parsergen::{ ParserGen, StructInfo, gpx, prettify };


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

        try!(f.write(gpx::Generator::header().as_bytes()).map_err(Error::Io));
        
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
    let structs = vec![
        StructInfo { name: "Metadata".into(),
                     type_: types.get("metadataType").unwrap(),
                     tags: HashMap::new() },
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
                           "longitudeType".into() => ("f64".into(), "Longitude::from_attr".into()),
                           "linkType".into() => ("String".into(), "parse_string".into()),
                           "fixType".into() => ("Fix".into(), "parse_fix".into()),
                           "dgpsStationType".into() => ("String".into(), "parse_string".into()), // FIXME
                           "extensionsType".into() => ("XmlElement".into(), "parse_elem".into()), // FIXME: dedicated type?
                           "xsd:decimal".into() => ("xsd::Decimal".into(), "parse_decimal".into()),
                           "xsd:dateTime".into() => ("xsd::DateTime".into(), "parse_time".into()),
                           "xsd:string".into() => ("String".into(), "parse_string".into()),
                           "xsd:nonNegativeInteger".into() => ("xsd::NonNegativeInteger".into(), "parse_u16".into()),
                           "xsd:degreesType".into() => ("xsd::Degrees".into(), "parse_string".into()), };
    let elem_convs = map!{ "boundsType".into() => "gpx::conv::Bounds".into() };

    try!(write_file(&out_dir.join("gpx_par_auto.rs"), |f| {
        for item in &structs {
            try!(f.write(
                gpx::Generator::parser_cls(&item.name, item.type_, &attr_convs).as_bytes()
            ).map_err(Error::Io));
        }
        Ok(())
    }));
    try!(write_file(&out_dir.join("gpx_ser_auto.rs"), |f| {
        for item in &structs {
            try!(f.write(
                gpx::Generator::serializer_impl(&item.name, &item.tags, item.type_, &elem_convs).as_bytes()
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
