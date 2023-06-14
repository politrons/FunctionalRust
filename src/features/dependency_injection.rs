/// Contract of the dependency that I will receive
pub trait LangDependency {
    fn say_hello(&self);
}

/// Data type for Service, that define a field [Dependency] that
/// it can be any implementation of [LangDependency]
pub struct LangService {
    dependency: Box<dyn LangDependency>,
}

/// Implementation of the service that it require in the [new] constructor, pass the dependency
/// so then we can instantiate [LangService] passing the dependency
impl LangService {
    pub fn new(dependency: Box<dyn LangDependency>) -> Self {
        LangService { dependency }
    }

    pub fn run(&self) {
        self.dependency.say_hello();
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    /// Dependencies implementations
    pub struct English;

    impl LangDependency for English {
        fn say_hello(&self) {
            println!("Hi mate");
        }
    }

    pub struct Spanish;

    impl LangDependency for Spanish {
        fn say_hello(&self) {
            println!("Hola amigo");
        }
    }

    #[test]
    fn dependency_injection() {
        let english = Box::new(English);
        let hello_service = LangService::new(english);
        hello_service.run();

        let spanish = Box::new(Spanish);
        let hola_service = LangService::new(spanish);
        hola_service.run()
    }
}