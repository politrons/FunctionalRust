mod features;

use crate::features::{
    async_programming, channels_feature, do_notation_style
    , new_types
    , try_monad,
};

fn main() {
    try_monad::run();
    async_programming::run();
    channels_feature::run();
    new_types::run();
    do_notation_style::run();
    crate::features::testing_patterns::run();
    crate::features::cargo_workspace::run();
    crate::features::performance_memory::run();
}
