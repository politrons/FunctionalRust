use dynamic_loading_contract::{PluginTrait};

pub struct PluginA;

impl PluginTrait for PluginA {
    fn perform_action(&self) {
        println!("Plugin A action");
    }
}

#[no_mangle]
pub extern "C" fn create_trait_wrapper() -> Box<dyn PluginTrait> {
    Box::new(PluginA {  })
}



