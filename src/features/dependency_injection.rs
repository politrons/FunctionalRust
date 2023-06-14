/// Contract of the dependency that I will receive
pub trait IdiomDependency {
    fn say_hello(&self);
}

/// Data type for Service, that define a field [Dependency] that
/// it can be any implementation of [IdiomDependency]
pub struct LanguageService {
    dependency: Box<dyn IdiomDependency>,
}

/// Implementation of the service that it require in the [new] constructor, pass the dependency
/// so then we can instantiate [LanguageService] passing the dependency
impl LanguageService {
    pub fn new(dependency: Box<dyn IdiomDependency>) -> Self {
        LanguageService { dependency }
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

    impl IdiomDependency for English {
        fn say_hello(&self) {
            println!("Hi mate");
        }
    }

    pub struct Spanish;

    impl IdiomDependency for Spanish {
        fn say_hello(&self) {
            println!("Hola amigo");
        }
    }

    #[test]
    fn dependency_injection() {
        let english = Box::new(English);
        let hello_service = LanguageService::new(english);
        hello_service.run();

        let spanish = Box::new(Spanish);
        let hola_service = LanguageService::new(spanish);
        hola_service.run()
    }
}