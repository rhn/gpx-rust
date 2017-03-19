/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub enum Type {
    Simple(SimpleType),
    Complex(ComplexType),
}

pub struct ComplexType {
    pub attributes: Vec<Attribute>,
    pub sequence: Vec<Element>,
}

pub struct SimpleType {
    pub base: String,
    pub min_inclusive: f64,
    pub max_inclusive: Option<f64>,
    pub max_exclusive: Option<f64>,
}

pub struct Element {
    pub name: String,
    pub type_: String,
    pub max_occurs: ElementMaxOccurs,
}

pub enum ElementMaxOccurs {
    Some(u64),
    Unbounded,
}

pub struct Attribute {
    pub name: String,
    pub type_: String,
    pub required: bool,
}
