
/// In rust we dont have Union type like in Scala.
/// But using enum type to create only the allowed types we want to union in a condition
/// invocation is fine.
enum UnionType {
    Apple(String),
    Banana(u32),
    Coconut()
}

/// In this example we only allow three possible types. In Scala it would be [Apple | Banana | Coconut]
fn allow_only_fruit(fruit:UnionType){
    match fruit   {
        UnionType::Apple(s) => println!("You're a Apple"),
        UnionType::Banana(i) => println!("You're a Banana"),
        UnionType::Coconut() => println!("You're a Coconut"),
    }
}


#[cfg(test)]
mod tests {
    use crate::features::union_type::UnionType::{Apple, Banana, Coconut};
    use super::*;

    #[test]
    fn enum_type() {
        allow_only_fruit(Apple(String::from("apple type")));
        allow_only_fruit(Banana(1981));
        allow_only_fruit(Coconut())

    }
}
