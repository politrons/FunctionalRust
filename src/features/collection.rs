use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_std::prelude::FutureExt;
use async_std::task::block_on;
use dashmap::DashMap;

fn queue_features() {
    let mut queue = Vec::new();
    queue.push("hello");
    queue.push("world");
    queue.push("!!!");

    println!("{:?}", queue.pop().unwrap());
    println!("{:?}", queue.pop().unwrap());
    println!("{:?}", queue.pop().unwrap());
}

/**
Superpower array type which can allow you to map, get, contains and other operators.
*/
fn array() {
    let list = [1, 2, 3, 4, 5].map(|v| v + 10);
    println!("{:?}", list)
}

fn vector() {
    println!("{:?}", vec![1, 2, 3, 4]);
    let mut vector = vec![1, 2, 3, 4];
    vector.push(5);
    println!("{:?}", vector);
}

/**
We can create an Iterator type from an Array, just using [into_iter] which it will bring all advance functional operators
to concatenate [flat_map] or [filter].
 */
fn list() {
    let result = ["hello", "", "rust", "world", ""]
        .into_iter()
        .filter(|v| !v.is_empty())
        .map(|v| String::from("[") + &v.to_uppercase() + &String::from("]"))
        .collect::<String>();
    println!("{}", result)
}

/**
Iterator is also a Monad in Rust, so you can compose two iterators using [flat_map] operator
*/
fn flat_map_list() {
    let result = ["hello", "functional", "rust", "world"]
        .into_iter()
        .flat_map(|e| [e.to_string() + &"!"].into_iter())
        .collect::<String>();
    println!("{}", result)
}

/**
Fold operator is able just like in any other functional language, define an initial value type as first argument,
and then pass a bi-function with the accumulative value in the specific type we made before, and the new element
is on the collection.
 */
fn fold_list() {
    let result = ["hello", "functional", "rust", "world"]
        .into_iter()
        .fold("-->".to_string(), |acc, elem| {
            acc.to_string() + &"-".to_string() + &elem
        });
    println!("{}", result)
}

/**
Same than fold but iterating the collection from right to left
**/
fn right_fold() {
    let result = ["lets","use", "right", "fold"]
        .into_iter()
        .rfold("".to_string(), |acc, elem| {
                acc.to_string() + &" ".to_string() +  &elem
        });
    println!("{}", result)
}

#[derive(Debug)]
struct FoldError{}

fn fold_effect() {
    let result = ["lets","use", "right", "fold"]
        .into_iter()
        .try_fold("".to_string(), |acc, elem| -> Result<String, FoldError> {
            Ok(acc.to_string() + &" ".to_string() +  &elem)
        });
    println!("{}", result.unwrap())
}

fn fold_effect_failed() {
    let result = ["lets","use", "right", "fold"]
        .into_iter()
        .try_fold("".to_string(), |acc, elem| -> Result<String, FoldError> {
            Err(FoldError{})
        });
    println!("{}", result.is_err())
}


/**
immutable map is by design the default option when you create in rust all data types. Here there is no different.
A map it can also be converter in iterable using [into_iter] operator
*/
fn immutable_map_collection() {
    let map = HashMap::from([(1, "hello"), (2, "rust"), (3, "map")]);
    map.into_iter()
        .for_each(|(k, v)| println!("Key:{} Value:{}", k, v))
}

/**
In case you need a mutable map to add/delete records on runtime, as usual you need to use [mut], and then
you can use [insert] or [remove] operators
*/
fn mutable_map_collection() {
    let mut map = HashMap::new();
    map.insert(1, "hello");
    map.insert(2, "mutable");
    map.insert(3, "map");
    map.into_iter()
        .for_each(|(k, v)| println!("Key:{} Value:{}", k, v))
}

/**
Implementation the standard way to have a concurrent hash map in rust without use external crates
**/
fn concurrent_map_collection() {
    block_on(async {
        let map = Arc::new(Mutex::new(HashMap::new()));

        let map_1 = Arc::clone(&map);
        let task_1 = async_std::task::spawn(async move {
            map_1.lock().unwrap().insert(10, String::from("hello"));
        });
        let map_2 = Arc::clone(&map);
        let task_2 = async_std::task::spawn(async move {
            map_2.lock().unwrap().insert(20, String::from("concurrent"));
        });
        let map_3 = Arc::clone(&map);
        let task_3 = async_std::task::spawn(async move {
            map_3.lock().unwrap().insert(30, String::from("map"));
        });
        task_1.await;
        task_2.await;
        task_3.await;

        map.lock().unwrap().iter()
            .for_each(|tuple| println!("Key:{} Value:{}", tuple.0, tuple.1));
    })
}

/**
To avoid the use of Arc Mutex we can garantee concurrency of the map between threads using the concurrent hashmap DashMap from dashmap crate.
**/
fn concurrent_map_collection_dash_map() {
    block_on(async {
        let map = Arc::new(DashMap::new());

        let map_1 = Arc::clone(&map);
        let task_1 = async_std::task::spawn(async move {
            map_1.insert(100, "hello");
        });
        let map_2 = Arc::clone(&map);
        let task_2 = async_std::task::spawn(async move {
            map_2.insert(200, "concurrent");
        });
        let map_3 = Arc::clone(&map);
        let task_3 = async_std::task::spawn(async move {
            map_3.insert(300, "map");
        });
        task_1.await;
        task_2.await;
        task_3.await;

        map.iter()
            .for_each(|entry| println!("Key:{} Value:{}", entry.key(), entry.value()))
    })
}

fn append_vectors() {
    let a = vec![1, 2, 3];
    let b = vec![7, 8, 9];
    println!("{:?}", [a, b].concat());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_all() {
        array();
        vector();
        list();
        fold_list();
        right_fold();
        fold_effect();
        fold_effect_failed();
        flat_map_list();
        immutable_map_collection();
        mutable_map_collection();
        concurrent_map_collection();
        concurrent_map_collection_dash_map();
        append_vectors();
        queue_features();
    }
}
