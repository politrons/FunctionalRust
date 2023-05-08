use crate::features::enum_type::RustTypes::IntegerR;

pub fn run() {
    let x = IntegerR("1981");
}

enum RustTypes<T> {
    IntegerR(T),
    StringR(T),
    BooleanR(T),
}