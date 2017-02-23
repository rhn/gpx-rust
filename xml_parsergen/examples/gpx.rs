extern crate clap;
extern crate xml_parsergen;

use std::io;
use std::io::{ Write, BufWriter };
use std::fs::File;
use clap::{ App, Arg };

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

fn save(filename: &str, structs: Vec<StructInfo>) -> Result<(), io::Error> {
    let f = try!(File::create(filename));
    let mut f = BufWriter::new(f);

    try!(f.write(gpx::Generator::header().as_bytes()));
    
    for item in structs {
        try!(f.write(
            gpx::Generator::serializer_impl(&item.name, &item.tags, item.type_).as_bytes()
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

    save(matches.value_of("destination").unwrap(), structs).expect("Failed to save");
}
