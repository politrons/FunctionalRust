use std::sync::{Mutex, Once};

/// RThis represent the Singleton object
pub struct Human {
    age: u32,
    name: String,
    sex: String,
}

/// Implementation of [instance] to retrieve always same instance created [ONCE]
/// Using static [Once] from rust, we ensure we have only one instance created during the life of the program.
/// We use [*const Human] to have a [lazy_static] creation of the type. This it will require we create an instance wrapped in
/// [Box::into_raw] of a [Box::new] of [Human]
/// Inside [call_once] we do the construction of the Singleton Box type.
impl Human {
    pub fn instance() -> &'static Self {
        static mut INSTANCE: *const Human = 0 as *const Human;
        static ONCE: Once = Once::new();

        unsafe {
            ONCE.call_once(|| {
                println!("Instantiating Singleton...");
                INSTANCE = Box::into_raw(Box::new(Human {
                    age: 42,
                    name: "Politrons".to_string(),
                    sex: "Male".to_string(),
                }));
            });
            &*INSTANCE
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::singleton::Human;

    #[test]
    fn singleton_pattern() {
        let human = Human::instance();
        let same_human = Human::instance();
    }
}
