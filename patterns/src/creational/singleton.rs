use std::sync::{Mutex, Once, OnceLock};

/// RThis represent the Singleton object
pub struct Human {
    age: u32,
    name: String,
    sex: String,
}

/// Implementation of [instance] to retrieve always same instance created [ONCE]
/// Using static [Once] from rust, we ensure we have only one instance created during the life of the program.
/// Inside [get_or_init] guarantee the creation of just one instance.
impl Human {
    pub fn instance() -> &'static Self {
        static ONCE: OnceLock<Human> = OnceLock::new();

        ONCE.get_or_init(|| Human {
            age: 42,
            name: "Politrons".to_string(),
            sex: "Male".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::creational::singleton::Human;

    #[test]
    fn singleton_pattern() {
        let human = Human::instance();
        let same_human = Human::instance();
        println!("name is {}", human.name);
        println!("age is {}", human.age);
        println!(" {}", std::ptr::eq(human, same_human));
    }
}
