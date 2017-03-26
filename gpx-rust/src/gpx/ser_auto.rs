/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate xml as _xml;

use std::borrow::Cow;

use self::_xml::attribute::{ Attribute, OwnedAttribute };
use self::_xml::name::OwnedName;
use self::_xml::namespace::Namespace;
use self::_xml::writer::{ XmlEvent, EventWriter };

use ser::{ Error, SerializeVia, ToCharsVia };
use gpx::*;

include!(concat!(env!("OUT_DIR"), "/gpx_ser_auto.rs"));
