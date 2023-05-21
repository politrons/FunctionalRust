use crate::features::create_macros::RustIO::{Empty, Right, Value, Wrong};
#[macro_export]
macro_rules! io {
  // return
  (return $r:expr ;) => {
    $crate::Lift::lift($r)
  };

  // let-binding
  (let $p:pat = $e:expr ; $($r:tt)*) => {{
    let $p = $e;
    io!($($r)*)
  }};

  // const-bind
  (_ <- $x:expr ; $($r:tt)*) => {
    $x.and_then(move |_| { io!($($r)*) })
  };

  // bind
  ($binding:ident <- $x:expr ; $($r:tt)*) => {
    $x.and_then(move |$binding| { io!($($r)*) })
  };

  // const-bind
  ($e:expr ; $($a:tt)*) => {
    $e.and_then(move |_| io!($($a)*))
  };

  // pure
  ($a:expr) => {
    $a
  }
}

/// Lift a value inside a monad.
pub trait Lift<A,T> {
    /// Lift a value into a default structure.
    fn lift(a: A) -> Self;

    fn of(a: A) -> Self;

    fn from_option(a: Option<A>) -> Self;

    fn from_result(a: Result<A, T>) -> Self;

    fn get(self) -> A;

    fn and_then<F: FnOnce(A) -> Self>(self, op: F) -> Self;
}

#[derive(Debug)]
enum RustIO<A, T> {
    Right(A),
    Wrong(T),
    Value(A),
    Empty(),
}

impl<A, T> Lift<A,T> for RustIO<A, T> {
    fn lift(a: A) -> Self {
        RustIO::of(a)
    }

    fn of(a: A) -> Self {
        Right(a)
    }

    fn from_option(a: Option<A>) -> Self {
        todo!()
    }

    fn from_result(a: Result<A, T>) -> Self {
        todo!()
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rio() {
        let rio_program: RustIO<String, String> = io! {
             v <- RustIO::of(String::from("hello"));
             x <- RustIO::of(String::from(" world"));
             RustIO::of(v + &x)
        };
        println!("${:?}", rio_program);
        assert_eq!(rio_program.get(), "hello world");
    }
}