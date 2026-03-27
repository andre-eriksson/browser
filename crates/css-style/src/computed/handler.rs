#[macro_export]
macro_rules! compute {
    ($spec:expr, $parent:expr, $field:ident) => {
        $spec.$field.compute($parent.$field)
    };
}

#[macro_export]
macro_rules! clone_compute {
    ($spec:expr, $parent:expr, $field:ident) => {
        $spec.$field.compute($parent.$field.clone())
    };
}

#[macro_export]
macro_rules! into_compute {
    ($spec:expr, $parent:expr, $field:ident) => {
        $spec.$field.compute($parent.$field.into())
    };
}

#[macro_export]
macro_rules! compute_px {
    ($spec:expr, $parent:expr, $field:ident, $type:ident) => {
        $spec.$field.compute($type::px($parent.$field))
    };
}

#[macro_export]
macro_rules! compute_parent_px {
    ($spec:expr, $parent:expr, $field:ident, $parent_field:ident, $type:ident) => {
        $spec.$field.compute($type::px($parent.$parent_field))
    };
}
