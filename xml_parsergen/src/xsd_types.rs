pub struct Type<'a> {
    pub attributes: Vec<Attribute>,
    pub sequence: Vec<XsdElement<'a>>,
}

pub struct XsdElement<'a> {
    pub name: String,
    pub type_: XsdElementType<'a>,
    pub max_occurs: ElementMaxOccurs,
}

pub enum XsdElementType<'a> {
    Name(String),
    Type_(&'a Type<'a>)
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
