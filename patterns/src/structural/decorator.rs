/// Contract of the component to implement
pub trait Animal {
    fn skills(&self) -> String;
}

/// Data type to implement a normal component
pub struct Human;

/// Implementation of component
impl Animal for Human {
    fn skills(&self) -> String {
        String::from("Hello there I can speak")
    }
}

/// [Decorator] Data type than can do what a [Animal] can do and even more.
/// It contains an instance of any [Animal] to do what that animal can do,
/// and [extend] the functionality
pub struct SuperHuman {
    animal: Box<dyn Animal>,
}

/// [Decorator] [Animal] implementation to behave like another animal, but with more capacity than
/// a regular animal type.
/// Here in the [skills] function, we execute the animal instance [skills] and we extend with something else.
impl Animal for SuperHuman {
    fn skills(&self) -> String {
        format!("{}. And I can also fly", self.animal.skills())
    }
}

#[cfg(test)]
mod tests {
    use crate::structural::decorator::{Animal, Human, SuperHuman};

    #[test]
    fn decorator_pattern() {
        let human = Box::new(Human);
        println!("${}",human.skills());
        let decorator = Box::new(SuperHuman{animal:human});
        println!("${}",decorator.skills());

    }
}