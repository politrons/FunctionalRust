use crate::features::create_macros::MIO::Value;
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
pub trait Lift<A> {
    /// Lift a value into a default structure.
    fn lift(a: A) -> Self;

    fn of(a:A) -> Self;

    fn get(self) -> A;

    fn and_then<F: FnOnce(A) -> Self>(self,op: F) -> Self;


}

#[derive( Debug)]
enum MIO<T> {
    Value(T),
    Empty
}

impl<A> Lift<A> for MIO<A> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option() {
        let mio_program: MIO<i32> = io! {
             v <- MIO::of(50);
             x <- MIO::of(100);
             MIO::of(v + x)
        };
        println!("${:?}",mio_program);
        assert_eq!(mio_program.get(), 150);

    }
}