pub fn run() {
    let string_value = "Hello world".to_string();
    println!("Contains hello:{}",string_value.contains_hello());
    println!("Number:{}",1981.multiply_by(10));
}

/**
Make extension methods in Rust is quite simple and clean.
We just need to define a [trait] with the the definition we want to extend.
And then create an implementation [impl] using the specific type to extend after [for]
*/
trait StringExt {
    fn contains_hello(&self) -> bool;
}

/**Implementation extension of [String] type*/
impl StringExt for String {
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
