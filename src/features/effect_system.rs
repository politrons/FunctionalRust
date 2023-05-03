use std::fmt::Error;
use std::ops::Not;


pub fn run() {
    result_effect();
    option_effect("hello option monad");
    option_effect("");
    extract_result_effect();
    extract_option_effect();
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

/**
Using operator [?] is sugar syntax to unwrap the happy path values from [Option] and [Result]
But compiler will force you to return same Result type in function, just in case of [Error] or [None]
 */
fn extract_result_effect() -> Result<String, Error>{
    let result = get_result_type()?;
    println!("{}", result);
    Ok(result)
}

fn get_result_type() -> Result<String, Error> {
    Ok(String::from("hello Result effect"))
}

/**
In the example of option using [?] we dont have a value so the lines 64,65 never are executed, and
we return the [Option] with [None] value
*/
fn extract_option_effect() -> Option<String>{
    let result = get_option_type()?;
    println!("{}", result);
    Some(result)
}

fn get_option_type() -> Option<String> {
    None
}