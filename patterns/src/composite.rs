/// Public contract for all data types to be used as component
pub trait Animal {
    fn speak(&self);
}

pub struct Human;

impl Animal for Human {
    fn speak(&self) {
        println!("Hello there");
    }
}

pub struct Dog;

impl Animal for Dog {
    fn speak(&self) {
        println!("Burf");
    }
}

/// [Composite] pattern allow the use of [one] or [multiple] groups of [components]
/// Here we specify the collection of Component elements to bind together.
pub struct Composite {
    children: Vec<Box<dyn Animal>>,
}

/// In the implementation of Composite, we can initialize the collection,
/// and then add elements into that collection.
/// Then using [add_animal] we can add components as much as we like in the collection.
impl Composite {
    pub fn new() -> Self {
        Composite {
            children: Vec::new(),
        }
    }

    pub fn add_animal(&mut self, child: Box<dyn Animal>) {
        self.children.push(child);
    }
}

/// We create a [Composite] implementation of the trait, tom be consider another [Component],
/// but in the implementation, we invoke the collection of all [Component] to [Compose] all of them
/// together
impl Animal for Composite {
    fn speak(&self) {
        println!("Composing all animals");
        for child in &self.children {
            child.speak();
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::composite::{Animal, Composite, Dog, Human};

    #[test]
    fn composite_pattern() {
        let human = Box::new(Human {});
        let dog = Box::new(Dog {});
        let mut composite = Composite::new();
        composite.add_animal(human);
        composite.add_animal(dog);
        composite.speak();
    }
}
