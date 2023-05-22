use crate::features::rust_io::RustIO::{Empty, Right, Value, Wrong};
/// Macro implementation for [rust_io] defining several operators to be used emulating
/// Haskel [do notation]
/// Work based on original idea of crate [do-notation]
#[macro_export]
macro_rules! rust_io {
  // return
  (return $r:expr ;) => {
    $crate::Lift::lift($r)
  };

  // let variable bind
  (let $p:pat = $e:expr ; $($r:tt)*) => {{
    let $p = $e;
    rust_io!($($r)*)
  }};

  // unused variable bind
  (_ <- $x:expr ; $($r:tt)*) => {
    $x.flat_map(move |_| { rust_io!($($r)*) })
  };

  // bind
  ($bind:ident <- $x:expr ; $($r:tt)*) => {
    $x.flat_map(move |$bind| { rust_io!($($r)*) })
  };

  // return type from do-notation
  ($a:expr) => {
    $a
  }
}

///Specification to be implemented by a monad.
/// [lift] a value into a default structure.
/// Operators to create monad:
/// [of][from_func][from_option_func][from_result_func][from_option][from_result]
/// Operators to transform monads
/// [map][fold]
/// Operators to compose monads
/// [flat_map]
/// Operators to filter monads
/// [filter]
pub trait Lift<A, T> {
    fn lift(a: A) -> Self;

    fn of(a: A) -> Self;

    fn from_func(f: fn() -> A) -> Self;

    fn from_option_func(f: fn() -> Option<A>) -> Self;

    fn from_result_func(f: fn() -> Result<A, T>) -> Self;

    fn from_option(a: Option<A>) -> Self;

    fn from_result(a: Result<A, T>) -> Self;

    fn get(self) -> A;

    fn is_ok(&self) -> bool;

    fn is_empty(&self) -> bool;

    fn map<F: FnOnce(A) -> A>(self, op: F) -> Self;

    fn flat_map<F: FnOnce(A) -> Self>(self, op: F) -> Self;

    fn filter<F: FnOnce(&A) -> bool>(self, op: F) -> Self;

    fn fold<F: FnOnce(A) -> A>(self, default: A, op: F) -> Self;

    fn recover<F: FnOnce() -> A>(self, op: F) -> Self;
}

///Data structure to be used as the monad to be implemented as [Lift]
#[derive(Debug)]
enum RustIO<A, T> {
    Right(A),
    Wrong(T),
    Value(A),
    Empty(),
}

/// Implementation of the Monad Lift.
impl<A, T> Lift<A, T> for RustIO<A, T> {
    fn lift(a: A) -> Self {
        RustIO::of(a)
    }

    fn of(a: A) -> Self {
        Value(a)
    }

    fn from_func(f: fn() -> A) -> Self {
        Value(f())
    }

    fn from_option_func(f: fn() -> Option<A>) -> Self {
        RustIO::from_option(f())
    }

    fn from_result_func(f: fn() -> Result<A, T>) -> Self {
        RustIO::from_result(f())
    }

    fn from_option(a: Option<A>) -> Self {
        match a {
            None => Empty(),
            Some(v) => Value(v)
        }
    }

    fn from_result(a: Result<A, T>) -> Self {
        match a {
            Ok(v) => Right(v),
            Err(t) => Wrong(t)
        }
    }

    fn get(self) -> A {
        match self {
            Value(v) => v,
            Right(t) => t,
            _ => panic!("Error, value not available"),
        }
    }

    fn is_ok(&self) -> bool {
        match self {
            Value(_) => true,
            Right(_) => true,
            _ => false,
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Value(_) => false,
            Right(_) => false,
            _ => true,
        }
    }

    fn map<F: FnOnce(A) -> A>(self, op: F) -> Self {
        match self {
            Value(v) => Value(op(v)),
            Right(v) => Right(op(v)),
            _ => self,
        }
    }

    fn flat_map<F: FnOnce(A) -> Self>(self, op: F) -> Self {
        match self {
            Value(t) => op(t),
            Empty() => Empty(),
            Right(a) => op(a),
            Wrong(e) => Wrong(e)
        }
    }

    fn filter<F: FnOnce(&A) -> bool>(self, op: F) -> Self {
        return match self {
            Value(t) => {
                let x = t;
                return if op(&x) { Value(x) } else { Empty() };
            }
            Empty() => Empty(),
            Right(a) => {
                let x = a;
                return if op(&x) { Right(x) } else { Empty() };
            }
            Wrong(e) => Wrong(e),
        };
    }

    fn fold<F: FnOnce(A) -> A>(self, default: A, op: F) -> Self {
        match self {
            Value(v) => Value(op(v)),
            Right(v) => Right(op(v)),
            Empty() => Value(default),
            _ => self
        }
    }

    fn recover<F: FnOnce() -> A>(self, op: F) -> Self {
        match self {
            Wrong(_) => Right(op()),
            Empty() => Value(op()),
            _ => self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

             RustIO::of(v + &t + &z + &x + &i + &y)
        };
        println!("${:?}", rio_program);
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello pure functional world!!!!");
    }

    #[test]
    fn rio_transformation() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_option(Some(String::from("hello")))
                        .map(|v| v.to_uppercase());
             x <- RustIO::from_result(Ok(String::from(" world")))
                        .map(|v| v.to_uppercase());
             i <- RustIO::of(String::from("!!"));
             RustIO::of(v + &x + &i)
        };
        println!("${:?}", rio_program);
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "HELLO WORLD!!");
    }

    #[test]
    fn rio_composition() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_option(Some(String::from("hello")))
                        .flat_map(|v| RustIO::of( v + &String::from(" world")))
                        .map(|v| v.to_uppercase());
             i <- RustIO::of(String::from("!!"));
             RustIO::of(v + &i)
        };
        println!("${:?}", rio_program);
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "HELLO WORLD!!");
    }

    #[test]
    fn rio_filter() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_option(Some(String::from("hello")))
                        .flat_map(|v| RustIO::of( v + &String::from(" world")))
                        .filter(|v| v.len() > 5);
             i <- RustIO::of(String::from("!!"));
             RustIO::of(v + &i)
        };
        println!("${:?}", rio_program);
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello world!!");
    }

    #[test]
    fn rio_compose_two_programs() {
        let rio_program_1: RustIO<String, String> = rust_io! {
             v <- RustIO::from_option(Some(String::from("hello")));
             RustIO::of(v + &" ".to_string())
        };
        let rio_program_2: RustIO<String, String> = rust_io! {
             v <- RustIO::from_option(Some(String::from("world")));
             RustIO::of(v + &"!!".to_string())
        };
        let rio_program: RustIO<String, String> = rust_io! {
             v <- rio_program_1;
             i <- rio_program_2;
             RustIO::of(v + &i).map(|v| v.to_uppercase())
        };
        println!("${:?}", rio_program);
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "HELLO WORLD!!");
    }

    #[test]
    fn rio_fold() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_option(None)
                        .fold("hello world!!".to_string(), |v| v.to_uppercase());
             RustIO::of(v)
        };
        println!("${:?}", rio_program);
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello world!!");
    }

    #[test]
    fn rio_empty_recover() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_option(None)
                        .recover(|| "hello world!!".to_string());
             RustIO::of(v)
        };
        println!("${:?}", rio_program);
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello world!!");
    }

    #[test]
    fn rio_error_recover() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_result(Err("".to_string()))
                        .recover(|| "hello world!!".to_string());
             RustIO::of(v)
        };
        println!("${:?}", rio_program);
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello world!!");
    }

    #[test]
    fn rio_error() {
        let rio_program: RustIO<String, i32> = rust_io! {
             i <- RustIO::from_option(Some(String::from("hello")));
             _ <- RustIO::from_result(Err(503));
             v <- RustIO::from_option(Some(String::from("world")));
             RustIO::of(i + &v)
        };
        println!("${:?}", rio_program);
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(false, rio_program.is_ok());
    }

    #[test]
    fn rio_filter_empty() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_option(Some(String::from("hello")))
                        .filter(|v| v.len() > 10);
             i <- RustIO::of(String::from("!!"));
             RustIO::of(v + &i)
        };
        println!("${:?}", rio_program);
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(true, rio_program.is_empty());
    }
}