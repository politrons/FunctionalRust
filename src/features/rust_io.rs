use std::future::Future;
use std::process::Output;
use std::thread;
use std::time::Duration;

use futures::executor::block_on;
use futures::future;
use futures::stream::iter;
use futures::future::{join_all};

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
/// [of][from_func][from_option_func][from_result_func][from_option][from_result][merge]
/// Operators to transform monads
/// [map][fold]
/// Operators to compose monads
/// [flat_map][zip][parallel]
/// Operators to filter monads
/// [filter]
/// Operators to recover from side-effects
/// [recover][recover_with]
/// To slow the monad execution
/// [delay]
/// To unwrap the value from monad.
/// [get][get_or_else]
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

    fn get_or_else(self, default: A) -> A;

    fn is_ok(&self) -> bool;

    fn is_empty(&self) -> bool;

    fn map<F: FnOnce(A) -> A>(self, op: F) -> Self;

    fn flat_map<F: FnOnce(A) -> Self>(self, op: F) -> Self;

    fn zip<Z1: FnOnce() -> Self, Z2: FnOnce() -> Self, F: FnOnce(A, A) -> Self>(a: Z1, b: Z2, op: F) -> Self;

    fn parallel<Task: FnOnce() -> Self, F: FnOnce(Vec<A>) -> Self>(tasks: Vec<Task>, op: F) -> Self;

    fn filter<F: FnOnce(&A) -> bool>(self, op: F) -> Self;

    fn fold<F: FnOnce(A) -> A>(self, default: A, op: F) -> Self;

    fn recover<F: FnOnce() -> A>(self, op: F) -> Self;

    fn recover_with<F: FnOnce() -> Self>(self, op: F) -> Self;

    fn delay(self, time: Duration) -> Self;
}

///Data structure to be used as the monad to be implemented as [Lift]
#[derive(Debug, Copy, Clone)]
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

    fn merge<F: FnOnce(A, A) -> Self>(a: Self, b: Self, op: F) -> Self {
        let x1 = a.get();
        let x = x1;
        let y = b.get();
        op(x, y)
    }

    fn get(self) -> A {
        match self {
            Value(v) => v,
            Right(t) => t,
            _ => panic!("Error, value not available"),
        }
    }

    fn get_or_else(self, default: A) -> A {
        match self {
            Empty() => default,
            Wrong(_) => default,
            _ => self.get()
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
            Value(a) | Right(a) => op(a),
            Empty() => Empty(),
            Wrong(e) => Wrong(e)
        }
    }

    fn zip<Z1: FnOnce() -> Self, Z2: FnOnce() -> Self, F: FnOnce(A, A) -> Self>(a: Z1, b: Z2, op: F) -> Self {
        let empty = RustIO::Empty();
        let (zip_1, zip_2) = block_on(empty.run_future_zip_tasks(a, b));
        if (zip_1.is_ok() || !zip_1.is_empty()) && (zip_2.is_ok() || !zip_2.is_empty()) {
            return op(zip_1.get(), zip_2.get());
        }
        return empty;
    }

    fn parallel<Task: FnOnce() -> Self, F: FnOnce(Vec<A>) -> Self>(tasks: Vec<Task>, op: F) -> Self {
        let empty = Empty();
        let mut rios: Vec<_> = vec!();
        let tasks_done = block_on(empty.run_future_tasks(tasks));
        let find_error_tasks = &tasks_done;
        return match find_error_tasks.into_iter().find(|rio| rio.is_empty() || !rio.is_ok()) {
            Some(_) => {
                println!("Some of the task failed. Returning Empty value");
                empty
            }
            None => {
                for rio in tasks_done {
                    rios.push(rio.get());
                }
                op(rios)
            }
        };
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

    fn recover_with<F: FnOnce() -> Self>(self, op: F) -> Self {
        match self {
            Wrong(_) | Empty() => op(),
            _ => self
        }
    }

    fn delay(self, time: Duration) -> Self {
        match self {
            Value(_) | Right(_) => {
                thread::sleep(time);
                self
            }
            _ => self
        }
    }
}

impl<A, T> RustIO<A, T> {
    async fn run_future_zip_tasks<Z1: FnOnce() -> Self, Z2: FnOnce() -> Self>(&self, a: Z1, b: Z2) -> (RustIO<A, T>, RustIO<A, T>) {
        let future_zip1 = async {
            a()
        };
        let future_zip2 = async {
            b()
        };
        return futures::join!(future_zip1,future_zip2);
    }

    async fn run_future_tasks<Task: FnOnce() -> Self>(&self, tasks: Vec<Task>) -> Vec<RustIO<A, T>> {
        let future_tasks = tasks.into_iter()
            .fold(vec!(), |futures, task: Task| {
                let future_task = vec![async { return task(); }];
                return futures.into_iter().chain(future_task).collect::<Vec<_>>();
            });
        return join_all(future_tasks).await;
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
    fn rio_option_recover_with() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_option(None)
                        .recover_with(|| RustIO::from_option(Some("hello world!!".to_string())));
             RustIO::of(v)
        };
        println!("${:?}", rio_program);
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello world!!");
    }

    #[test]
    fn rio_error_recover_with() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_result(Err("".to_string()))
                        .recover_with(|| RustIO::from_result(Ok("hello world!!".to_string())));
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

    #[test]
    fn rio_delay() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_option(Some("hello world!!".to_string()))
                        .delay(Duration::from_secs(2));
             RustIO::of(v)
        };
        println!("${:?}", rio_program);
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello world!!");
    }

    #[test]
    fn rio_get_or_else() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_result(Err("".to_string()));
             RustIO::of(v)
        };
        println!("${:?}", rio_program);
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get_or_else("hello world!!".to_string()), "hello world!!");
    }

    #[test]
    fn rio_merge() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::merge(
                RustIO::from_option(Some("hello".to_string())), RustIO::from_option(Some(" world!!".to_string())),
                |a,b| RustIO::from_option(Some(a + &b)));
             RustIO::of(v)
        };
        println!("${:?}", rio_program);
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello world!!");
    }

    #[test]
    fn rio_zip() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::zip(
                || RustIO::from_option(Some("hello".to_string())), || RustIO::from_option(Some(" world!!".to_string())),
                |a,b| RustIO::from_option(Some(a + &b)));
             RustIO::of(v)
        };
        println!("${:?}", rio_program);
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello world!!");
    }

    #[test]
    fn rio_parallel() {
        let mut parallel_tasks: Vec<fn() -> RustIO<String, String>> = vec!();
        parallel_tasks.push(|| RustIO::from_option(Some("hello".to_string())));
        parallel_tasks.push(|| RustIO::from_option(Some(" world!!".to_string())));
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::parallel(parallel_tasks,|tasks| RustIO::of(tasks.into_iter().collect()));
             RustIO::of(v)
        };
        println!("${:?}", rio_program);
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello world!!");
    }
}