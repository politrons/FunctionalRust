pub fn run() {
    println!("Contains hello:{}","Hello world".contains_hello());
    println!("Number:{}",1981.multiply_by(10));
    println!("Animal info:{}",Animal{ species:"Dog".to_string(), age:5}.animal_description());
}

/**
Make extension methods in Rust is quite simple and clean.
We just need to define a [trait] with the the definition we want to extend.
And then create an implementation [impl] using the specific type to extend after [for]
*/
trait StringExt {
    fn contains_hello(&self) -> bool;
}

/**Implementation extension of [str] type*/
impl StringExt for str {
    fn contains_hello(&self) -> bool {
        self.contains("Hello")
    }
}

trait NumberExt {
    fn multiply_by(&self,number:i32) -> i32;
}

/**Implementation extension of [i32] type*/
impl NumberExt for i32 {

    fn multiply_by(&self, number: i32) -> i32 {
        self * number
    }
}

trait AnimalExt {
    fn animal_description(self)->String;
}

/**Implementation extension of [Animal] type*/
impl AnimalExt for Animal{
    fn animal_description(self) -> String {
       self.species + &"-" + &self.age.to_string()
    }
}

struct Animal {
    species:String,
    age:i32,
}
