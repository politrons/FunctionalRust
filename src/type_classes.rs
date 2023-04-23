pub fn run() {
    let int_result = TypeClassImpl::sum(10,20);
    println!("Integer:{}", int_result.to_string());

    let str_result = TypeClassImpl::sum("Hello".to_string(),"world".to_string());
    println!("String:{}", str_result.to_string());

    let bool_result = TypeClassImpl::sum(true,true);
    println!("Boolean:{}", bool_result.to_string());

    let float_result = TypeClassImpl::sum(30.5,10.1);
    println!("Float:{}", float_result.to_string());

}

/**Trait interface like in Scala, where we define the function to implement by type classes*/
trait TypeClass<T> {
    fn sum(t1: T, t2: T) -> T;
}

/**
Type to be used as generic type implementation for [TypeClass] trait.
Once we have implementation of the trait, we can reference this struct, passing
the specific type of the generic, to be redirect to the specific implementation.
 */
struct TypeClassImpl<T> {
    value: T,
}

/**
We define [impl] of the trait type [TypeClass], and then we use
[for] operator to specify over which type class we implement the trait.
In this case we implement [u64] type class
*/
impl TypeClass<u64> for TypeClassImpl<u64>{
    fn sum(t1: u64, t2: u64) -> u64 {
        t1 + t2
    }
}

/**
Type class of [String] type class
*/
impl TypeClass<String> for TypeClassImpl<String>{
    fn sum(t1: String, t2: String) -> String {
        t1 + &t2.to_string()
    }
}

/**
Type class of [f64] type class
 */
impl TypeClass<f64> for TypeClassImpl<f64>{
    fn sum(t1: f64, t2: f64) -> f64 {
        t1 + t2
    }
}

/**
Type class of [bool] type class
 */
impl TypeClass<bool> for TypeClassImpl<bool>{
    fn sum(t1: bool, t2: bool) -> bool {
        t1 && t2
    }
}