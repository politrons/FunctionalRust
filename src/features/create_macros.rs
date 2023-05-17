use std::array::IntoIter;
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
}

impl<A> Lift<A> for Option<A> {
    fn lift(a: A) -> Self {
        Some(a)
    }
}

impl<A, E> Lift<A> for Result<A, E> {
    fn lift(a: A) -> Self {
        Ok(a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option() {

        let monad = io!(Some(3));

        let r: Option<i32> = io! {
             v <- Some(3);
             Some(v)
        };
        assert_eq!(r, Some(3));

        let r: Option<i32> = io! {
            v <- r;
            x <- Some(10);
             Some(v * x)
        };
        assert_eq!(r, Some(30));

    }
}