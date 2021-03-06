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

//! Generates foo_auto.rs files containing impls
extern crate xml_parsergen;

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::fs::File;
use std::io;
use std::io::{ Write, BufWriter };

use xml_parsergen::{ ParserGen, ParserInfo, StructInfo, TypeMap, ConvMap, gpx, prettify };
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
        "gpxType".into() => ("Gpx".into(), "::gpx::conv::Gpx".into()),
        "boundsType".into() => ("Bounds".into(), "::gpx::conv::Bounds".into()),
        "copyrightType".into() => ("Copyright".into(), "::gpx::conv::Copyright".into()),
        "latitudeType".into() => ("f64".into(), "gpx::conv::Latitude".into()),
        "longitudeType".into() => ("f64".into(), "gpx::conv::Longitude".into()),
        "linkType".into() => ("Link".into(), "::gpx::conv::Link".into()),
        "fixType".into() => ("Fix".into(), "::gpx::conv::Fix".into()),
        "dgpsStationType".into() => ("u16".into(), "::gpx::conv::DgpsStation".into()),
        "extensionsType".into() => ("xml::Element".into(), "::gpx::conv::Extensions".into()), // FIXME: dedicated type?
        "personType".into() => ("Person".into(), "::gpx::conv::Person".into()),
        "wptType".into() => ("Waypoint".into(), "::gpx::conv::Wpt".into()),
        "metadataType".into() => ("Metadata".into(), "::gpx::conv::Metadata".into()),
        "trkType".into() => ("Track".into(), "::gpx::conv::Trk".into()),
        "rteType".into() => ("Route".into(), "::gpx::conv::Rte".into()),
        "trksegType".into() => ("TrackSegment".into(), "::gpx::conv::Trkseg".into()),
        "emailType".into() => ("String".into(), "::gpx::conv::Email".into()),
        "_gpx:version".into() => ("Version".into(), "::gpx::conv::Version".into()),
        "xsd:decimal".into() => ("xsd::Decimal".into(), "::xsd::conv::Decimal".into()),
        "xsd:dateTime".into() => ("xsd::DateTime".into(), "::xsd::conv::DateTime".into()),
        "xsd:string".into() => ("String".into(), "::xsd::conv::String".into()),
        "xsd:nonNegativeInteger".into() => ("u64".into(), "::xsd::conv::NonNegativeInteger".into()),
        "degreesType".into() => ("f32".into(), "::gpx::conv::Degrees".into()),
        "xsd:anyURI".into() => ("xsd::Uri".into(), "::xsd::conv::Uri".into()),
        "xsd:integer".into() => ("i64".into(), "::xsd::conv::Integer".into()),
        "xsd:gYear".into() => ("i16".into(), "::xsd::conv::GYear".into()),
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
    let builder_impls = ["RteParser", "TrkParser", "LinkParser", "GpxElemParser", "TrackSegmentParser", "CopyrightParser", "PersonParser", "MetadataParser"]
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
        ("WaypointParser", attr_convs.get("wptType").expect("wpterr")),
        ("MetadataParser", attr_convs.get("metadataType").expect("metaerr"))
    ];
    try!(write_file(&out_dir.join("gpx_auto.rs"), |f| {
        for item in &structs {
            try!(f.write(
                DEFAULT_GENERATOR.data_struct_type(&item.name, &item.tags,
                                                   get_complex(&types, item.type_name.as_str()),
                                                   &attr_convs).as_bytes()
            ).map_err(Error::Io));
        }
        Ok(())
    }));
    try!(write_file(&out_dir.join("gpx_par_auto.rs"), |f| {
        for item in &parsers {
            try!(f.write(
                DEFAULT_GENERATOR.parser_type(&item.name, item.type_, &attr_convs).as_bytes()
            ).map_err(Error::Io));
            try!(f.write(
                DEFAULT_GENERATOR.parser_impl(&item.name, item.type_, &attr_convs).as_bytes()
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
