use std::thread;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

pub fn thread_pool() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(10)
        .thread_name_fn(|| {
            static WORKER_ID: AtomicUsize = AtomicUsize::new(1);
            let id = WORKER_ID.fetch_add(1, Ordering::SeqCst);
            format!("TokioPool-{id}")
        })
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        let mut tasks = Vec::new();

        for i in 1..=100 {
            tasks.push(tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(10)).await;
                println!("Task {} thread name {:?}", i, std::thread::current().name());
            }));
        }

        for task in tasks {
            task.await.unwrap();
        }
    });
}

mod test {
    #[test]
    fn test_thread_pool() {
        super::thread_pool();
    }
}