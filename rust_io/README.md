# Rust_IO

Macro implementation for [rust_io] defining several operators to be used emulating Haskel [do notation]

## Use

```rust
#[cfg(test)]
mod tests {
    
    use rust_io::{rust_io, RustIO};
    use rust_io::{Lift};

    #[test]
    fn rio() {
        let rio_program: RustIO<String, String> = rust_io! {
             _ <- RustIO::of(String::from("1981"));
             v <- RustIO::from_option(Some(String::from("hello")));
             t <- RustIO::from_option_func(|| Some(String::from(" pure")));
             z <- RustIO::from_func(|| String::from(" functional"));
             x <- RustIO::from_result(Ok(String::from(" world")));
             i <- RustIO::of(String::from("!!"));
             y <- RustIO::from_result_func(|| Ok(String::from("!!")));

             yield v + &t + &z + &x + &i + &y;
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello pure functional world!!!!");
    }
}
```


## Operators

```rust
///Specification to be implemented by a monad.
/// [lift] a value into a default structure.
/// Operators to create monad:
/// [of][from_func][from_option_func][from_result_func][from_option][from_result][merge]
/// Operators to transform monads
/// [map][fold][map_error]
/// Operators to compose monads
/// [flat_map][zip]
/// Operators to filter monads
/// [filter]
/// Operators to filter and transform monad in one transaction
/// [when][when_rio]
/// Operators to recover from side-effects
/// [recover][recover_with][eventually]
/// To slow the monad execution
/// [delay]
/// To unwrap the value from monad.
/// [get][get_or_else]
/// Check the state of the monad
/// [is_ok][is_failed][is_empty]
/// Async task executions
/// [parallel][fork]
pub trait Lift<A, T> {
    fn lift(a: A) -> Self;

    fn of(a: A) -> Self;

    fn from_func(f: fn() -> A) -> Self;

    fn from_option_func(f: fn() -> Option<A>) -> Self;

    fn from_result_func(f: fn() -> Result<A, T>) -> Self;

    fn from_option(a: Option<A>) -> Self;

    fn from_result(a: Result<A, T>) -> Self;

    fn merge<F: FnOnce(A, A) -> Self>(a: Self, b: Self, op: F) -> Self;

    fn get(self) -> A;

    fn failed(self) -> T;

    fn get_or_else(self, default: A) -> A;

    fn is_ok(&self) -> bool;

    fn is_failed(&self) -> bool;

    fn is_empty(&self) -> bool;

    fn map<F: FnOnce(A) -> A>(self, op: F) -> Self;

    fn map_error<F: FnOnce(T) -> T>(self, op: F) -> Self;

    fn flat_map<F: FnOnce(A) -> Self>(self, op: F) -> Self;

    fn at_some_point<F: FnOnce(A) -> Self>(self, op: F) -> Self where A: Clone, F: Clone;

    fn at_some_point_while<P: FnOnce() -> bool, F: FnOnce(A) -> Self>(self, predicate: P, op: F) -> Self where A: Clone, P: Clone, F: Clone;

    fn at_some_point_until<P: FnOnce() -> bool, F: FnOnce(A) -> Self>(self, predicate: P, op: F) -> Self where A: Clone, P: Clone, F: Clone;

    fn when<P: FnOnce(&A) -> bool, F: FnOnce(A) -> A>(self, predicate: P, op: F) -> Self;

    fn when_rio<P: FnOnce(&A) -> bool, F: FnOnce(A) -> Self>(self, predicate: P, op: F) -> Self;

    fn zip<Z1: FnOnce() -> Self, Z2: FnOnce() -> Self, F: FnOnce(A, A) -> Self>(a: Z1, b: Z2, op: F) -> Self;

    fn filter<F: FnOnce(&A) -> bool>(self, op: F) -> Self;

    fn fold<F: FnOnce(A) -> A>(self, default: A, op: F) -> Self;

    fn recover<F: FnOnce() -> A>(self, op: F) -> Self;

    fn recover_with<F: FnOnce() -> Self>(self, op: F) -> Self;

    fn delay(self, time: Duration) -> Self;

    fn parallel<Task: FnOnce() -> Self, F: FnOnce(Vec<A>) -> Self>(tasks: Vec<Task>, op: F) -> Self;

    /// Provide [A:'static] in the definition it can extend the lifetime of a specific type
    fn fork<F: FnOnce(A) -> A>(self, op: F) -> Self where A: 'static, F: 'static;

    /// Provide [A:'static] in the definition it can extend the lifetime of a specific type
    fn join(self) -> Self;

    fn daemon<F: FnOnce(&A) -> ()>(self, op: F) -> Self;

    fn peek<F: FnOnce(&A) -> ()>(self, op: F) -> Self;

    fn on_error<F: FnOnce(&T) -> ()>(self, op: F) -> Self;

    fn on_success<F: FnOnce(&A) -> ()>(self, op: F) -> Self;
}

```