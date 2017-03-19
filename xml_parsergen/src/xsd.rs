/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub struct XsdType<'a> {
    pub sequence: Vec<XsdElement<'a>>,
}

pub struct XsdElement<'a> {
    pub name: String,
    pub type_: XsdElementType<'a>,
    pub max_occurs: XsdElementMaxOccurs,
}

pub enum XsdElementType<'a> {
    Name(String),
    Type_(&'a XsdType<'a>)
}

pub enum XsdElementMaxOccurs {
    Some(u64),
    Unbounded,
}
