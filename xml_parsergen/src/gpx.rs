extern crate rustache;

use std;
use std::io;
use std::collections::HashMap;
use quote;
use self::rustache::{ HashBuilder, Render };

use xsd_types::{ Type, Element, ElementMaxOccurs, Attribute };
use ::{ ParserGen, TagMap, TypeConverter, TypeMap, ident_safe, UserType };


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

macro_rules! ElementSingle (
    ( $name:expr, $type_:expr ) => {
        Element { name: String::from($name),
                  type_: $type_.into(),
                  max_occurs: ElementMaxOccurs::Some(1) }
    }
);


pub fn get_types<'a>() -> HashMap<&'a str, Type> {
    map!{
        "gpxType".into() => Type {
            sequence: vec![
                ElementSingle!("metadata", "metadataType"),
                Element { name: "wpt".into(),
                          type_: "wptType".into(),
                          max_occurs: ElementMaxOccurs::Unbounded },
                //Element { name: "rte".into(),
                  //        type_: "rteType".into(),
                    //      max_occurs: ElementMaxOccurs::Unbounded },
                Element { name: "trk".into(),
                          type_: "trkType".into(),
                          max_occurs: ElementMaxOccurs::Unbounded },
                ElementSingle!("extensions", "extensionsType"),
            ],
            attributes: vec![
                Attribute { name: "version".into(), type_: "_gpx:version".into(), required: true },
                Attribute { name: "creator".into(), type_: "xsd:string".into(), required: true },
            ],
        },
        "metadataType".into() => Type {
            sequence: vec![
                Element { name: String::from("name"),
                          type_: "xsd:string".into(),
                          max_occurs: ElementMaxOccurs::Some(1) },
                Element { name: String::from("desc"),
                          type_: "xsd:string".into(),
                          max_occurs: ElementMaxOccurs::Some(1) },
                ElementSingle!("author", "personType"),
                ElementSingle!("copyright", "copyrightType"),
                Element { name: String::from("link"),
                          type_: "linkType".into(),
                          max_occurs: ElementMaxOccurs::Unbounded },
                ElementSingle!("time", "xsd:dateTime"),
                ElementSingle!("keywords", "xsd:string"),
                ElementSingle!("bounds", "boundsType"),
                ElementSingle!("extensions", "extensionsType"),
            ],
            attributes: vec![],
        },
        "trkType".into() => Type {
            sequence: vec![
                ElementSingle!("name", "xsd:string"),
                ElementSingle!("cmt", "xsd:string"),
                ElementSingle!("desc", "xsd:string"),
                ElementSingle!("src", "xsd:string"),
                Element { name: String::from("link"),
                          type_: "linkType".into(),
                          max_occurs: ElementMaxOccurs::Unbounded },
                ElementSingle!("number", "xsd:nonNegativeInteger"),
                ElementSingle!("type", "xsd:string"),
                ElementSingle!("extensions", "extensionsType"),
                Element { name: String::from("trkseg"),
                          type_: "trksegType".into(),
                          max_occurs: ElementMaxOccurs::Unbounded },
            ],
            attributes: vec![],
        },
        "trksegType".into() => Type {
            sequence: vec![
                Element { name: "trkpt".into(),
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
                ElementSingle!("ele", "xsd:decimal"),
                ElementSingle!("time", "xsd:dateTime"),
                ElementSingle!("magvar", "xsd:degreesType"),
                ElementSingle!("geoidheight", "xsd:decimal"),
                ElementSingle!("name", "xsd:string"),
                ElementSingle!("cmt", "xsd:string"),
                ElementSingle!("desc", "xsd:string"),
                ElementSingle!("src", "xsd:string"),
                Element { name: String::from("link"),
                          type_: "linkType".into(),
                          max_occurs: ElementMaxOccurs::Unbounded },
                ElementSingle!("sym", "xsd:string"),
                ElementSingle!("type", "xsd:string"),
                ElementSingle!("fix", "fixType"),
                ElementSingle!("sat", "xsd:nonNegativeInteger"),
                ElementSingle!("hdop", "xsd:decimal"),
                ElementSingle!("pdop", "xsd:decimal"),
                ElementSingle!("vdop", "xsd:decimal"),
                ElementSingle!("ageofdgpsdata", "xsd:decimal"),
                ElementSingle!("dgpsid", "dgpsStationType"),
                ElementSingle!("extensions", "extensionsType"),
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

use ser::{ Serialize, SerError, SerializeVia };
use gpx;
use gpx::*;
"
    }
    
    fn parser_cls(name: &str, data: &Type, types: &TypeMap)
            -> String {
        let cls_name = quote::Ident::new(name);
        let attrs = data.attributes.iter().map(|attr| {
            let fallback = UserType("String".into());
            let attr_type = match types.get(&attr.type_) {
                Some(&(ref type_, TypeConverter::AttributeFun(_))) => type_,
                Some(&(ref type_, TypeConverter::UniversalClass(_))) => type_,
                Some(_) => panic!("Type {} doesn't have converter appropriate for attribute", &attr.type_),
                None => {
                     println!("cargo:warning=\"Missing type for attr {}\"", &attr.type_);
                     &fallback
                }
            };
            quote::Ident::new(format!("{}: Option<{}>",
                                      ident_safe(&attr.name),
                                      attr_type.as_user_type()))
        });
        let elems = data.sequence.iter().map(|elem| {
            let fallback = UserType("XmlElement".into());
            let elem_type = match types.get(&elem.type_) {
                Some(&(ref cls, _)) => cls,
                None => {
                     println!("cargo:warning=\"Missing type for elem {}\"", &elem.type_);
                     &fallback
                }
            };
            let wrap_type = match elem.max_occurs {
                ElementMaxOccurs::Some(0) => panic!("Element has 0 occurrences, can't derive data type"),
                ElementMaxOccurs::Some(1) => "Option",
                _ => "Vec",
            };
            quote::Ident::new(format!("{}: {}<{}>",
                                      ident_safe(&elem.name),
                                      wrap_type,
                                      elem_type.as_user_type()))
        });
        quote!(
            pub struct #cls_name<'a, T: 'a + Read> {
                reader: &'a mut EventReader<T>,
                elem_name: Option<OwnedName>,
                #( #attrs, )*
                #( #elems, )*
            }
        ).to_string()
    }
    
    fn parser_impl(name: &str, data: &Type, types: &TypeMap) -> String {
        let cls_name = quote::Ident::new(name);
        let attrs = data.attributes.iter().map(|attr| {
            quote::Ident::new(attr.name.clone())
        });
        let macroattrs = data.attributes.iter().map(|attr| {
            let field = quote::Ident::new(attr.name.clone());
            let attr_name = &attr.name;
            let conv = quote::Ident::new(match types.get(&attr.type_) {
                Some(&(_, TypeConverter::AttributeFun(ref foo))) => foo.clone(),
                Some(&(_, TypeConverter::UniversalClass(ref conv_name))) => {
                    format!("{}::from_attr", conv_name)
                },
                Some(_) => panic!("Attribute {} must be parsed with a function", &attr.name),
                None => {
                    println!("No parser for {}", &attr.type_);
                    "FIXME".into()
                }
            });
            quote!(
                #attr_name => { #field, #conv }
            )
        });
        let elem_inits = data.sequence.iter().map(|elem| {
            quote::Ident::new(
                format!("{ident}: {init}",
                        ident=ident_safe(&elem.name),
                        init=match elem.max_occurs {
                            ElementMaxOccurs::Some(0) => {
                                panic!("Element has 0 occurrences, can't derive data type")
                            }
                            ElementMaxOccurs::Some(1) => "None",
                            _ => "Vec::new()"
                        })
            )
        });
        let match_elems = data.sequence.iter().map(|elem| {
            let field = ident_safe(&elem.name).clone();
            let tag = &elem.name;
            let saver = match elem.max_occurs {
                ElementMaxOccurs::Some(0) => {
                    println!("cargo:warning=\"Element has 0 occurrences, inserting panic on encounter.\"");
                    format!("panic!(\"Element {} should never appear\")", tag)
                },
                ElementMaxOccurs::Some(1) => format!("self.{} = Some", field),
                _ => format!("self.{}.push", field),
            };
            let conv = match types.get(&elem.type_) {
                Some(&(_, TypeConverter::ParseFun(ref fun))) => {
                    format!("{fun}(self.reader, elem_start)", fun=fun)
                },
                Some(&(_, TypeConverter::ParserClass(ref cls))) => {
                    format!("{cls}::new(self.reader).parse(elem_start)", cls=cls)
                },
                Some(&(_, TypeConverter::UniversalClass(ref conv_name))) => {
                    format!("{}::parse_via(self.reader, elem_start)", conv_name)
                }
                Some(&(_, TypeConverter::AttributeFun(_))) => {
                    panic!("Element {} has attribute conversion", &elem.type_)
                }
                None => panic!("Missing conversion for {}", &elem.type_),
            };
            quote::Ident::new(format!("{tag} => {{
                {saver}(try!({conv}));
            }}", tag=quote!(#tag), saver=saver, conv=conv))
        });
        let body = render_string(HashBuilder::new().insert("match_elems",
                                                           quote!( #( #match_elems, )* ).to_string()),
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
                {{{ match_elems }}}
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
        let body1 = quote::Ident::new(quote!(
            fn new(reader: &'a mut EventReader<T>) -> Self {
                    #cls_name { reader: reader,
                                elem_name: None,
                                #( #attrs: None, )*
                                #( #elem_inits, )* }
            }
            ParserStart!( #( #macroattrs ),* );
        ).to_string().replace("{", "{\n").replace(";", ";\n"));
        quote!(
            impl<'a, T: Read> ElementParse<'a, T> for #cls_name<'a, T> {
                #body1
                #body
            }
        ).to_string()
    }
    
    //fn build_impl(cls_name: &str, data: &Type, tage: &TagMap) -> String {
       // panic!("not implemented");
    //}

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
                    quote!(#type_name::serialize_via(item, sink, #elem_name))
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
