extern crate rustache;

use std;
use std::io;
use std::collections::HashMap;
use quote;
use self::rustache::{ HashBuilder, Render };

use xsd_types::{ Type, XsdElement, ElementMaxOccurs, Attribute };
use ::{ ParserGen, TagMap, ident_safe };


fn render_string(data: HashBuilder, template: &str) -> String {
    let mut out = io::Cursor::new(Vec::new());
    data.render(template, &mut out).expect("Error in rendering");
    String::from_utf8(out.into_inner()).expect("Error in encoding") // FIXME: what's the encoding of the source?
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

macro_rules! XsdElementSingle (
    ( $name:expr, $type_:expr ) => {
        XsdElement { name: String::from($name),
                     type_: $type_.into(),
                     max_occurs: ElementMaxOccurs::Some(1) }
    }
);


pub fn get_types<'a>() -> HashMap<&'a str, Type> {
    map!{
        "metadataType".into() => Type {
            sequence: vec![
                XsdElement { name: String::from("name"),
                             type_: "xsd:string".into(),
                             max_occurs: ElementMaxOccurs::Some(1) },
                XsdElement { name: String::from("desc"),
                             type_: "xsd:string".into(),
                             max_occurs: ElementMaxOccurs::Some(1) },
                XsdElementSingle!("author", "personType"),
                XsdElementSingle!("copyright", "copyrightType"),
                XsdElement { name: String::from("link"),
                             type_: "linkType".into(),
                             max_occurs: ElementMaxOccurs::Unbounded },
                XsdElementSingle!("time", "xsd:dateTime"),
                XsdElementSingle!("keywords", "xsd:string"),
                XsdElementSingle!("bounds", "boundsType"),
                XsdElementSingle!("extensions", "extensionsType"),
            ],
            attributes: vec![],
        },
        "trkType".into() => Type {
            sequence: vec![
                XsdElementSingle!("name", "xsd:string"),
                XsdElementSingle!("cmt", "xsd:string"),
                XsdElementSingle!("desc", "xsd:string"),
                XsdElementSingle!("src", "xsd:string"),
                XsdElement { name: String::from("link"),
                             type_: "linkType".into(),
                             max_occurs: ElementMaxOccurs::Unbounded },
                XsdElementSingle!("number", "xsd:nonNegativeInteger"),
                XsdElementSingle!("type", "xsd:string"),
                XsdElementSingle!("extensions", "extensionsType"),
                XsdElement { name: String::from("trkseg"),
                             type_: "trksegType".into(),
                             max_occurs: ElementMaxOccurs::Unbounded },
            ],
            attributes: vec![],
        },
        "trksegType".into() => Type {
            sequence: vec![
                XsdElement { name: "trkpt".into(),
                             type_: "wptType".into(),
                             max_occurs: ElementMaxOccurs::Unbounded },
            ],
            attributes: vec![],
        },
        "boundsType".into() => Type {
            sequence: vec![],
            attributes: vec![
                Attribute { name: "minlat".into(), type_: "latitudeType".into(), required: true },
                Attribute { name: "minlon".into(), type_: "longitudeType".into(), required: true },
                Attribute { name: "maxlat".into(), type_: "latitudeType".into(), required: true },
                Attribute { name: "maxlon".into(), type_: "longitudeType".into(), required: true },
            ],
        },
        "wptType".into() => Type {
            sequence: vec![
                XsdElementSingle!("ele", "xsd:decimal"),
                XsdElementSingle!("time", "xsd:dateTime"),
                XsdElementSingle!("magvar", "xsd:degreesType"),
                XsdElementSingle!("geoidheight", "xsd:decimal"),
                XsdElementSingle!("name", "xsd:string"),
                XsdElementSingle!("cmt", "xsd:string"),
                XsdElementSingle!("desc", "xsd:string"),
                XsdElementSingle!("src", "xsd:string"),
                XsdElement { name: String::from("link"),
                             type_: "linkType".into(),
                             max_occurs: ElementMaxOccurs::Unbounded },
                XsdElementSingle!("sym", "xsd:string"),
                XsdElementSingle!("type", "xsd:string"),
                XsdElementSingle!("fix", "fixType"),
                XsdElementSingle!("sat", "xsd:nonNegativeInteger"),
                XsdElementSingle!("hdop", "xsd:decimal"),
                XsdElementSingle!("pdop", "xsd:decimal"),
                XsdElementSingle!("vdop", "xsd:decimal"),
                XsdElementSingle!("ageofdgpsdata", "xsd:decimal"),
                XsdElementSingle!("dgpsid", "dgpsStationType"),
                XsdElementSingle!("extensions", "extensionsType"),
            ],
            attributes: vec![
                Attribute { name: "lat".into(), type_: "latitudeType".into(), required: true },
                Attribute { name: "lon".into(), type_: "longitudeType".into(), required: true }
            ],
        }
    }
}


pub struct Generator {}

trait GetOrElse<K, V> {
    fn get_or_else(&self, key: &K, f: &Fn(&K) -> V) -> V;
}

impl<'a, K: std::cmp::Eq + std::hash::Hash, V: Clone> GetOrElse<K, V> for HashMap<K, V> {
    fn get_or_else(&self, key: &K, f: &Fn(&K) -> V) -> V {
        match self.get(key) {
            Some(i) => V::clone(i),
            None => f(key),
        }
    }
}


impl ParserGen for Generator {
    fn header() -> &'static str {
        "extern crate xml as _xml;

use std::borrow::Cow;
use self::_xml::writer;
use self::_xml::name::Name;
use self::_xml::namespace::Namespace;
use self::_xml::writer::{ XmlEvent, EventWriter };

use ser::{ Serialize, SerError };
use gpx::ser::SerializeVia;
use gpx;
use gpx::*;
"
    }
    
    fn parser_cls(name: &str, data: &Type, types: &HashMap<String, (String, String)>) -> String {
        let cls_name = quote::Ident::new(name);
        let attrs = data.attributes.iter().map(|attr| {
            let type_ = quote::Ident::new(
                match types.get(&attr.type_) {
                    Some(a) => &a.0,
                    None => {
                         println!("Missing type for attr {}", &attr.type_);
                         "String"
                    }
                }.clone()
            );
            let name = quote::Ident::new(attr.name.clone());
            quote!(
                #name : Option< #type_ >
            )
        });
        let elems = data.sequence.iter().map(|elem| {
            let elem_name = &elem.name;
            let elem_type = match types.get(&elem.type_) {
                Some(a) => &a.0,
                None => {
                     println!("Missing type for elem {}", &elem.type_);
                     "XmlElement"
                }
            }.clone();
            let wrap_type = match elem.max_occurs {
                ElementMaxOccurs::Some(0) => panic!("Element has 0 occurrences, can't derive data type"),
                ElementMaxOccurs::Some(1) => "Option",
                _ => "Vec",
            };
            quote::Ident::new(format!("{}: {}<{}>", ident_safe(elem_name), wrap_type, elem_type))
        });
        quote!(
            struct #cls_name<'a, T: 'a + Read> {
                reader: &'a mut EventReader<T>,
                elem_name: Option<OwnedName>,
                #( #attrs, )*
                #( #elems, )*
            }
        ).to_string()
    }
    
    fn parser_impl(name: &str, data: &Type, types: &HashMap<String, (String, String)>) -> String {
        let cls_name = quote::Ident::new(name);
        let attrs = data.attributes.iter().map(|attr| {
            quote::Ident::new(attr.name.clone())
        });
        let macroattrs = data.attributes.iter().map(|attr| {
            let field = quote::Ident::new(attr.name.clone());
            let attr_name = &attr.name;
            let conv = quote::Ident::new(
                types.get(&attr.type_)
                     .unwrap_or(&(String::new(), "Result::Ok<String, _>".into()))
                     .1
                     .clone()
            );
            quote!(
                #attr_name => { #field, #conv }
            )
        });
        let elems = data.sequence.iter().map(|elem| {
            quote::Ident::new(ident_safe(&elem.name).clone())
        });
        let macroelems = data.sequence.iter().map(|elem| {
            let field = quote::Ident::new(ident_safe(&elem.name).clone());
            let tag = &elem.name;
            let conv = quote::Ident::new(
                match types.get(&elem.type_) {
                    Some(t) => &t.1,
                    None => {
                        println!("Missing conversion for {}", &elem.type_);
                        "parse_elem"
                    }.clone()
                }
            );
            quote!(
                #tag => {
                    self.#field = Some(try!(#conv(self.reader, elem_start)));
                }
            )
        });
        let body = render_string(HashBuilder::new().insert("match_code",
                                                           quote!( #( #macroelems, )* ).to_string()),
                                 r#"
        fn parse_element(&mut self, elem_start: ElemStart)
                -> Result<(), Self::Error> {
            if let Some(ref ns) = elem_start.name.namespace.clone() {
                match &ns as &str {
                    "http://www.topografix.com/GPX/1/1" => (),
                    "http://www.topografix.com/GPX/1/0" => {
                        println!("WARNING: GPX 1.0 not fully supported, errors may appear");
                    },
                    ns => {
                        {
                            let name = &elem_start.name;
                            println!("WARNING: unknown namespace ignored on {:?}:{}: {}",
                                 name.prefix,
                                 name.local_name,
                                 ns);
                        }
                        try!(ElementParser::new(self.reader).parse(elem_start));
                        return Ok(());
                    }
                }
            }
            match &elem_start.name.local_name as &str {
                {{{ match_code }}}
                _ => {
                    // TODO: add config and handler
                    return Err(Error::from(
                        ElementError::from_free(_ElementError::UnknownElement(elem_start.name),
                                                self.reader.position())));
                }
            };
            Ok(())
        }
        
        fn get_name(&self) -> &OwnedName {
            match &self.elem_name {
                &Some(ref i) => i,
                &None => panic!("Name was not set while parsing"),
            }
        }
        fn next(&mut self) -> Result<XmlEvent, xml::Error> {
            self.reader.next().map_err(xml::Error::Xml)
        }"#);
        let body = quote::Ident::new(body);
        quote!(
            impl<'a, T: Read> ElementParse<'a, T> for #cls_name<'a, T> {
                fn new(reader: &'a mut EventReader<T>) -> Self {
                    #cls_name { reader: reader,
                                elem_name: None,
                                #( #attrs: None, )*
                                #( #elems: None, )* }
                }
                ParserStart!( #( #macroattrs ),* );
                #body
            }
        ).to_string().replace("{", "{\n").replace(";", ";\n")
    }

    fn serializer_impl(cls_name: &str, tags: &TagMap, data: &Type,
                       type_convs: &HashMap<String, String>) -> String {
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
            let ser_call = match type_convs.get(&elem.type_) {
                Some(name) => {
                    let type_name = quote::Ident::new(name.clone());
                    quote!(#type_name::serialize_via(&item, sink, #elem_name))
                },
                None => quote!(item.serialize_with(sink, #elem_name))
            };
            match elem.max_occurs {
                ElementMaxOccurs::Some(1) => {
                    let name = get_attr_name(&|n| { String::from(n) });
                    quote!(
                        if let Some(ref item) = self.#name {
                            try!(#ser_call);
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
        render_string(HashBuilder::new().insert("cls_name", cls_name)
                                        .insert("events", quote!( #( #events )* ).to_string()),
                      r#"
        impl Serialize for {{{ cls_name }}} {
            fn serialize_with<W: io::Write>(&self, sink: &mut EventWriter<W>, name: &str)
                    -> Result<(), SerError> {
                let elemname = Name::local(name);
                try!(sink.write(
                    XmlEvent::StartElement {
                        name: elemname.clone(),
                        attributes: Cow::Owned(vec![]),
                        namespace: Cow::Owned(Namespace::empty())
                    }
                ));
                
                {{{ events }}}
                
                try!(sink.write(XmlEvent::EndElement { name: Some(elemname) }));
                Ok(())
            }
        }"#)
    }
}
