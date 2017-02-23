use std::collections::HashMap;
use quote;

use xsd_types::{ XsdType, XsdElement, XsdElementType, XsdElementMaxOccurs };
use ::{ ParserGen, TagMap };


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

macro_rules! XsdElementSingle (
    ( $name:expr, $type_:expr ) => {
        XsdElement { name: String::from($name),
                     type_: XsdElementType::Name(String::from($type_)),
                     max_occurs: XsdElementMaxOccurs::Some(1) }
    }
);


pub fn get_types<'a, 'b>() -> HashMap<&'a str, XsdType<'b>> {
    map!{
        "metadataType".into() => XsdType {
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
        "trkType".into() => XsdType {
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
        "trksegType".into() => XsdType {
            sequence: vec![
                XsdElement { name: "trkpt".into(),
                             type_: XsdElementType::Name("wptType".into()),
                             max_occurs: XsdElementMaxOccurs::Unbounded },
            ]
        },
    }
}


pub struct Generator {}


impl ParserGen for Generator {
    fn header() -> &'static str {
        "extern crate xml as _xml;

use std::borrow::cow;
use self::_xml::name::Name;
use self::_xml::name::Namespace;
use self::_xml::writer::{ XmlEvent, EventWriter };
use gpx_rust::ser::Serialize;"
    }

    fn serializer_impl(cls_name: &str, tags: &TagMap, data: &XsdType) -> String {
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
}
