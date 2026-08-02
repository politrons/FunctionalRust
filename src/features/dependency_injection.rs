struct Dependency1 {
    dependency2: Dependency2,
    value:String
}

struct Dependency2 {
    value:String
}

impl Dependency1 {

    pub fn new(value:String, dependency2: Dependency2) -> Self {
        Dependency1{value,dependency2, }
    }

    pub fn print_value(&self){
        println!("{} {}",self.value,self.dependency2.value );
    }
}

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
    fn dependency_injection_basic() {
        let dependency2 = Dependency2{value:String::from("World")};
        let dependency1 = Dependency1::new(String::from("Hello"), dependency2);
        dependency1.print_value();
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