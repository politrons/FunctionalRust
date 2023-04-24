pub fn run() {
    let string_value = "Hello world".to_string();
    println!("Contains hello:{}",string_value.contains_hello());
    println!("Number:{}",1981.multiply_by(10));
}

trait StringExt {
    fn contains_hello(&self) -> bool;
}

impl StringExt for String {
    fn contains_hello(&self) -> bool {
        self.contains("Hello")
    }
}

trait NumberExt {
    fn multiply_by(&self,number:i32) -> i32;
}

impl NumberExt for i32 {

    fn multiply_by(&self, number: i32) -> i32 {
        self * number
    }
}
