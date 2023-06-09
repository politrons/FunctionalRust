extern crate libloading;

use libloading::{Library, Symbol};
use dynamic_loading_contract::PluginTrait;

fn main() {
    unsafe {
        // Load the shared object file
        let lib = Library::new("../dynamic_loading_plugin/target/release/libdynamic_loading_plugin.dylib").unwrap();

        // Use a symbol from the shared object file
        let create_wrapper: Symbol<extern "C" fn() -> Box<dyn PluginTrait>> = lib.get(b"create_trait_wrapper\0").unwrap();
        // Convert the symbol to a function pointer
        let wrapper = create_wrapper();
        // Run the action
        wrapper.perform_action();
    }

    // The shared object file is unloaded automatically when 'lib' goes out of scope
}
