pub fn run(){
    owner_variable();
    borrow_variable();
}

/**
One of the best feature of [rust] by design is the memory management, and how protect our programs in compilation time.
Every variable allocated in heap memory, can only have one owner. So in case we decide to transfer the content of one
variable to another, the old one cannot be used anymore, and it wont compile if you want to use it.
*/
fn owner_variable() {
    let variable = String::from("Memory management:Transferring");
    let transfer_variable = variable;
    // println!("{}", variable);//It wont compile
    println!("{}", transfer_variable);
}

/**
One way that we can assign the content of one variable into another, is not doing a copy, but passing
a reference(pointer) [&] just like in c, c++.
Once we do that we can continue using the old variable since what we made with the new allocation is pass a reference.
*/
fn borrow_variable() {
    let variable = String::from("Memory management:Borrowing");
    let new_variable = &variable;
    println!("{}", variable);
    // println!("{}", variable.push_str("Change value"));//It wont compile
    println!("{}", new_variable);
}
