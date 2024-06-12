use rxrust::observable;
use rxrust::observable::{ObservableExt, ObservableItem};

fn main() {
    println!("Hello Reactive world!");
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;
    use rxrust::prelude::FuturesLocalSchedulerPool;
    use super::*;

    #[test]
    fn iter_observable() {
        // The `from_iter` operator creates an observable from an iterator.
        // It will emit each item of the iterator sequentially.
        // This is useful for converting an existing collection into a stream of items.
        observable::from_iter(0..10)
            .on_complete(|| println!("Stream completed"))
            .subscribe(|n| println!("{}", n));
    }

    #[test]
    fn filter_observable() {
        // The `filter` operator allows you to filter the items emitted by an observable.
        // Only the items that satisfy the provided predicate are emitted.
        // In this case, it filters out negative numbers.
        observable::from_iter(vec![1, 2, 3, -6, 4, 5, -8])
            .filter(|n| n > &0)
            .subscribe(|n| println!("{}", n));
    }

    #[test]
    fn map_observable() {
        // The `map` operator transforms each item emitted by an observable by applying a function to it.
        // It emits the transformed items as a new observable.
        // Here, it converts each string to uppercase.
        observable::from_iter(vec!["hello", "transforming", "reactive", "world", "from", "rust"])
            .map(|word| word.to_uppercase())
            .subscribe(|n| println!("{}", n));
    }

    #[test]
    fn flatmap_observable() {
        // The `flat_map` operator is used to transform the items emitted by an observable
        // into observables, then flatten these emissions into a single observable.
        // This is useful for chaining Observable operations.
        observable::from_iter(vec!["hello", "composition", "reactive", "world"])
            .flat_map(|word| {
                observable::from_iter(vec![word])
                    .map(|new_word| format!("[{}]", new_word))
            })
            .subscribe(|n| println!("{}", n));
    }

    #[test]
    fn merge_observable() {
        // The `merge` operator combines multiple observables into one by interleaving their emissions.
        // It emits items from all source observables as they arrive.
        // This test merges two streams: one emitting "hello" and the other emitting "world".
        observable::from_iter(vec!["hello"])
            .merge(observable::from_iter(vec!["world"]))
            .subscribe(|n| println!("{}", n));
    }

    #[test]
    fn zip_observable() {
        // The `zip` operator combines the emissions of multiple observables into a single observable
        // by pairing their items one-by-one.
        // It emits tuples where each tuple contains one item from each of the source observables.
        observable::from_iter(vec!["hello"])
            .zip(observable::from_iter(vec!["world"]))
            .subscribe(|(w, w1)| println!("Zip result {}-{}", w, w1));
    }

    #[test]
    fn take_observable() {
        // The `take` operator limits the number of items emitted by an observable.
        // It only emits the specified number of items from the start of the stream and then completes.
        // Here, it takes the first three items from the stream.
        observable::from_iter(vec!["hello", "reactive", "world", "from", "rust"])
            .take(3)
            .subscribe(|n| println!("{}", n));
    }

    #[test]
    fn take_while_observable() {
        // The `take_while` operator emits items from an observable until a specified condition is no longer met.
        // It stops emitting as soon as the condition fails.
        // This example emits items as long as their length is 5 or less characters.
        observable::from_iter(vec!["hello", "reactive", "world", "from", "rust"])
            .take_while(|w| w.len() <= 5)
            .subscribe(|n| println!("{}", n));
    }

    #[test]
    fn future_observable() {
        // The `from_future` operator converts a future into an observable.
        // The observable emits the result of the future when it completes.
        // Here, it converts a future that resolves to the string "hello future world".
        let mut scheduler_pool = FuturesLocalSchedulerPool::new();
        observable::from_future(std::future::ready("hello future world"), scheduler_pool.spawner())
            .on_complete(|| println!("Complete in thread {:?}", thread::current().name()))
            .subscribe(move |n| println!("{}", n));
        scheduler_pool.run()
    }

    #[test]
    fn subscribe_on_observable() {
        // The `subscribe_on` operator specifies the scheduler on which to subscribe to the observable.
        // This determines the thread that the observable's emissions are handled on.
        // It allows the observable to run asynchronously on a specified scheduler.
        let mut scheduler_pool = FuturesLocalSchedulerPool::new();
        observable::from_iter(vec!["hello", "reactive", "world", "from", "rust"])
            .subscribe_on(scheduler_pool.spawner())
            .on_complete(|| println!("Complete in thread {:?}", thread::current().name()))
            .subscribe(|n| println!("{}", n));
        scheduler_pool.run()
    }

    #[test]
    fn delay_observable() {
        // The `delay` operator delays the emission of items from an observable by a specified duration.
        // Each item is emitted after the delay period has passed.
        // This example delays each item by 500 milliseconds.
        let mut scheduler_pool = FuturesLocalSchedulerPool::new();
        observable::from_iter(vec!["hello", "reactive", "world", "from", "rust"])
            .delay(Duration::from_millis(500), scheduler_pool.spawner())
            .on_complete(|| println!("Complete in thread {:?}", thread::current().name()))
            .subscribe(|n| println!("{}", n));
        scheduler_pool.run()
    }
}
