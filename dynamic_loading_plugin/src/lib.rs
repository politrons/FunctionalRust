use dynamic_loading_contract::{PluginTrait};

/// In order to create a dynamic library that can be used in runtime discovery we need to use
/// crate-type = ["cdylib"] in our [Cargo.toml] file. Then it will create a .[dylib] extension file

pub struct PluginA;

///Implementation of the contract [X] defined as an external dependency, we share with [dynamic_loading] cargo.
impl PluginTrait for PluginA {
    fn perform_action(&self) {
        println!("Plugin A action");
    }
}

///For this patter, we use "dynamic objects", which typically refers to objects whose types
/// are not known at compile time and can vary at runtime.
/// The [Box] it will expose all functions defined in the [Trait].
/// Use [pub extern "C"] is mandatory to create a reference that can be used by consumer of this library,
/// to discover the function in runtime loading.
#[no_mangle]
pub extern "C" fn create_trait_wrapper() -> Box<dyn PluginTrait> {
    Box::new(PluginA {  })
}



