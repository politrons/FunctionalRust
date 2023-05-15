/**
In rust we can use [struct] as [New-types] which is an opaque wrapper for a type.
[New-types] are a zero-cost abstraction – there’s no runtime overhead
*/
pub fn run() {
    let user = User(String::from("Politrons"));
    let password = Password(String::from("fgsafdsak"));
    login(user, password)
}

/**
[New-types] it has anonymous access, and is defined as a tuple of only one element.
So the way we access one element is by his array numeric position.
*/
fn login(user: User, pass: Password) {
    println!("Login user:{:?} password:{:?}", user.0, pass.0)
}

struct User(String);

struct Password(String);
