pub fn run() {
    let f = |x: i32, y: i32| x + y;
    let result = f(10, 20);
    println!("{}", result);

    let sentence = hello_world_fun()(String::from("Paul"));
    println!("{}", sentence);

    let sentence = apply_hello_world(hello_world_fun());
    println!("{}", sentence);

    let number = multiply_by_1000()(1981);
    println!("{}", number);

    let in_upper_case = map(String::from("hello world in upper case"), |s| s.to_uppercase());
    println!("{}", in_upper_case);

    let append_char = map(String::from("!!!"), |s| "hello world".to_string() + &s);
    println!("{}", append_char);

    let concat_result = concat_func("hello ".to_string(), |t| t + "world", |t| t.to_uppercase());
    println!("{}", concat_result.to_string());

    let is_higher_than_2000 = predicate_func(1981, |n| n > 2000);
    println!("{}", is_higher_than_2000.to_string());

    let contains_hello = predicate_func("hello world", |n| n.contains("hello"));
    println!("{}", contains_hello.to_string());

    consumer_func("hello consumer function", |s| println!("{}", s));

    let zip_result = zip_func("hello".to_string(), "WORLD".to_string(), |t1| t1.to_uppercase(), |t2| t2.to_lowercase());
    println!("{}", zip_result);

    let response = where_func(String::from("hello world"), |v| v.to_uppercase());
    println!("{}", response);

    let response = function_2("hello", "world", |s, s1| s.to_string() + s1);
    println!("{}", response);

    let response = function_3("hello", "world", "!!!", |s, s1, s2| s.to_string() + s1 + s2);
    println!("{}", response);
}

// Functions
//-----------

//Rust works with High order functions, so we can return a function in a function.
fn hello_world_fun() -> fn(String) -> String {
    |name| String::from("hello world ") + &name
}

//We can also receive a function inside a function.
fn apply_hello_world(hello_world_func: fn(String) -> String) -> String {
    hello_world_func(String::from("Paul"))
}

//Simple function to multiply any number by 1000
fn multiply_by_1000() -> fn(u64) -> u64 {
    |number| number * 1000
}

//Transform function that receive a generic value T and transform applying the function [m]
fn map<T>(t: T, m: fn(T) -> T) -> T {
    m(t)
}

//Function that concatenate two functions, passing the output of one function to the next one.
fn concat_func<T>(t: T, func1: fn(T) -> T, func2: fn(T) -> T) -> T {
    func2(func1(t))
}

//Predicate function that receive an argument and a predicate func and apply the func over the value.
fn predicate_func<T>(t: T, func: fn(T) -> bool) -> bool {
    func(t)
}

//A Consumer function that receive a param and just apply the function.
fn consumer_func<T>(t: T, func: fn(T)) {
    func(t)
}

//A function that receive two functions and zip the result of both functions.
fn zip_func(t1: String, t2: String, func_t1: fn(String) -> String, func_t2: fn(String) -> String) -> String {
    func_t1(t1).to_string() + &func_t2(t2).to_string()
}

//In Rust we can use [where] syntax to define a generic type after the definition adding specific constraints
fn where_func<F, T>(value: T, handler: F) -> T where F: Fn(T) -> T,
{
    handler(value)
}

//Function 2 it just a function that expect to receive two arguments in the function
fn function_2<F, T, X>(f: F, t: T, func_2: fn(s: F, s1: T) -> X) -> X {
    return func_2(f, t);
}

//Function 3 it just a function that expect to receive three arguments in the function
fn function_3<F, T, Z, X>(f: F, t: T, z: Z, func_3: fn(s: F, s1: T, s2: Z) -> X) -> X {
    return func_3(f, t, z);
}