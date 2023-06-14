
pub trait Animal {
    fn next_animal(&mut self, next: Box<dyn Animal>);
    fn eat(&self, request: &str);
}

pub struct Human {
    next: Option<Box<dyn Animal>>,
}

impl Animal for Human {
    fn next_animal(&mut self, next: Box<dyn Animal>) {
        self.next = Some(next);
    }

    fn eat(&self, food: &str) {
        if food == "Fish & chips" {
            println!("This {} is a human food mmmmm", food);
        } else if let Some(ref next) = self.next {
            next.eat(food);
        }
    }
}

pub struct Dog {
    next: Option<Box<dyn Animal>>,
}

impl Animal for Dog {
    fn next_animal(&mut self, next: Box<dyn Animal>) {
        self.next = Some(next);
    }

    fn eat(&self, food: &str) {
        if food == "bone" {
            println!("This {} is a dog food mmmmm", food);
        }  else {
            println!("Nobody wants this {} food", food);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::behavioral::chain_of_responsibility::{Animal, Dog, Human};

    #[test]
    fn cor_pattern() {
        let dog = Box::new(Dog { next: None });
        let human = Box::new(Human { next: Some(dog) });
        human.eat("bone");
        human.eat("Fish & chips");
        human.eat("poison cake")

    }
}
