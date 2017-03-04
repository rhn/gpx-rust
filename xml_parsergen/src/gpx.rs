extern crate rustache;

use std;
use std::io;
use std::collections::HashMap;
use quote;
use self::rustache::{ HashBuilder, Render };

use xsd_types::{ Type, Element, ElementMaxOccurs, Attribute };
use ::{ StructInfo, ParserGen, TagMap, TypeConverter, TypeMap, ident_safe, UserType };


fn render_string(data: HashBuilder, template: &str) -> String {
    let mut out = io::Cursor::new(Vec::new());
    data.render(template, &mut out).expect("Error in rendering");
    String::from_utf8(out.into_inner()).expect("Error in encoding") // FIXME: what's the encoding of the source?
}

fn get_elem_field_name(elem: &Element, tags: &TagMap) -> String {
    match tags.get(elem.name.as_str()) {
        Some(i) => String::from(*i),
        None => {
            match elem.max_occurs {
                ElementMaxOccurs::Some(1) => elem.name.clone(),
                _ => format!("{}s", elem.name)
            }
        }
    }
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
                Element { name: "rte".into(),
                          type_: "rteType".into(),
                          max_occurs: ElementMaxOccurs::Unbounded },
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
        "rteType".into() => Type {
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
                Element { name: String::from("rtept"),
                          type_: "wptType".into(),
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
    fn struct_def(name: &str, tags: &TagMap, data: &Type, type_convs: &TypeMap) -> String {
        let get_elem_field_name = |elem: &Element| {
            match tags.get(elem.name.as_str()) {
                Some(i) => String::from(*i),
                None => {
                    match elem.max_occurs {
                        ElementMaxOccurs::Some(1) => elem.name.clone(),
                        _ => format!("{}s", elem.name)
                    }
                }
            }
        };
        let fields = data.attributes.iter().fold(String::new(), |mut s, _| {
            s.push_str(&format!("x"));
            s
        }) + &data.sequence.iter().fold(String::new(), |mut s, elem| {
            let data_type = match type_convs.get(elem.type_.as_str()) {
                Some(&(ref type_, _)) => type_,
                None => panic!("No type found for field {} ({}) on {}", elem.name, elem.type_, name)
            };
            let field_type = match elem.max_occurs {
                ElementMaxOccurs::Some(0) => {
                    println!("cargo:warning=\"Element {} can repeat 0 times, skipping\"",
                             elem.name);
                    String::new()
                }
                ElementMaxOccurs::Some(1) => format!("Option<{}>", data_type.as_user_type()),
                _ => format!("Vec<{}>", data_type.as_user_type()),
            };
            s.push_str(&format!("    {}: {},\n",
                                get_elem_field_name(elem), field_type));
            s
        });
        render_string(HashBuilder::new().insert("name", name)
                                        .insert("fields", fields),
                      r#"
#[derive(Debug)]
struct {{{ name }}} {
{{{ fields }}}
}"#)
    }

    fn parser_cls(name: &str, data: &Type, types: &TypeMap) -> String {
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
            let field = &attr.name;
            let attr_name = &attr.name;
            let conv = match types.get(&attr.type_) {
                Some(&(_, TypeConverter::AttributeFun(ref foo))) => foo.clone(),
                Some(&(_, TypeConverter::UniversalClass(ref conv_name))) => {
                    format!("{}::from_attr", conv_name)
                },
                Some(_) => panic!("Attribute {} must be parsed with a function", &attr.name),
                None => {
                    println!("No parser for {}", &attr.type_);
                    "FIXME".into()
                }
            };
            format!("{attr_name} => {{ {field}, {conv} }},\n",
                    attr_name=quote!(#attr_name), field=field, conv=conv)
        }).collect::<String>();
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
                        }
                )
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
            format!("{tag} => {{
                {saver}(try!({conv}));
            }}\n", tag=quote!(#tag), saver=saver, conv=conv)
        }).collect::<String>();
        
        let parse_elem_body = if !match_elems.is_empty() {
            render_string(HashBuilder::new().insert("match_elems", match_elems),
                          r#"
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
            Ok(())"#)
        } else {
            String::from(r#"
            Err(Error::from(ElementError::from_free(_ElementError::UnknownElement(elem_start.name),
                                                    self.reader.position())))"#)
        };

        let body = quote!(
            fn new(reader: &'a mut EventReader<T>) -> Self {
                    #cls_name { reader: reader,
                                elem_name: None,
                                #( #attrs: None, )*
                                #( #elem_inits, )* }
            }
        ).to_string().replace("{", "{\n").replace(";", ";\n");
        render_string(HashBuilder::new().insert("cls_name", name)
                                        .insert("macro_attrs", macroattrs)
                                        .insert("parse_element_body", parse_elem_body)
                                        .insert("body", body),
                      r#"
impl<'a, T: Read> ElementParse<'a, T> for {{{ cls_name }}}<'a, T> {
    ParserStart!( {{{ macro_attrs }}} );
    {{{ body }}}
    fn parse_element(&mut self, elem_start: ElemStart)
            -> Result<(), Self::Error> {
        {{{ parse_element_body }}}
    }
    fn get_name(&self) -> &OwnedName {
        match &self.elem_name {
            &Some(ref i) => i,
            &None => panic!("Name was not set while parsing"),
        }
    }
    fn next(&mut self) -> Result<XmlEvent, xml::Error> {
        self.reader.next().map_err(xml::Error::Xml)
    }
}"#)
    }
    
    fn build_impl(parser_name: &str, data: &Type, struct_info: &StructInfo, type_convs: &TypeMap)
            -> String {
        let inits = &data.sequence.iter().map(|elem| {
            format!("{}: self.{},\n", get_elem_field_name(elem, &struct_info.tags), ident_safe(&elem.name))
        }).collect::<String>();
        render_string(HashBuilder::new().insert("parser_name", parser_name)
                                        .insert("struct_name", (&struct_info.name).as_str())
                                        .insert("error_name", "Error") // TODO: figure out how to handle the class
                                        .insert("inits", inits.as_str()),
                      r#"
impl<'a, T: Read> ElementBuild for {{{ parser_name }}}<'a, T> {
    type Element = {{{ struct_name }}};
    type Error = {{{ error_name }}};
    fn build(self) -> Result<Self::Element, Self::Error> {
        Ok({{{ struct_name }}} {
            {{{ inits }}}
        })
    }
}"#)
    }

    fn serializer_impl(cls_name: &str, tags: &TagMap,
                       type_name: &str, data: &Type, type_convs: &TypeMap) -> String {
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
                Some(&(_, TypeConverter::UniversalClass(ref conv_name))) => {
                    let type_name = quote::Ident::new(conv_name.clone());
                    quote!(#type_name::serialize_via(item, sink, #elem_name))
                },
                _ => quote!(item.serialize_with(sink, #elem_name))
            };
            match elem.max_occurs {
                ElementMaxOccurs::Some(1) => {
                    let name = get_attr_name(&|n| { String::from(n) });
                    quote!(
                        if let Some(ref item) = data.#name {
                            try!(#ser_call);
                        }
                    )
                }
                _ => {
                    let name = get_attr_name(&|n| { format!("{}s", n) });
                    quote!(
                        for item in &data.#name {
                            try!(#ser_call);
                        }
                    )
                }
            }
        });
        
        let conv_name = match type_convs.get(type_name) {
            Some(&(_, TypeConverter::UniversalClass(ref conv_name))) => conv_name.as_str(),
            _ => panic!("Refusing to create serializer for non-universal class {}", type_name)
        };
        render_string(HashBuilder::new().insert("cls_name", cls_name)
                                        .insert("conv_name", conv_name)
                                        .insert("events", quote!( #( #events )* ).to_string()),
                      r#"
        impl SerializeVia<{{{ cls_name }}}> for {{{ conv_name }}} {
            fn serialize_via<W: io::Write>(data: &{{{ cls_name }}}, sink: &mut EventWriter<W>, name: &str)
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
