use crate::features::rust_io::RustIO::{Empty, Right, Value, Wrong};
/// Macro implementation for [rust_io] defining several operators to be used emulating
/// Haskel [do notation]
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
    $x.and_then(move |_| { rust_io!($($r)*) })
  };

  // bind
  ($bind:ident <- $x:expr ; $($r:tt)*) => {
    $x.and_then(move |$bind| { rust_io!($($r)*) })
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
/// Operators to compose monads
/// [and_then]
pub trait Lift<A, T> {
    fn lift(a: A) -> Self;

    fn of(a: A) -> Self;

    fn from_func(f: fn() -> A) -> Self;

    fn from_option_func(f: fn() -> Option<A>) -> Self;

    fn from_result_func(f: fn() -> Result<A, T>) -> Self;

    fn from_option(a: Option<A>) -> Self;

    fn from_result(a: Result<A, T>) -> Self;

    fn get(self) -> A;

    fn and_then<F: FnOnce(A) -> Self>(self, op: F) -> Self;

    fn is_ok(&self) -> bool;

    fn is_empty(&self) -> bool;
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

    fn and_then<F: FnOnce(A) -> RustIO<A, T>>(self, op: F) -> RustIO<A, T> {
        match self {
            Value(t) => op(t),
            Empty() => Empty(),
            Right(a) => op(a),
            Wrong(e) => Wrong(e)
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
            Value(_) => true,
            Right(_) => true,
            _ => false,
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
    fn rio_error() {
        let rio_program: RustIO<String, i32> = rust_io! {
             x <- RustIO::from_result(Err(503));
             v <- RustIO::from_option(Some(String::from("hello")));
             RustIO::of(v)
        };
        println!("${:?}", rio_program);
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(false, rio_program.is_ok());
    }
}