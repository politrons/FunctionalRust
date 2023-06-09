extern crate libloading;

use libloading::{Library, Symbol};
use dynamic_loading_contract::PluginTrait;

/// Using [libloading] library we're able to load Trait implementations in runtime, without have to know the specific type
/// that implement the trait, allowing same pattern than in Java Service Provider Interface(SPI).
/// This allow inject in a program different implementations just adding the path of the [.dylib] file that contains
/// the implementation, and knowing the name of the [symbol] created in that library.
///
/// [Library::new] oad the shared object file
/// [lib.get(b"your_symbol_name\0")] Use a symbol from the shared object file and return a [Symbol] function type
/// [plugin_symbol_func()] we run the function so we get the dynamic object Box<dyn PluginTrait>
/// [plugin.hello_world()] we run the action
///  The shared object file is unloaded automatically when 'lib' goes out of scope
fn main() {
    unsafe {
        let lib = Library::new("../dynamic_loading_plugin/target/release/libdynamic_loading_plugin.dylib").unwrap();
        let plugin_symbol_func: Symbol<extern "C" fn() -> Box<dyn PluginTrait>> = lib.get(b"create_plugin\0").unwrap();
        let plugin = plugin_symbol_func();
        plugin.hello_world();
    }
}
