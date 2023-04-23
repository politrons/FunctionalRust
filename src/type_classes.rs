pub fn run() {
    let int_result = TypeClassImpl::sum(10,20);
    println!("{}", int_result.to_string());
    let str_result = TypeClassImpl::sum("Hello".to_string(),"world".to_string());
    println!("{}", str_result.to_string());

}

/**Trait interface like in Scala, where we define functions to implement*/
trait TypeClass<T> {
    fn sum(t1: T, t2: T) -> T;
}

/**Type to be used as implementation type for [TryMonad] trait*/
struct TypeClassImpl<T> {
    value: T,
}

/**
Same syntax like in goLang where we define [impl] of the trait type, and then we use
[for] operator to specify over which type class we implement the trait,
*/
impl TypeClass<u64> for TypeClassImpl<u64>{
    fn sum(t1: u64, t2: u64) -> u64 {
        t1 + t2
    }
}

impl TypeClass<String> for TypeClassImpl<String>{
    fn sum(t1: String, t2: String) -> String {
        t1 + &t2.to_string()
    }
}