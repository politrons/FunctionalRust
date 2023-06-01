use std::fmt::Error;

use rand::Rng;

/**
Very powerful pattern matching functionality like in Scala or Haskell, provide the functionality
to match types, unwrap values from container types like [Option], [Result].
Also it is possible do some predicate functions in each case to match the value under some conditions.
 */
pub fn run() {
    primitive_types();
    enum_types();
    option_type();
    result_type();
    predicate_conditions();
    struct_type();
    tuple_type();
}

/**
Pattern matching in rust it can match any enum type, and knowing the static limits of enum
offer in the match all options.
 */
fn enum_types() {
    let fruit: Fruit = Fruit::Apple;
    match fruit {
        Fruit::Apple => println!("You're an Apple"),
        Fruit::Bananas => println!("You're a Banana"),
    }
}

/**
We can also not only match types, but also the values of that type.
Pattern matching also provide the functionality to return the value that has been matched.
We can use or operator [|] when we watch the type to collect multiple possible values.
We can also use range [n..=n] to define a possible range of number in the case.
 */
fn primitive_types() {
    let mut rng = rand::thread_rng();
    let num = rng.gen_range(0..4);
    match num {
        1 | 2 => println!("The number is 1 or 2"),
        3 => println!("The number is 3"),
        _ => println!("The number is not in the list"),
    }

    match num {
        1..=3 => println!("The number is  between 1 and 3"),
        4 => println!("The number is 4"),
        _ => println!("The number is not in the list"),
    }

    let string = "hello world";
    match string {
        "bye world" => println!("bye world"),
        "hello world" => println!("hello world"),
        "hello dude" => println!("hello dude"),
        _ => {}
    }

    let value = match *Box::new(1) {
        1 => "uno",
        2 => "dos",
        _ => "Empty"
    };
    println!("{}", value);

    match 5 {
        1..=5 => println!("The number is  between 1 and 5"),
        6 => println!("The number is 3"),
        _ => println!("The number is not in the list"),
    }
}

/**
Pattern matching it can match the [Option] Monad type, ans also unwrap the value of that monad.
[Option] can be [Some] or [None] and for the some type, we also extract the value
 */
fn option_type() {
    let option: Option<String> = Some(String::from("hello pattern matching"));
    match option {
        Some(v) => println!("{}", v),
        None => println!("No value found"),
    }
}

/**
Pattern matching it can match the [Result] Monad type, ans also unwrap the value of that monad.
[Result] can be [Ok] or [Err] and for the some type, we also extract the value
 */
fn result_type() {
    let result: Result<String, Error> = Ok(String::from("Success pattern matching"));
    match result {
        Ok(v) => println!("All good:{}", v),
        Err(_) => {}
    }
}

/**
Pattern matching it can also provide predicate function in cases to apart from match the type,
also do some filter by the value we extract.
Here apart from extract the value from the [Option] type, we also apply a filter over the value
 */
fn predicate_conditions() {
    let head = rand::thread_rng().gen_range(0..2) == 0;
    let message = String::from(if head { "Hello pattern matching with conditions" } else { "pattern matching" });
    let option: Option<String> = Some(message);
    match option {
        Some(v) if v.contains("Hello") && v.len() > 5 => println!("{}", v),
        None => println!("No value found"),
        _ => println!("Some value Condition did not match")
    }
}

/**
We can also apply pattern matching over a Struct attributes to folder over the same type, the attributes
we want to match.
Here we can use attributes [male] and [age] to filter multiple [Animals]
*/
fn struct_type() {
    let animal = Animal { male: true, age: 5 };
    match animal {
        Animal { male: true, age: _age } => println!("Male Animal"),
        Animal { male: _male, age: 10 } => println!("Old Animal"),
        _ => println!("Animal does not match"),
    }
}

/**
We can also use pattern matching to extract values from [Tuple] type
*/
fn tuple_type(){
    let array = ("hello", "pattern", "matching","world");
    match array {
        (first , _, _, last) => println!("First:{} Last:{}",first, last ),
    }
}

struct Animal {
    male: bool,
    age: i32,
}

enum Fruit {
    Apple,
    Bananas,
}