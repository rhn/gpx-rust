//! Generates foo_auto.rs files containing impls
extern crate xml_parsergen;

use std::env;
use std::path::PathBuf;
use std::fs::File;
use std::io;
use std::io::{ Write, BufWriter };

use xml_parsergen::{ ParserGen, StructInfo, gpx };


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

fn process() -> Result<(), Error> {
    let out_dir = PathBuf::from(try!(env::var("OUT_DIR").map_err(Error::Var)));
    let mut out_path = out_dir.clone();

    let types = gpx::get_types();
    let structs = vec![
        StructInfo { name: "Metadata".into(),
                     type_: types.get("metadataType").unwrap(),
                     tags: map! { } },
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

    out_path.set_file_name("gpx_ser_auto.rs");
    let f = try!(File::create(out_path).map_err(Error::Io));
    let mut f = BufWriter::new(f);

    try!(f.write(gpx::Generator::header().as_bytes()).map_err(Error::Io));
    
    for item in structs {
        try!(f.write(
            gpx::Generator::serializer_impl(&item.name, &item.tags, item.type_).as_bytes()
        ).map_err(Error::Io));
    }
    Ok(())
}

fn main() {
    match process() {
        Ok(_) => (),
        Err(e) => println!("{:?}", e),
    }
}
