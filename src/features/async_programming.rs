use futures::executor::block_on;
use futures::{pin_mut, select, FutureExt};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{thread, time};
use std::pin::Pin;
use std::task::{Context, Poll};

pub fn run() {
    async_block();
    composition();
    block_on(parallel_tasks());
    async_with_arguments();
    if let Err(error) = block_on(async_pipeline()) {
        eprintln!("Async pipeline example failed: {}", error);
    }
}

/**
Provide the execution of a task in one rust [coroutine] returning a [future] of the type specify in the function.
In order to execute in this [coroutine] is as simple as mark the function with async at the beginning.
it's by default lazy execution, and only when you [poll] or wrap it up in a [block_on] function,
is when is executed.
 */
fn async_block() {
    let future = async_hello_world().map(|v| v.to_uppercase());

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

async fn dependency_b(future_dep_a: impl Future<Output = String>) -> String {
    future_dep_a.await + &String::from("Async ")
}

async fn dependency_c(future_dep_b: impl Future<Output = String>) -> String {
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
    let future2 = async { String::from("World") };

    let (v1, v2) = futures::join!(future1, future2);
    println!("{} {}", v1, v2)
}

/**
It's also possible pass arguments into a async task using [async move] closure, where the variable
it can be used then in the scope of the future.
 */
fn async_with_arguments() {
    let value = String::from("hello world out of Thread");
    let future =
        async move { println!("Variable:{} in Thread:{:?}", value, thread::current().id()) };
    block_on(future)
}

/// In Rust we can also implement [Fire & Forget] pattern.
/// We only need to have an invocation of a async method which return a [Future]
/// Then use this future passing into [async_std::task::spawn]
fn fire_and_forget() {
    let future = async {
        std::thread::sleep(Duration::from_secs(2));
        println!("Hello fire and forget ${:?}", std::time::Instant::now());
    };
    let _ = async_std::task::spawn(future);
    println!(
        "Continue execution without blocks ${:?}",
        std::time::Instant::now()
    );
    std::thread::sleep(Duration::from_secs(4));
}

// Async runtime patterns
//--------------------------

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum WorkError {
    Timeout,
    ChannelClosed,
}

impl fmt::Display for WorkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkError::Timeout => formatter.write_str("work timed out"),
            WorkError::ChannelClosed => formatter.write_str("channel was closed"),
        }
    }
}

impl Error for WorkError {}

#[derive(Debug, Eq, PartialEq)]
struct ProcessReport {
    output: Vec<String>,
    backpressure_events: usize,
}

/**
This is backpressure because the channel is bounded to one queued message.
When the queue is full, the producer cannot keep pushing work forever: `try_send`
returns `Full`, and the producer must wait on `send(...).await` until the consumer
receives something.

The timeout wraps the complete producer + consumer pipeline so callers do not wait forever.
*/
async fn process_with_backpressure_and_timeout(
    input: Vec<String>,
    max_duration: Duration,
) -> Result<ProcessReport, WorkError> {
    let (sender, receiver) = async_std::channel::bounded::<String>(1);

    let producer = async_std::task::spawn(async move {
        let mut backpressure_events = 0;

        for item in input {
            match sender.try_send(item) {
                Ok(()) => {}
                Err(async_std::channel::TrySendError::Full(item)) => {
                    backpressure_events += 1;
                    sender
                        .send(item)
                        .await
                        .map_err(|_| WorkError::ChannelClosed)?;
                }
                Err(async_std::channel::TrySendError::Closed(_)) => {
                    return Err(WorkError::ChannelClosed);
                }
            }
        }

        Ok::<usize, WorkError>(backpressure_events)
    });

    let consumer = async_std::task::spawn(async move {
        let mut output = Vec::new();

        while let Ok(item) = receiver.recv().await {
            output.push(item.to_uppercase());
        }

        Ok::<Vec<String>, WorkError>(output)
    });

    match async_std::future::timeout(max_duration, async {
        let backpressure_events = producer.await?;
        let output = consumer.await?;

        Ok(ProcessReport {
            output,
            backpressure_events,
        })
    })
    .await
    {
        Ok(result) => result,
        Err(_) => Err(WorkError::Timeout),
    }
}

/**
Real Pin use case: `select!` polls several futures until one of them completes.
Those futures must stay in the same memory location while they are being polled,
because an async block is compiled into a state machine that can contain internal references.

`pin_mut!` pins the local futures on the stack. Once pinned, `select!` can safely poll
`api_call` and `timeout` without moving them.
*/
async fn api_call_with_timeout_using_pin(
    api_duration: Duration,
    max_wait: Duration,
) -> Result<String, WorkError> {
    let api_call = async move {
        async_std::task::sleep(api_duration).await;
        "remote api response".to_string()
    }
    .fuse();

    let timeout = async move {
        async_std::task::sleep(max_wait).await;
    }
    .fuse();

    pin_mut!(api_call, timeout);

    select! {
        response = api_call => Ok(response),
        _ = timeout => Err(WorkError::Timeout),
    }
}

async fn async_pipeline() -> Result<Vec<String>, WorkError> {
    let api_response =
        api_call_with_timeout_using_pin(Duration::from_millis(10), Duration::from_millis(100))
            .await?;

    println!("Pinned select response: {}", api_response);

    let report = process_with_backpressure_and_timeout(
        vec![
            "rust".to_string(),
            "async".to_string(),
            "backpressure".to_string(),
        ],
        Duration::from_secs(1),
    )
    .await?;

    println!("Backpressure events: {}", report.backpressure_events);
    Ok(report.output)
}

// User repo use case
// ------------------

trait UserService {
    async fn get_user_name(&self) -> &str;
    async fn get_user_income(&self) -> f32;

    async fn increase_income(&mut self, id: String, amount: f32);
}

struct User {
    id: String,
    name: String,
    income_repo: Arc<Mutex<IncomeRepo>>,
}

impl User {
    fn new(id: String, name: String, income_repo: Arc<Mutex<IncomeRepo>>) -> Self {
        User {
            id,
            name,
            income_repo,
        }
    }
}

struct IncomeRepo {
    incomes: HashMap<String, f32>,
}
impl IncomeRepo {
    fn new() -> Self {
        IncomeRepo {
            incomes: HashMap::from([
                (String::from("1000"), 100.1),
                (String::from("1001"), 1001.1),
                (String::from("2000"), 2999.4),
            ]),
        }
    }

    fn find_income_by_id(self, id: String) -> Option<f32> {
        self.incomes.get(&id).cloned()
    }
}

use rand::{thread_rng, Rng};

impl UserService for User {
    async fn get_user_name(&self) -> &str {
        let delay_time = rand::thread_rng().gen_range(500..=1000);
        async_std::task::sleep(Duration::from_millis(delay_time)).await;
        &self.name
    }

    async fn get_user_income(&self) -> f32 {
        self.income_repo
            .lock()
            .unwrap()
            .incomes
            .get(&self.id)
            .cloned()
            .unwrap_or(0.0)
    }

    async fn increase_income(&mut self, id: String, amount: f32) {
        let current_income = self.get_user_income().await;
        self.income_repo
            .lock()
            .unwrap()
            .incomes
            .insert(id, current_income + amount);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::future::join;
    use std::sync::Arc;

    #[test]
    fn user_service_test() {
        let income_repo = Arc::new(Mutex::new(IncomeRepo::new()));
        let mut user_1 = User::new(
            String::from("1000"),
            String::from("politrons"),
            Arc::clone(&income_repo),
        );
        let mut user_2 = User::new(
            String::from("1001"),
            String::from("John"),
            Arc::clone(&income_repo),
        );
        block_on(async  {
            let username_1_fut = user_1.get_user_name()
                .map(|username| username.to_uppercase())
                .fuse();
            let username_2_fut = user_2.get_user_name()
                .fuse();

            pin_mut!(username_1_fut, username_2_fut);

            select! {
            username_1 = username_1_fut => {
                println!("Username 1: {}", username_1);
            }
            username_2 = username_2_fut => {
                println!("Username 2: {}", username_2);
            }
        }
        });

        let income_user_1_fut = user_1.get_user_income();
        let income_user_2_fut = user_2.get_user_income();

        //Check income
        let join_income = join(income_user_1_fut, income_user_2_fut);
        let tuple = block_on(join_income);
        println!("User 1 income: {}", tuple.0);
        println!("User 2 income: {}", tuple.1);

        //Increase income
        let increase_income_user1_fut = user_1.increase_income(String::from("1000"), 1500.0);
        let increase_income_user2_fut = user_2.increase_income(String::from("1001"), 1000.0);
        let join_increase = join(increase_income_user1_fut, increase_income_user2_fut);
        block_on(join_increase);

        let income_user_1_fut = user_1.get_user_income();
        let income_user_2_fut = user_2.get_user_income();
        //
        let join_income = join(income_user_1_fut, income_user_2_fut);
        let tuple = block_on(join_income);
        println!("User 1 income after update: {}", tuple.0);
        println!("User 2 income after update: {}", tuple.1);
    }

    #[test]
    fn fire_and_forget_test() {
        fire_and_forget()
    }

    #[test]
    fn processes_with_backpressure_and_timeout() {
        let output = block_on(async_pipeline()).expect("demo should finish");

        assert_eq!(output, vec!["RUST", "ASYNC", "BACKPRESSURE"]);
    }

    #[test]
    fn bounded_channel_reports_backpressure_when_queue_is_full() {
        let report = block_on(process_with_backpressure_and_timeout(
            vec!["one".to_string(), "two".to_string(), "three".to_string()],
            Duration::from_secs(1),
        ))
        .expect("pipeline should finish");

        assert_eq!(report.output, vec!["ONE", "TWO", "THREE"]);
        assert!(report.backpressure_events > 0);
    }

    #[test]
    fn pin_select_returns_timeout_when_timeout_future_wins() {
        let result = block_on(api_call_with_timeout_using_pin(
            Duration::from_millis(100),
            Duration::from_millis(10),
        ));

        assert_eq!(result, Err(WorkError::Timeout));
    }

    #[test]
    fn pin_select_returns_api_response_when_api_future_wins() {
        let result = block_on(api_call_with_timeout_using_pin(
            Duration::from_millis(10),
            Duration::from_millis(100),
        ));

        assert_eq!(result, Ok("remote api response".to_string()));
    }
}
