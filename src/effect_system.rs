use std::fmt::Error;
use std::ops::Not;


pub fn run() {
    result_effect();
    null_effect("hello option monad");
    null_effect("");
}

fn result_effect() {
    let result: Result<String, Error> =
        Result::Ok("hello world")
            .map(|v| v.to_uppercase());
    match result {
        Ok(v) => println!("{}", v.to_string()),
        _ => println!("Side effect found"),
    }
}

fn handling_errors() {
    let result: Result<String, Error> =
        Result::Ok("hello world")
            .map(|v| v.to_uppercase());
    match result {
        Ok(v) => println!("{}", v.to_string()),
        _ => println!("Side effect found"),
    }
}

/**
Rust does not allow produce NULL values by design. So in case we need to express absence of value,
 We can use Option type.
In rust String API we can concat some operators to transform the String into Option type.
 */
fn null_effect(value: &str) {
    let option: Option<String> = value.is_empty().not().then(|| value)
        .map(|v| v.to_uppercase());
    match option {
        Some(v) => println!("{}", v.to_string()),
        None => println!("No element found "),
    }
}