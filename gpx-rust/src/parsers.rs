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
