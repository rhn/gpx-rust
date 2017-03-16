extern crate xml as xml_;
extern crate chrono;


macro_rules! _parser_attr {
    ( $self_:ident, $value:ident, { $field:ident, $func:expr } ) => {
        $self_.$field = Some(try!($func($value)));
    };
    ( $self_:ident, $value:ident, { $field:ident, $func:path } ) => {
        $self_.$field = Some(try!($func($value).map_err(Self::Error::from_bad_attr_val)));
    };
}

#[macro_export]
macro_rules! ParserStart {
    ( $( $name:pat => $attr:tt ),* $(,)* ) => {
        fn parse_start(&mut self, elem_start: ElemStart)
                -> Result<(), ::xml::AttributeError> {
            for attr in elem_start.attributes {
                let name = attr.name;

                if let &Some(ref ns) = &name.namespace {
                    match &ns as &str {
                        "http://www.topografix.com/GPX/1/1" |
                        "http://www.topografix.com/GPX/1/0" => (),
                        ns => {
                            println!("WARNING: namespace ignored on {:?}:{}: {}",
                                 name.prefix,
                                 name.local_name,
                                 ns);
                            continue;
                        }
                    }
                }
                match &(name.local_name) as &str {
                    $( $name => {
                        let v = &attr.value;
                        _parser_attr! { self, v, $attr }
                    } ),*
                    _ => {
                        return Err(::xml::AttributeError::Unexpected(name));
                    }
                }
            }
            self.elem_name = Some(elem_start.name);
            Ok(())
        }
    };
}
