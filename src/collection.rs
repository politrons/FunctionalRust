pub fn run() {
    array();
    vector();
    list();
    fold_list();
    flat_map__list();
}


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

fn flat_map__list() {
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
