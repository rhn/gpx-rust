/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate xml as _xml;

use std::io::Read;
use std::error::Error as ErrorTrait;

use self::_xml::reader::EventReader;
use self::_xml::name::OwnedName;
use self::_xml::attribute::OwnedAttribute;

use par::{ ParseVia, Positioned, ElementParse, ElementBuild };

use xml;
use xml::conv;
use xml::{ Element, Node };

use gpx::par::Error;

#[derive(Debug)]
pub enum BuildError {
    Custom(Box<ErrorTrait>)
}

pub struct ElementParser {
    name: Option<OwnedName>, // Using reference intentionally - this code does not need to interact with Name
    attributes: Vec<OwnedAttribute>,
    nodes: Vec<Node>,
}

impl ElementBuild for ElementParser {
    type Element = Element;
    type BuildError = BuildError;
    fn build(self) -> Result<Self::Element, Self::BuildError> {
        Ok(Element {
            name: self.name.unwrap().to_owned(),
            attributes: self.attributes,
            nodes: self.nodes
        })
    }
}

impl ElementParse<::gpx::par::Error> for ElementParser {
    fn new() -> ElementParser {
        ElementParser { name: None,
                        attributes: Vec::new(),
                        nodes: Vec::new() }
    }
    fn parse_start(&mut self, name: &OwnedName, attributes: &[OwnedAttribute])
            -> Result<(), ::par::AttributeError<::gpx::par::Error>> {
        self.name = Some(name.clone());
        let _ = attributes; // FIXME: break if attributes present
        Ok(())
    }
    fn parse_element<'a, R: Read>(&mut self, reader: &'a mut EventReader<R>,
                                  name: &OwnedName, attributes: &[OwnedAttribute])
            -> Result<(), Positioned<::gpx::par::Error>> {
        let elem = try!(ElementParser::new().parse(name, attributes, reader));
        self.nodes.push(Node::Element(elem));
        Ok(())
    }
    fn parse_characters(&mut self, data: String) -> Result<(), ::gpx::par::Error> {
        self.nodes.push(Node::Text(data));
        Ok(())
    }
    fn get_name(&self) -> &OwnedName {
        match &self.name {
            &Some(ref i) => i,
            &None => unreachable!(),
        }
    }
}

impl ParseVia<xml::Element> for conv::Element {
    fn parse_via<R: Read>(parser: &mut EventReader<R>,
                              name: &OwnedName, attributes: &[OwnedAttribute])
            -> Result<xml::Element, Positioned<Error>> {
        ElementParser::new().parse(name, attributes, parser)
    }
}
