use std::fmt::Error;

/**
Having in Rust [Result] monad, this implementation is more a patter to show how we can abstract behavior.
 */
pub fn run() {
    let either: EitherMonad<String, String> = EitherMonad::right(String::from("hello either monad"));
    println!("Either is right:{}", either.is_right());
    println!("Either right:{}", either.get_right().to_string());

    let monad_program: EitherMonad<String, String> = EitherMonad::right("hello either monad".to_string())
        .map(|v| v.to_uppercase())
        .flat_map(|v| EitherMonad { right: Some(v + &"!!!"), left: None });
    println!("Program:{}", monad_program.get_right().to_string());

    let error_monad_program: EitherMonad<String, String> = EitherMonad::left("hello error either monad".to_string())
        .map_left(|v| v.to_uppercase());
    println!("Program:{}", error_monad_program.get_left().to_string());

}

/**Trait interface like in Scala, where we define functions to implement*/
trait Monad<L, R> {
    fn right(v: R) -> EitherMonad<L, R>;
    fn left(v: L) -> EitherMonad<L, R>;
    fn get_right(self) -> R;
    fn get_left(self) -> L;
    fn is_right(&self) -> bool;
    fn is_left(&self) -> bool;
    fn map(self, func: fn(R) -> R) -> EitherMonad<L, R>;
    fn map_left(self, func: fn(L) -> L) -> EitherMonad<L, R>;
    fn flat_map(self, func: fn(R) -> EitherMonad<L, R>) -> EitherMonad<L, R>;
}

/**Type to be used as implementation type for [Monad] trait.
Here we define Option type for the possible two values of this monad:
[right]: Means the execution of the monad is using the right value passed
[left]: Means the execution of the monad is using the left value passed
 */
struct EitherMonad<L, R> {
    left: Option<L>,
    right: Option<R>,
}

/**
Same syntax like in goLang where we define [impl] of the trait type, and then we use
[for] operator to specify over which type class we implement the trait,

Rust provide a powerful pattern matching to check the state of your types, pretty similar like in Scala
we can match not only the types, but also the types that we have inside our types.
 */
impl<L, R> Monad<L, R> for EitherMonad<L, R> {
    //Constructor function to create the monad with right value
    fn right(v: R) -> EitherMonad<L, R> {
        EitherMonad { right: Some(v), left: None }
    }
    //Constructor function to create the monad with left value
    fn left(v: L) -> EitherMonad<L, R> {
        EitherMonad { right: None, left: Some(v) }
    }
    //Supplier function to extract the right value from the monad
    fn get_right(self) -> R {
        self.right.unwrap()
    }
    //Supplier function to extract the left value from the monad
    fn get_left(self) -> L {
        self.left.unwrap()
    }
    //Predicate function to specify if the [either] monad is right
    fn is_right(&self) -> bool {
        self.right.is_some()
    }
    //Predicate function to specify if the [either] monad is left
    fn is_left(&self) -> bool {
        self.left.is_some()
    }
    //Transformation operator to get the right value from the monad and transform using the function
    fn map(self, func: fn(R) -> R) -> EitherMonad<L, R> {
        match self {
            | EitherMonad { right, left: None } => EitherMonad { right: right.map(func), left: None },
            | either @ _ => either, // return as is
        }
    }
    //Transformation operator to get the left value from the monad and transform using the function
    fn map_left(self, func: fn(L) -> L) -> EitherMonad<L, R> {
        match self {
            | EitherMonad { right:None, left } => EitherMonad { right: None, left: left.map(func) },
            | either @ _ => either, // return as is
        }
    }
    //Composition operator to compose two monad either
    fn flat_map(self, func: fn(R) -> EitherMonad<L, R>) -> EitherMonad<L, R> {
        match self {
            | EitherMonad { right, left: None } => func(right.unwrap()),
            | either @ _ => either, // return as is
        }
    }
}