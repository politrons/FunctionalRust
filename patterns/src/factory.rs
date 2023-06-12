///Trait contract of the feature we want to implement
pub trait Animal {
    fn speak(&self);
}

/// Data type of the implementation of trait
pub struct Human;

/// Implementation of the trait
impl Animal for Human{
    fn speak(&self) {
        println!("Hello there")
    }
}

/// Trait contract of the factory
pub trait Factory {
    fn build_animal(&self) -> Box<dyn Animal>;
}

/// Factory type to be implemented
pub struct AnimalFactory;

/// Implementation of the factory for a specific type, but without specify that type to the
/// consumer of the factory. We hide that implementation using [Box<dyn Animal>]
impl Factory for AnimalFactory {
    fn build_animal(&self) -> Box<dyn Animal> {
        Box::new(Human)
    }
}

#[cfg(test)]
mod tests {
    use crate::factory::{AnimalFactory, Factory};

    #[test]
    fn factory_pattern() {
        let animal_factory:Box<dyn Factory> = Box::new(AnimalFactory);
        let animal = animal_factory.build_animal();
        println!("${:?}", animal.speak());
    }
}
