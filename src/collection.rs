use std::collections::HashMap;

pub fn run() {
    array();
    vector();
    list();
    fold_list();
    flat_map_list();
    immutable_map_collection();
    mutable_map_collection();
}

/**
Superpower array type which can allow you to map, get, contains and other operators.
*/
fn array() {
    let list = [1, 2, 3, 4, 5]
        .map(|v| v + 10);
    println!("{:?}", list)
}

fn vector() {
    println!("{:?}", vec![1, 2, 3, 4]);
    let mut vector = vec![1, 2, 3, 4];
    vector.push(5);
    println!("{:?}", vector);
}


/**
We can create an Iterator type from an Array, just using [into_iter] which it will bring all functional operators
to transform [map], concatenate [flat_map] or [filter].
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
    let result = ["hello", "functional", "rust", "world", ]
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
    let result = ["hello", "functional", "rust", "world", ]
        .into_iter()
        .fold("-->".to_string(), |acc, elem| acc.to_string() + &"-".to_string() + &elem);
    println!("{}", result)
}

/**
immutable map is bu design the default option when you create in rust all data types. Here there is no different.
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
