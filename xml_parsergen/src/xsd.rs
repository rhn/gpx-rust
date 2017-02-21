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
