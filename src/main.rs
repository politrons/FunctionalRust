mod features;

use crate::features::{
    async_programming, channels_feature, collection, currying_function, do_notation_style,
    effect_system, either_monad, extension_method, functions, memory_management, monad, new_types,
    pattern_matching, smart_pointer, try_monad, type_classes,
};

fn main() {
    try_monad::run();
    extension_method::run();
    async_programming::run();
    channels_feature::run();
    new_types::run();
    do_notation_style::run();
    crate::features::unsafe_rust::run();
    crate::features::testing_patterns::run();
    crate::features::cargo_workspace::run();
    crate::features::performance_memory::run();
}
