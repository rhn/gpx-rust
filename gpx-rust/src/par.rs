extern crate xml as _xml;

use self::_xml::name::OwnedName;


/// Error classes in ElementParser must implement this
pub trait ParserMessage
        where Self: From<&'static str> {
    fn from_unexp_attr(elem_name: OwnedName, attr_name: OwnedName) -> Self;
    fn from_xml_error(_xml::reader::Error) -> Self;
    fn from_bad_attr_val(::gpx::par::AttributeValueError) -> Self;
}
