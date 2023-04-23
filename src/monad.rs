pub fn run() {
    let my_monad = MyMonad::of("hello monad try");
    println!("Try:{}", my_monad.get().to_string());

    let monad_program = MyMonad::of("hello monad try".to_string())
        .map(|v| v.to_uppercase())
        .flat_map(|v| MyMonad { value: v + &"!!!"});
    println!("Program:{}", monad_program.get().to_string());

}

/**Trait interface like in Scala, where we define functions to implement*/
trait Monad<T> {
    fn of(v: T) -> MyMonad<T>;
    fn get(self) -> T;
    fn map(self, func: fn(T) -> T) -> MyMonad<T>;
    fn flat_map(self, func:fn(T) -> MyMonad<T>) -> MyMonad<T>;
}

/**Type to be used as implementation type for [Monad] trait*/
struct MyMonad<T> {
    value: T,
}

/**
Same syntax like in goLang where we define [impl] of the trait type, and then we use
[for] operator to specify over which type class we implement the trait,
 */
impl<T> Monad<T> for MyMonad<T> {
    //Constructor function to create the monad
    fn of(v: T) -> MyMonad<T> {
        MyMonad { value: v }
    }
    //Supplier function to extract the value from the monad
    fn get(self) -> T {
        self.value
    }
    //Transformation operator to get the value from the monad and transform using the function
    fn map(self, func: fn(T) -> T) -> MyMonad<T> {
        MyMonad { value: func(self.value) }
    }
    //Composition operator to compose two monad try
    fn flat_map(self, func: fn(T) -> MyMonad<T>) -> MyMonad<T> {
       func(self.value)
    }
}