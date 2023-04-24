use std::fmt::Error;

pub fn run() {
    let my_monad = TryMonad::of("hello try monad");
    println!("Try:{}", my_monad.get().to_string());

    let monad_program = TryMonad::of("hello try monad".to_string())
        .map(|v| v.to_uppercase())
        .flat_map(|v| TryMonad { success: Some(v + &"!!!"),failure:None});
    println!("Program:{}", monad_program.get().to_string());

}

/**Trait interface like in Scala, where we define functions to implement*/
trait Monad<T> {
    fn of(v: T) -> TryMonad<T>;
    fn get(self) -> T;
    fn map(self, func: fn(T) -> T) -> TryMonad<T>;
    fn flat_map(self, func:fn(T) -> TryMonad<T>) -> TryMonad<T>;
}

/**Type to be used as implementation type for [Monad] trait.
Here we define Option type for the possible two values of this monad:
[success]: Means the execution of the monad is success
[failure]: Means some side-effect were detected and marked as failure.
 */
struct TryMonad<T> {
    success:Option<T>,
    failure:Option<Error>
}

/**
Same syntax like in goLang where we define [impl] of the trait type, and then we use
[for] operator to specify over which type class we implement the trait,

Rust provide a powerful pattern matching to check the state of your types, pretty similar like in Scala
we can match not only the types, but also the types that we have inside our types.
 */
impl<T> Monad<T> for TryMonad<T> {
    //Constructor function to create the monad
    fn of(v: T) -> TryMonad<T> {
        TryMonad { success: Some(v), failure:None }
    }
    //Supplier function to extract the value from the monad
    fn get(self) -> T {
        match self {
            | TryMonad { success, failure: None } =>
                success.unwrap(),
            | failed @ _ =>  panic!("{}",failed.failure.unwrap()),

        }
    }
    //Transformation operator to get the value from the monad and transform using the function
    fn map(self, func: fn(T) -> T) -> TryMonad<T> {
        match self {
            | TryMonad { success, failure: None } =>
                TryMonad { success: success.map(func), failure: None },
            | failed @ _ => failed, // return as is
        }
    }

    //Composition operator to compose two monad try
    fn flat_map(self, func: fn(T) -> TryMonad<T>) -> TryMonad<T> {
        match self {
            | TryMonad { success, failure: None } =>
                func(success.unwrap()),
            | failed @ _ => failed, // return as is
        }
    }
}