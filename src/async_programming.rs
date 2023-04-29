use std::future::Future;
use std::{thread, time};
use futures::executor::block_on;
use futures::{FutureExt, TryFutureExt, TryStreamExt};

pub fn run() {
    async_block();
    composition();
    block_on(parallel_tasks());
    async_with_arguments();
}


/**
Provide the execution of a task in one rust [coroutine] returning a [future] of the type specify in the function.
In order to execute in this [coroutine] is as simple as mark the function with async at the beginning.
it's by default lazy execution, and only when you [poll] or wrap it up in a [block_on] function,
is when is executed.
 */
fn async_block() {
    let future = async_hello_world()
        .map(|v| v.to_uppercase());

    let result = block_on(future);
    println!("{}", result);
}

async fn async_hello_world() -> String {
    String::from("Hello async world")
}

/**
In order to emulate composition of [futures] in rust, we can use await operator, which it will extract
the value from the future, once is ready. This operator it can be used only inside a async function since is a blocking operation.
 */
fn composition() {
    let future_program = dependency_c(dependency_b(dependency_a()));
    let result = block_on(future_program);
    println!("{}", result)
}

async fn dependency_a() -> String {
    String::from("Hello ")
}

async fn dependency_b(future_dep_a: impl Future<Output=String>) -> String {
    future_dep_a.await + &String::from("Async ")
}

async fn dependency_c(future_dep_b: impl Future<Output=String>) -> String {
    future_dep_b.await + &String::from("World ")
}

/**
We can also create futures just running some logic inside async closures.
It will automatically return a [future].
to run both futures in parallel we can use [join] operator which it will merge both result in a tuple (v1,v2)
 */
async fn parallel_tasks() {
    let future1 = async {
        thread::sleep(time::Duration::from_millis(1000));
        String::from("Hello")
    };
    let future2 = async {
        String::from("World")
    };

    let (v1, v2) = futures::join!(future1,future2);
    println!("{} {}", v1, v2)
}

/**
It's also possible pass arguments into a async task using [async move] closure, where the variable
it can be used then in the scope of the future.
 */
fn async_with_arguments() {
    let value = String::from("hello world out of Thread");
    let future = async move {
        println!("Variable:{} in Thread:{:?}", value, thread::current().id())
    };
    block_on(future)
}
