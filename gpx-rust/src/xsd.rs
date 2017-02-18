extern crate chrono;
extern crate xml as _xml;
extern crate std;

use std::str::FromStr;
use std::borrow::Cow;
use self::chrono::{ DateTime, FixedOffset };
use self::_xml::name::Name;
use self::_xml::namespace::Namespace;
use self::_xml::attribute::Attribute;
use self::_xml::writer::XmlEvent;

use generator::{ Generator, make_gen };
use parsers::{ parse_chars, CharNodeError };
use xml::ElemStart;
use ser::Serialize;


pub type Time = DateTime<FixedOffset>;
pub type NonNegativeInteger = u64;


pub fn parse_int<T: std::io::Read, Error: CharNodeError + From<std::num::ParseIntError>>
        (mut parser: &mut _xml::EventReader<T>, elem_start: ElemStart)
        -> Result<NonNegativeInteger, Error> {
    parse_chars(parser, elem_start,
                |chars| NonNegativeInteger::from_str(chars)
    )
}

impl Serialize for Time {
    fn events<'a>(&'a self) -> Generator<XmlEvent<'a>> {
        let value = self.to_rfc3339();
        make_gen(move |ctx| {
            let elemname = Name::local("time");
            ctx.suspend(XmlEvent::StartElement {
                name: elemname.clone(),
                attributes: Cow::Owned(vec![]),
                namespace: Cow::Owned(Namespace::empty()),
            });
            ctx.suspend(XmlEvent::Characters(&value));
            ctx.suspend(XmlEvent::EndElement { name: Some(elemname) });
        })
    }
}
