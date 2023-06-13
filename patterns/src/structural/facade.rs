/// Data type to implement a human component
pub struct Human;

/// Implementation of human data type
impl Human {
    pub fn speak(&self) {
        println!("Hello there");
    }
}

/// Data type to implement a dog component
pub struct Dog;

/// Implementation of dog data type
impl Dog {
    pub fn speak(&self) {
        println!("Barf");
    }
}

/// Facade pattern allow to gather multiple component executions, orchestate the order of execution,
/// and made that complexity agnostic for the one that use this [Facade]
/// We contain all dependencies that we want to execute
pub struct Facade {
    human: Human,
    dog: Dog,
}

/// Implementation of [Facade] patter.
/// with [new] We create internally all dependencies that we want to execute.
/// with [animal_speak] we orchestrate the execution of all animals we have as dependencies
/// completely agnostic for the client.
impl Facade {
    pub fn new() -> Self {
        Facade {
            human: Human,
            dog: Dog,
        }
    }

    pub fn animal_speak(&self) {
        self.human.speak();
        self.dog.speak();
    }
}


#[cfg(test)]
mod tests {
    use crate::structural::facade::{Human, Dog, Facade};

    #[test]
    fn decorator_pattern() {
       let facade = Facade::new();
        facade.animal_speak();
    }
}