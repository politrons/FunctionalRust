pub fn run() {
    let string_value = "Hello world".to_string();
    println!("Contains hello:{}",string_value.contains_hello());
}

trait StringExt {
    fn contains_hello(&self) -> bool;
}

impl StringExt for String {
    fn contains_hello(&self) -> bool {
        self.contains("Hello")
    }
}
