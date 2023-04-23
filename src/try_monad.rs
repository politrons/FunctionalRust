pub fn run() {
    let try_monad = Try::of("hello monad try");
    println!("{}", try_monad.get().to_string());

}

/**Trait interface like in Scala, where we define functions to implement*/
trait TryMonad<T> {
    fn of( v: T) -> Try<T>;
    fn get(self) -> T;
    fn map(t: T, m: fn(T) -> T) -> T;
}

/**Type to be used as implementation type for [TryMonad] trait*/
struct Try<T> {
    value: T,
}

/**
Same syntax like in goLang where we define [impl] of the trait type, and then we use
[for] operator to specify over which type class we implement the trait,
*/
impl<T> TryMonad<T> for Try<T>{
    fn of(v: T) -> Try<T> {
        Try { value: v }
    }
    fn get(self) -> T {
        self.value
    }
    fn map(t: T, m: fn(T) -> T) -> T {
        m(t)
    }
}