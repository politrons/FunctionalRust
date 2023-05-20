use crate::features::create_macros::MIO::Value;
use crate::features::create_macros::RIO::{Right, Wrong};
#[macro_export]
macro_rules! io {
  // return
  (return $r:expr ;) => {
    $crate::MLift::lift($r)
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
pub trait MLift<A> {
    /// Lift a value into a default structure.
    fn lift(a: A) -> Self;

    fn of(a: A) -> Self;

    fn get(self) -> A;

    fn and_then<F: FnOnce(A) -> Self>(self, op: F) -> Self;
}

pub trait RLift<A, T> {
    /// Lift a value into a default structure.
    fn lift(a: A) -> Self;

    fn of(a: A) -> Self;

    fn get(self) -> A;

    fn and_then<F: FnOnce(A) -> Self>(self, op: F) -> Self;

    fn error(self) -> T;
}

#[derive(Debug)]
enum MIO<T> {
    Value(T),
    Empty,
}

#[derive(Debug)]
enum RIO<A, T> {
    Right(A),
    Wrong(T),
}

impl<A> MLift<A> for MIO<A> {
    fn lift(a: A) -> Self {
        MIO::of(a)
    }

    fn of(a: A) -> Self {
        Value(a)
    }

    fn get(self) -> A {
        match self {
            Value(t) => t,
            _ => panic!("No value available"),
        }
    }

    fn and_then<F: FnOnce(A) -> MIO<A>>(self, op: F) -> MIO<A> {
        match self {
            Value(t) => op(t),
            _ => MIO::Empty,
        }
    }
}

impl<A, T> RLift<A, T> for RIO<A, T> {
    fn lift(a: A) -> Self {
        RIO::of(a)
    }

    fn of(a: A) -> Self {
        Right(a)
    }

    fn get(self) -> A {
        match self {
            Right(t) => t,
            _ => panic!("Error, value not available"),
        }
    }

    fn and_then<F: FnOnce(A) -> RIO<A, T>>(self, op: F) -> RIO<A, T> {
        match self {
            Right(a) => op(a),
            Wrong(e) => Wrong(e)
        }
    }

    fn error(self) -> T {
        match self {
            Wrong(t) => t,
            _ => panic!("Error, value not available"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mio() {
        let mio_program: MIO<i32> = io! {
             v <- MIO::of(50);
             x <- MIO::of(100);
             MIO::of(v + x)
        };
        println!("${:?}", mio_program);
        assert_eq!(mio_program.get(), 150);
    }

    #[test]
    fn rio() {
        let rio_program: RIO<String, String> = io! {
             v <- RIO::of(String::from("hello"));
             x <- RIO::of(String::from(" world"));
             RIO::of(v + &x)
        };
        println!("${:?}", rio_program);
        assert_eq!(rio_program.get(), "hello world");
    }

    // #[test]
    // fn mio_rio() {
    //     let rio_program: RIO<String, String> = io! {
    //          v <- RIO::of(String::from("hello"));
    //          x <- MIO::of(String::from(" world"));
    //          RIO::of(v + &x)
    //     };
    //     println!("${:?}", rio_program);
    //     assert_eq!(rio_program.get(), "hello world");
    // }
}