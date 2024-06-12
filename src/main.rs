mod features;

use crate::features::{async_programming, channels_feature, collection, currying_function, do_notation_style, effect_system, either_monad, extension_method, functions, memory_management, monad, new_types, pattern_matching, smart_pointer, try_monad, type_classes};

fn main() {
    functions::run();
    monad::run();
    type_classes::run();
    try_monad::run();
    effect_system::run();
    extension_method::run();
    memory_management::run();
    collection::run();
    async_programming::run();
    channels_feature::run();
    pattern_matching::run();
    either_monad::run();
    smart_pointer::run();
    new_types::run();
    do_notation_style::run();
}
