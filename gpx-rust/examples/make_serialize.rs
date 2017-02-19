#[macro_use]
extern crate quote;
extern crate clap;

use std::io;
use std::io::{ Write, BufWriter };
use std::fs::File;
use std::collections::HashMap;
use clap::{ App, Arg };


struct XsdType<'a> {
    sequence: Vec<XsdElement<'a>>,
}

struct XsdElement<'a> {
    name: String,
    type_: XsdElementType<'a>,
    max_occurs: XsdElementMaxOccurs,
}


enum XsdElementType<'a> {
    Name(String),
    Type_(&'a XsdType<'a>)
}

enum XsdElementMaxOccurs {
    Some(u64),
    Unbounded,
}

type TagMap<'a> = HashMap<&'a str, &'a str>;

struct StructInfo<'a> {
    name: String,
    type_: &'a XsdType<'a>,
    tags: TagMap<'a>,
}

macro_rules! XsdElementSingle (
    ( $name:expr, $type_:expr ) => {
        XsdElement { name: String::from($name),
                     type_: XsdElementType::Name(String::from($type_)),
                     max_occurs: XsdElementMaxOccurs::Some(1) }
    }
);

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

fn make_header() -> &'static str {
    "extern crate xml as _xml;

use std::borrow::cow;
use self::_xml::name::Name;
use self::_xml::name::Namespace;
use self::_xml::writer::{ XmlEvent, EventWriter };
use gpx_rust::ser::Serialize;"
}

fn make_ser_impl(cls_name: &str, tags: &TagMap, data: &XsdType) -> String {
    let cls_name = quote::Ident::new(cls_name);
    let events = data.sequence.iter().map(|elem| {
        let elem_name = elem.name.clone();
        let get_attr_name = |f: &Fn(&str) -> String| {
            quote::Ident::new(match tags.get(elem_name.as_str()) {
                Some(i) => { 
                    String::from(i.clone())
                },
                None => f(elem_name.as_str())
            })
        };
        match elem.max_occurs {
            XsdElementMaxOccurs::Some(1) => {
                let name = get_attr_name(&|n| { String::from(n) });
                quote!(
                    if let Some(ref item) = self.#name {
                        try!(item.serialize_with(sink, #elem_name));
                    }
                )
            }
            _ => {
                let name = get_attr_name(&|n| { format!("{}s", n) });
                quote!(
                    for item in &self.#name {
                        try!(item.serialize_with(sink, #elem_name));
                    }
                )
            }
        }
    });
    let fun_body = quote!(
        fn serialize_with<W: io::Write>(&self, sink: &mut EventWriter<W>, name: &str) -> writer::Result<()> {
            let elemname = Name::local(name);
            try!(sink.write(
                XmlEvent::StartElement {
                    name: elemname.clone(),
                    attributes: Cow::Owned(vec![]),
                    namespace: Cow::Owned(Namespace::empty())
                }
            ));
            
            #( #events )*
            
            sink.write(XmlEvent::EndElement { name: Some(elemname) })
        }
    );
    quote!(
        impl Serialize for #cls_name {
            #fun_body
        }
    ).to_string()
}

fn save(filename: &str, structs: Vec<StructInfo>) -> Result<(), io::Error> {
    let f = try!(File::create(filename));
    let mut f = BufWriter::new(f);
    
    try!(f.write(make_header().as_bytes()));
    
    for item in structs {
        try!(f.write(
            make_ser_impl(&item.name, &item.tags, item.type_).as_bytes()
        ));
    }
    //try!(f.write("sss".as_bytes()));
    Ok(())
}

fn main() {
    let matches = App::new("codegen")
                      .arg(Arg::with_name("destination")
                              .required(true))
                      .get_matches();
    let types = map![
        "metadataType" => XsdType {
            sequence: vec![
                XsdElement { name: String::from("name"),
                             type_: XsdElementType::Name(String::from("xsd:string")),
                             max_occurs: XsdElementMaxOccurs::Some(1) },
                XsdElement { name: String::from("desc"),
                             type_: XsdElementType::Name(String::from("xsd:string")),
                             max_occurs: XsdElementMaxOccurs::Some(1) },
                XsdElementSingle!("author", "personType"),
                XsdElementSingle!("copyright", "copyrightType"),
                XsdElement { name: String::from("link"),
                             type_: XsdElementType::Name(String::from("linkType")),
                             max_occurs: XsdElementMaxOccurs::Unbounded },
                XsdElementSingle!("time", "xsd:dateTime"),
                XsdElementSingle!("keywords", "xsd:string"),
                XsdElementSingle!("bounds", "boundsType"),
                XsdElementSingle!("extensions", "extensionsType"),
            ]
        },
        "trkType" => XsdType {
            sequence: vec![
                XsdElementSingle!("name", "xsd:string"),
                XsdElementSingle!("cmt", "xsd:string"),
                XsdElementSingle!("desc", "xsd:string"),
                XsdElementSingle!("src", "xsd:string"),
                XsdElement { name: String::from("link"),
                             type_: XsdElementType::Name(String::from("linkType")),
                             max_occurs: XsdElementMaxOccurs::Unbounded },
                XsdElementSingle!("number", "xsd:nonNegativeInteger"),
                XsdElementSingle!("type", "xsd:string"),
                XsdElementSingle!("extensions", "extensionType"),
                XsdElement { name: String::from("trkseg"),
                             type_: XsdElementType::Name(String::from("trksegType")),
                             max_occurs: XsdElementMaxOccurs::Unbounded },
            ]
        },
        "trksegType" => XsdType {
            sequence: vec![
                XsdElement { name: "trkpt".into(),
                             type_: XsdElementType::Name("wptType".into()),
                             max_occurs: XsdElementMaxOccurs::Unbounded },
            ]
        },
    ];
    
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
