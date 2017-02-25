//! Generates foo_auto.rs files containing impls
extern crate xml_parsergen;

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

fn process() -> Result<(), Error> {
    let out_dir = PathBuf::from(try!(env::var("OUT_DIR").map_err(Error::Var)));

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
    let elem_convs = map!{ "boundsType".into() => "gpx::conv::Bounds".into() };

    let out_path = out_dir.join("gpx_ser_auto.rs");
    { // to drop f before prettification
        let f = try!(File::create(out_path.clone()).map_err(Error::Io));
        let mut f = BufWriter::new(f);

        try!(f.write(gpx::Generator::header().as_bytes()).map_err(Error::Io));
        
        for item in structs {
            try!(f.write(
                gpx::Generator::serializer_impl(&item.name, &item.tags, item.type_, &elem_convs).as_bytes()
            ).map_err(Error::Io));
        }
    }
    match prettify(&out_path) {
        Ok(_) => {},
        Err(e) => println!("warning=prettifying failed with {:?}", e),
    }
    Ok(())
}

fn main() {
    match process() {
        Ok(_) => (),
        Err(e) => println!("{:?}", e),
    }
}
