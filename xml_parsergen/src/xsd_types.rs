pub struct Type {
    pub attributes: Vec<Attribute>,
    pub sequence: Vec<XsdElement>,
}

pub struct XsdElement {
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
