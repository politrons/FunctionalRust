///The trait contract to be used by the [plugin] to create the implementation,
/// and by the [dynamic_loading] cargo to load in runtime any implementation of this contract
pub trait PluginTrait {
    fn perform_action(&self);
}

