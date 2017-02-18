#[macro_use]
extern crate quote;
extern crate clap;

use std::io;
use std::io::{ Write, BufReader, BufWriter };
use std::fs::File;
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


struct StructInfo<'a> {
    name: String,
    type_: &'a XsdType<'a>,
}

macro_rules! XsdElementSingle (
    ( $name:expr, $type_:expr ) => {
        XsdElement { name: String::from($name),
                     type_: XsdElementType::Name(String::from($type_)),
                     max_occurs: XsdElementMaxOccurs::Some(1) }
    }
);

fn make_ser_impl(cls_name: &str, data: &XsdType) -> String {
    let cls_name = quote::Ident::new(cls_name);
    let events = data.sequence.iter().map(|elem| {
        let elem_name = elem.name.clone();
        match elem.max_occurs {
            XsdElementMaxOccurs::Some(1) => {
                let name = quote::Ident::new(elem.name.clone());
                quote!(
                    if let Some(ref item) = self.#name {
                        try!(item.serialize_with(sink, #elem_name));
                    }
                )
            }
            _ => {
                let name = quote::Ident::new(format!("{}s", elem.name));
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
            
            sink.write(XmlEvent::EndElement { name: Some(elemname) });
        }
    );
    quote!(
        extern crate xml as _xml;

        use std::borrow::cow;
        use self::_xml::name::Name;
        use self::_xml::name::Namespace;
        use self::_xml::writer::{ XmlEvent, EventWriter };
        use gpx_rust::ser::Serialize;

        impl Serialize for #cls_name {
            #fun_body
        }
    ).to_string()
}

fn save(filename: &str, structs: Vec<StructInfo>) -> Result<(), io::Error> {
    let f = try!(File::create(filename));
    let mut f = BufWriter::new(f);
    for item in structs {
        try!(f.write(
            make_ser_impl(&item.name, item.type_).as_bytes()
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
    let types = vec![
        XsdType {
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
    ];
    
    let structs = vec![
        StructInfo { name: "Metadata".into(), type_: &types[0] }
    ];
    
    save(matches.value_of("destination").unwrap(), structs).expect("Failed to save");
}
