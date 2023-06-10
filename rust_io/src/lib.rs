use std::thread;
use std::time::Duration;

use futures::{FutureExt};
use futures::executor::block_on;
use futures::future::{join_all, LocalBoxFuture};
use rand::{Rng, thread_rng};

use crate::RustIO::{Empty, Fut, Right, Value, Wrong};

/// Macro implementation for [rust_io] defining several operators to be used emulating
/// Haskel [do notation]
/// Work based on original idea of crate [do-notation]
#[macro_export]
macro_rules! rust_io {
  // return
  (yield $r:expr ;) => {
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

///Data structure to be used as the monad to be implemented as [Lift]
/// RustIO monad can have the list of possible states.
pub enum RustIO<A, T> {
    Right(A),
    Wrong(T),
    Value(A),
    Empty(),
    Fut(LocalBoxFuture<'static, A>),
}

/// Implementation of the Monad Lift.
impl<A, T> Lift<A, T> for RustIO<A, T> {
    fn lift(a: A) -> Self {
        RustIO::of(a)
    }

    /// Pure value to create RustIO monad without side-effects.
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
        return a.flat_map(|x| b.flat_map(|y| op(x, y)));
    }

    fn get(self) -> A {
        match self {
            Value(v) => v,
            Right(t) => t,
            _ => panic!("Error, value not available"),
        }
    }

    fn failed(self) -> T {
        match self {
            Value(_) | Right(_) | Empty() | Fut(_) => panic!("Error, value not available"),
            Wrong(e) => e,
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

    fn is_failed(&self) -> bool {
        match self {
            Value(_) => false,
            Right(_) => false,
            _ => true,
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

    fn map_error<F: FnOnce(T) -> T>(self, op: F) -> Self {
        match self {
            Wrong(e) => Wrong(op(e)),
            _ => self
        }
    }

    fn flat_map<F: FnOnce(A) -> Self>(self, op: F) -> Self {
        match self {
            Value(a) | Right(a) => op(a),
            Empty() => Empty(),
            Wrong(e) => Wrong(e),
            _ => self
        }
    }

    ///Returns an effect that ignores errors and runs repeatedly until it [at_some_point] succeeds
    /// We mark A type as Clone since we need a clone of the value for each iteration in the loop.
    /// In case you need a backoff between iterations, or a escape clause, you can use
    /// [until] or [while] [at_some_point] operator conditions.
    fn at_some_point<F: FnOnce(A) -> Self>(self, op: F) -> Self where A: Clone, F: Clone {
        match self {
            Value(a) | Right(a) => {
                loop {
                    let op_copy = op.clone();
                    let a_copy = a.clone();
                    let result = op_copy(a_copy);
                    if result.is_ok() {
                        break result;
                    }
                }
            }
            _ => self
        }
    }

    /// Retry pattern of a task while a predicate condition is [false]
    fn at_some_point_while<P: FnOnce() -> bool, F: FnOnce(A) -> Self>(self, predicate: P, op: F) -> Self where A: Clone, P: Clone, F: Clone {
        self.at_some_point_cond(false, predicate, op)
    }

    /// Retry pattern of a task while a predicate condition is [true]
    fn at_some_point_until<P: FnOnce() -> bool, F: FnOnce(A) -> Self>(self, predicate: P, op: F) -> Self where A: Clone, P: Clone, F: Clone {
        self.at_some_point_cond(true, predicate, op)
    }

    fn when<P: FnOnce(&A) -> bool, F: FnOnce(A) -> A>(self, predicate: P, op: F) -> Self {
        return match self {
            Value(t) => {
                let x = t;
                return if predicate(&x) { Value(op(x)) } else { Empty() };
            }
            Empty() => Empty(),
            Right(a) => {
                let x = a;
                return if predicate(&x) { Right(op(x)) } else { Empty() };
            }
            Wrong(e) => Wrong(e),
            _ => self
        };
    }

    fn when_rio<P: FnOnce(&A) -> bool, F: FnOnce(A) -> Self>(self, predicate: P, op: F) -> Self {
        return match self {
            Value(t) => {
                let x = t;
                return if predicate(&x) { op(x) } else { Empty() };
            }
            Empty() => Empty(),
            Right(a) => {
                let x = a;
                return if predicate(&x) { op(x) } else { Empty() };
            }
            Wrong(e) => Wrong(e),
            _ => self
        };
    }

    fn zip<Z1: FnOnce() -> Self, Z2: FnOnce() -> Self, F: FnOnce(A, A) -> Self>(a: Z1, b: Z2, op: F) -> Self {
        let empty = Empty();
        let (zip_1, zip_2) = block_on(empty.run_future_zip_tasks(a, b));
        if (zip_1.is_ok() || !zip_1.is_empty()) && (zip_2.is_ok() || !zip_2.is_empty()) {
            return op(zip_1.get(), zip_2.get());
        }
        return empty;
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
            _ => self
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

    /// Operator to run every task in the Vector asynchronously using async.
    /// After we create the list of Futures, we use [join_all] to run all futures in parallel.
    /// Once all of them are finished, we invoke the passed function with [Vector<A>] as input param
    fn parallel<Task: FnOnce() -> Self, F: FnOnce(Vec<A>) -> Self>(tasks: Vec<Task>, op: F) -> Self {
        let empty = Empty();
        let tasks_done = block_on(empty.run_future_tasks(tasks));
        let find_error_tasks = &tasks_done;
        return match find_error_tasks.into_iter().find(|rio| rio.is_empty() || !rio.is_ok()) {
            Some(_) => {
                println!("Some of the task failed. Returning Empty value");
                empty
            }
            None => {
                let rios = tasks_done.into_iter()
                    .fold(vec!(), |rios, task_done| {
                        return rios.into_iter().chain(vec![task_done.get()]).collect::<Vec<_>>();
                    });
                op(rios)
            }
        };
    }

    /// It run the execution of the task in another green thread
    /// We use type [Fut] to wrap the [LocalBoxFuture<A>] which it contains the output of the function execution.
    fn fork<F: FnOnce(A) -> A>(self, op: F) -> Self where A: 'static, F: 'static {
        match self {
            Value(v) | Right(v) => {
                Fut(async { op(v) }.boxed_local())
            }
            _ => self,
        }
    }

    ///Join the [LocalBoxFuture<A>].
    fn join(self) -> Self {
        block_on(self.unbox_fork())
    }

    /// async consumer function that does not affect the current value of the monad.
    fn daemon<F: FnOnce(&A) -> ()>(self, op: F) -> Self {
        return block_on(self.run_daemon(op));
    }

    fn peek<F: FnOnce(&A) -> ()>(self, op: F) -> Self {
        return match self {
            Value(v) => {
                let x = v;
                op(&x);
                Value(x)
            }
            Right(v) => {
                let x = v;
                op(&x);
                Right(x)
            }
            _ => self
        };
    }

    fn on_error<F: FnOnce(&T) -> ()>(self, op: F) -> Self {
        return match self {
            Wrong(v) => {
                let x = v;
                op(&x);
                Wrong(x)
            }
            _ => self
        };
    }

    fn on_success<F: FnOnce(&A) -> ()>(self, op: F) -> Self {
        return match self {
            Right(v) => {
                let x = v;
                op(&x);
                Right(x)
            }
            _ => self
        };
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

    async fn unbox_fork(self) -> RustIO<A, T> {
        match self {
            Fut(fut_box) => {
                println!("Extracting future");
                Value(fut_box.await)
            }
            _ => Empty(),
        }
    }

    async fn run_daemon<F: FnOnce(&A) -> ()>(self, op: F) -> RustIO<A, T> {
        return match self {
            Value(v) => {
                let x = v;
                async { op(&x) }.await;
                Value(x)
            }
            Right(v) => {
                let x = v;
                async { op(&x) }.await;
                Right(x)
            }
            _ => self
        };
    }

    /// Generic function to cover [at_some_point] [while] and [until]
    fn at_some_point_cond<P: FnOnce() -> bool, F: FnOnce(A) -> Self>(self, cond: bool, predicate: P, op: F) -> Self where A: Clone, P: Clone, F: Clone {
        match self {
            Value(a) | Right(a) => {
                loop {
                    let op_copy = op.clone();
                    let predicate_copy = predicate.clone();
                    let a_copy = a.clone();
                    let result = op_copy(a_copy);
                    if result.is_ok() || predicate_copy() == cond {
                        break result;
                    }
                }
            }
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

             yield v + &t + &z + &x + &i + &y;
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello pure functional world!!!!");
    }

    #[test]
    fn rio_map() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_option(Some(String::from("hello")))
                        .map(|v| v.to_uppercase());
             x <- RustIO::from_result(Ok(String::from(" world")))
                        .map(|v| v.to_uppercase());
             i <- RustIO::of(String::from("!!"));
             yield v + &x + &i;
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "HELLO WORLD!!");
    }

    #[test]
    fn rio_flat_map() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_option(Some(String::from("hello")))
                        .flat_map(|v| RustIO::of( v + &String::from(" world")))
                        .map(|v| v.to_uppercase());
             i <- RustIO::of(String::from("!!"));
             yield v + &i;
        };
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
             yield v + &i;
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello world!!");
    }

    #[test]
    fn rio_compose_two_programs() {
        let rio_program_1: RustIO<String, String> = rust_io! {
             v <- RustIO::from_option(Some(String::from("hello")));
             yield v + &" ".to_string();
        };
        let rio_program_2: RustIO<String, String> = rust_io! {
             v <- RustIO::from_option(Some(String::from("world")));
             yield v + &"!!".to_string();
        };
        let rio_program: RustIO<String, String> = rust_io! {
             v <- rio_program_1;
             i <- rio_program_2;
             RustIO::of(v + &i).map(|v| v.to_uppercase())
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "HELLO WORLD!!");
    }

    #[test]
    fn rio_fold() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_option(None)
                        .fold("hello world!!".to_string(), |v| v.to_uppercase());
             yield v;
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello world!!");
    }

    #[test]
    fn rio_empty_recover() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_option(None)
                        .recover(|| "hello world!!".to_string());
             yield v;
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello world!!");
    }

    #[test]
    fn rio_error_recover() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_result(Err("".to_string()))
                        .recover(|| "hello world!!".to_string());
            yield v;
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello world!!");
    }

    #[test]
    fn rio_option_recover_with() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_option(None)
                        .recover_with(|| RustIO::from_option(Some("hello world!!".to_string())));
             yield v;
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello world!!");
    }

    #[test]
    fn rio_error_recover_with() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_result(Err("".to_string()))
                        .recover_with(|| RustIO::from_result(Ok("hello world!!".to_string())));
             yield v;
        };
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
             yield (i + &v);
        };
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
             yield (v + &i);
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(true, rio_program.is_empty());
    }

    #[test]
    fn rio_delay() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_option(Some("hello world!!".to_string()))
                        .delay(Duration::from_secs(2));
            yield v;
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello world!!");
    }

    #[test]
    fn rio_get_or_else() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_result(Err("".to_string()));
             yield v;
        };
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
             yield v;
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello world!!");
    }

    #[test]
    fn rio_merge_error() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::merge(
                RustIO::from_option(Some("hello".to_string())), RustIO::from_option(None),
                |a,b| RustIO::from_option(Some(a + &b)));
             yield v;
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.is_ok(), false);
    }

    #[test]
    fn rio_zip() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::zip(
                || RustIO::from_option(Some("hello".to_string())), || RustIO::from_option(Some(" world!!".to_string())),
                |a,b| RustIO::from_option(Some(a + &b)));
            yield v;
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello world!!");
    }

    #[test]
    fn rio_parallel() {
        let mut parallel_tasks: Vec<fn() -> RustIO<String, String>> = vec!();
        parallel_tasks.push(|| RustIO::from_option(Some("hello".to_string())));
        parallel_tasks.push(|| RustIO::from_result(Ok(" world".to_string())));
        parallel_tasks.push(|| RustIO::of("!!".to_string()));

        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::parallel(parallel_tasks,|tasks| RustIO::of(tasks.into_iter().collect()));
             yield v;
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello world!!");
    }

    #[test]
    fn rio_map_error() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_result(Err(String::from("Error A")))
                .map_error(|t| String::from("Error B"));
            yield v;
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.failed(), "Error B");
    }

    #[test]
    fn rio_when() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_option(Some(String::from("hello")))
                        .when(|v| v.len() > 3, |v| v + &" world!!".to_string());
             yield v;
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello world!!");
    }

    #[test]
    fn rio_when_rio() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_option(Some(String::from("hello")))
                        .when_rio(|v| v.len() > 3, |v| RustIO::from_option(Some(v + &" world!!".to_string())));
             yield v;
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello world!!");
    }

    #[test]
    fn rio_peek() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_option(Some(String::from("hello world!!")))
                .peek(|v| println!("${}",v));
             yield v;
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello world!!");
    }

    #[test]
    fn rio_fork() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_option(Some(String::from("hello")))
                        .fork(|v| {
                            println!("Fork. Variable:{} in Thread:{:?}", v, thread::current().id());
                            return v.to_uppercase();
                        })
                        .join();
             x <- RustIO::from_option(Some(String::from(" world!!")))
                    .peek(|v| println!("Join. Variable:{} in Thread:{:?}", v, thread::current().id()));
             yield v + &x;
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "HELLO world!!");
    }

    #[test]
    fn rio_daemon() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_option(Some(String::from("hello world!!")))
                .daemon(|v| println!("${}",v));
             yield v;
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello world!!");
    }

    #[test]
    fn rio_on_success() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_result(Ok(String::from("hello world!!")))
                .on_success(|v| println!("Success program: ${}",v));
             yield v;
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello world!!");
    }

    #[test]
    fn rio_on_error() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_result(Err(String::from("burning world!!")))
                .on_error(|v| println!("Error program: ${}",v));
             yield v;
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.is_failed(), true);
    }

    #[test]
    fn rio_eventually() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_result(Ok("hello".to_string()))
                .at_some_point(|v| get_eventual_result( v));
             yield v;
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.is_failed(), false);
    }

    #[test]
    fn rio_eventually_while() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_result(Ok("hello".to_string()))
                .at_some_point_while(|| true,|v| get_eventual_result( v));
             yield v;
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.is_failed(), false);
    }

    #[test]
    fn rio_eventually_until() {
        let rio_program: RustIO<String, String> = rust_io! {
             v <- RustIO::from_result(Ok("hello".to_string()))
                .at_some_point_until(|| {
                    std::thread::sleep(Duration::from_millis(100));
                    false
                },|v| get_eventual_result( v));
             yield v;
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.is_failed(), false);
    }

    #[test]
    fn features() {
        let rio_program: RustIO<String, String> =
            RustIO::from_option(Some(String::from("hello")))
                .when(|v| v.len() > 3, |v| v + &" world!!".to_string())
                .at_some_point(|v| get_eventual_result(v))
                .map(|v| v.to_uppercase())
                .flat_map(|v| RustIO::of(v + &"!!!".to_string()))
                .filter(|v| v.len() > 10)
                .delay(Duration::from_secs(1))
                .on_error(|v| println!("Error program: ${}", v))
                .map_error(|t| String::from("Error B"))
                .on_success(|v| println!("Success program: ${}", v))
                .peek(|v| println!("${}", v));

        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(false, rio_program.is_empty());
        assert_eq!(true, rio_program.is_ok());

    }

    fn get_eventual_result(v: String) -> RustIO<String, String> {
        let mut rng = thread_rng();
        let n: i32 = rng.gen_range(0..100);
        println!("${}", n);
        if n < 90 {
            eprintln!("Returning error");
            RustIO::from_result(Err("Error".to_string()))
        } else {
            eprintln!("Returning success");
            RustIO::from_result(Ok(v + &"world".to_string()))
        }
    }
}