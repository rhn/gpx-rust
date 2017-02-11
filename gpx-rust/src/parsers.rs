extern crate xml as xml_;
extern crate chrono;

use std;
use self::xml_::reader::{ EventReader, XmlEvent };
use self::xml_::name::OwnedName;
use xml::ElemStart;
#[allow(unused_imports)]
use xml::{ ElementParser, ElementParse };


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

macro_rules! _parser_attr {
    ( $self_:ident, $value:ident, { $field:ident, $func:expr } ) => {
        $self_.$field = Some(try!($func($value)));
    };
    ( $self_:ident, $value:ident, { $field:ident, $func:path } ) => {
        $self_.$field = Some(try!($func($value)));
    };
}

#[macro_export]
macro_rules! ParserStart {
    ( $( $name:pat => $attr:tt ),* ) => {
        fn parse_start(&mut self, elem_start: ElemStart)
                -> Result<(), Self::Error> {
            for attr in elem_start.attributes {
                match &(attr.name.local_name) as &str {
                    $( $name => {
                        let v = attr.value;
                        _parser_attr! { self, v, $attr }
                    } ),*
                    _ => {
                        return Err(Self::Error::from_unexp_attr(elem_start.name, attr.name));
                    }
                }
            }
            self.elem_name = Some(elem_start.name);
            Ok(())
        }
    };
}

macro_rules! make_fn {
    ( fn, $parser:tt<$T:ty>, $reader:expr, $elem_start:expr ) => {
        $parser($reader, $elem_start);
    };
    ( ElementParse, $parser:ty, $reader:expr, $elem_start:expr ) => {
        <$parser>::new($reader).parse($elem_start);
    };
}

macro_rules! make_tag {
    ( $T:ty, $self_:expr, $elem_start:expr, { $field:ident = Some, $ptype:tt, $parser:tt } ) => {
        $self_.$field = Some(try!(make_fn!($ptype, $parser<$T>, $self_.reader, $elem_start)));
    };
    ( $T:ty, $self_:expr, $elem_start:expr, { $field:ident = Vec, $ptype:tt, $parser:tt } ) => {
        $self_.$field.push(try!(make_fn!($ptype, $parser<$T>, $self_.reader, $elem_start)));
    };
}

macro_rules! _ParserImplBody {
    (
        attrs: { $( $attr:pat => $attrdata:tt ),* },
        tags: { $( $tag:pat => $tagdata:tt, )* }
    ) => {
        ParserStart!( $( $attr => $attrdata ),* );

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

macro_rules! _elem_field {
    ( One <$t:ty> ) => { $t };
    ( $k:ident <$t:ty> ) => { $k <$t> };
}

macro_rules! Elem {
    ((),
        then $cb:tt,
        $(#[$($attrs:tt)*])*
        $(pub)* struct $name:ident {
            $( $i:ident : $k:tt <$t:ty>, )*
        }
    ) => {
        macro_attr_callback! {
            $cb,
            $(#[$($attrs)*])*
            pub struct $name {
                $( $i : _elem_field!( $k <$t> ), )*
            }
        }
    };
}

macro_rules! _parser_field {
    ( One <$t:ty> ) => { Option<$t> };
    ( $k:ident <$t:ty> ) => { $k <$t> };
}

macro_rules! Parser {
    (
        ( $parser:ident {
            attrs: { $( $attr:pat => $attrdata:tt ),* },
            tags: { $( $tag:pat => $tagdata:tt ),* $(,)* }
        })
        $(pub)* struct $name:ident {
            $( $i:ident : $k:tt <$t:ty>, )*
        }
    ) => {
        struct $parser<'a, T: 'a + Read> {
            reader: &'a mut EventReader<T>,
            elem_name: Option<OwnedName>,
            $( $i : _parser_field!{ $k <$t> }, )*
        }

        impl<'a, T: Read> ElementParse<'a, T> for $parser<'a, T> {
            fn new(reader: &'a mut EventReader<T>) -> Self {
                $parser { reader: reader,
                          elem_name: None,
                          $( $i : <_parser_field!{ $k <$t> }>::empty(), )* }
            }
            _ParserImplBody!( attrs: { $( $attr => $attrdata, )* },
                              tags: { $( $tag => $tagdata, )* } );
        }
    }
}

macro_rules! ParserExp {
    (
        ( $parser:ident {
            attrs: { $( $attr:pat => $attrdata:tt ),* },
            tags: { $( $tag:pat => $tagdata:tt ),* }
        })
        $(pub)* struct $name:ident {
            $( $i:ident : $k:tt ! ($t:ty), )*
        }
    ) => {
        struct $parser<'a, T: 'a + Read> {
            reader: &'a mut EventReader<T>,
            elem_name: Option<OwnedName>,
            $( $i : _parser_field!{ $k <$t> }, )*
        }

        impl<'a, T: Read> ElementParse<'a, T> for $parser<'a, T> {
            fn new(reader: &'a mut EventReader<T>) -> Self {
                $parser { reader: reader,
                          elem_name: None,
                          $( $i : <_parser_field!{ $k <$t> }>::empty(), )* }
            }
            _ParserImplBody!( attrs: { $( $attr => $attrdata ),* },
                              tags: { $( $tag => $tagdata, )* } );
        }
    }
}

macro_rules! ElementBuild {
    (
        ($parsername:ident, $error:ident)
        $(pub)* struct $name:ident {
            $( $i:ident : $t:ty, )*
        }
    ) => {
        impl<'a, T: Read> ElementBuild for $parsername<'a, T> {
            type Element = $name;
            type Error = $error;
            fn build(self) -> Result<Self::Element, Self::Error> {
                Ok($name { $( $i: self.$i ),* })
            }
        }
    }
}

macro_rules! One( ( $t:ty ) => { $t } );
macro_rules! Option( ( $t:ty ) => { Option<$t> } );
macro_rules! Vec( ( $t:ty ) => { Vec<$t> } );
