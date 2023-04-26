use std::fmt::Error;
use std::ops::Not;


pub fn run() {
    result_effect();
    option_effect("hello option monad");
    option_effect("");
}

/**
In rust there is no such things like [runtime] errors. In case an Error happens is consider a [Panic] error type
and is not recoverable, so the Thread will die.
For any other possible side-effect that  might happen when we interact with real world, rust provide by design
a Monad Error Transformer called [Result] which we can use to map error and compose them, or just pass the successful value.
Just like a monad it's allow you to use all common functional operations.
*/
fn result_effect() {
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
fn option_effect(value: &str) {
    let option: Option<String> = value.is_empty().not().then(|| value)
        .map(|v| v.to_uppercase());
    match option {
        Some(v) => println!("{}", v.to_string()),
        None => println!("No element found "),
    }
}