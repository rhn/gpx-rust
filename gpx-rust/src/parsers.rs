extern crate xml as xml_;
extern crate chrono;

use std;
use self::xml_::reader::{ EventReader, XmlEvent };
use self::xml_::name::OwnedName;
use xml::ElemStart;
#[allow(unused_imports)]
use xml::{ ElementParser, ElementParse };
use xsd;


pub trait ParserMessage
        where Self: From<&'static str> {
    fn from_unexp_attr(elem_name: OwnedName, attr_name: OwnedName) -> Self;
    fn from_xml_error(xml_::reader::Error) -> Self;
}

 
pub fn parse_chars<T: std::io::Read, F, R, E: ParserMessage>
    (mut parser: &mut EventReader<T>, elem_start: ElemStart, decode: F)
    -> Result<R, E>
        where F: Fn(&str) -> Result<R, E> {
    let mut ret = None;
    loop {
        match parser.next() {
            Ok(XmlEvent::Characters(data)) => {
                ret = Some(try!(decode(&data)));
            }
            Ok(XmlEvent::EndElement { name }) => {
                if name == elem_start.name {
                    return match ret {
                        Some(c) => Ok(c),
                        None => Err(E::from("Missing data"))
                    }
                }
                return Err(E::from("Unexpected end"));
            }
            Ok(XmlEvent::Whitespace(s)) => {
                println!("{:?}", s);
            }
            Ok(ev) => {
                println!("{:?}", ev);
                return Err(E::from("Unexpected event"));
            }
            Err(error) => {
                return Err(E::from_xml_error(error));
            }
        }
    }
}

pub fn parse_time<T: std::io::Read,
                  Error: ParserMessage + From<chrono::format::ParseError>>
        (mut parser: &mut EventReader<T>, elem_start: ElemStart)
        -> Result<xsd::Time, Error> {
    parse_chars(parser, elem_start,
                |chars| xsd::Time::parse_from_rfc3339(chars).map_err(Error::from))
}

macro_rules! Parser {
    (
        ($parser:ident {
            $( $tag:pat => $tagdata:tt, )*
        })
        $(pub)* struct $name:ident {
            $( $i:ident : $t:ty, )*
        }
    ) => {
        struct $parser<'a, T: 'a + Read> {
            reader: &'a mut EventReader<T>,
            elem_name: Option<OwnedName>,
            $( $i: $t, )*
        }

        impl<'a, T: Read> ElementParse<'a, T> for $parser<'a, T> {
            fn new(reader: &'a mut EventReader<T>) -> Self {
                $parser { reader: reader,
                          elem_name: None,
                          $( $i : <$t>::empty(), )* }
            }

            ParserStart!();

            fn parse_element(&mut self, elem_start: ElemStart)
                    -> Result<(), Self::Error> {
                match &elem_start.name.local_name as &str {
                    $( $tag => {
                        make_tag!(T, self, elem_start, $tagdata);
                    }),*
                    _ => {
                        try!(ElementParser::new(self.reader).parse(elem_start));
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
            }
        }
    }
}
