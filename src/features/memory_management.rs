pub fn run(){
    owner_variable();
    borrow_variable();
    reference_dereference();
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


/**
When we use [&] we are creating a reference [pointer] of a variable.
And when we use [*] we are de-referencing a reference.
*/
fn reference_dereference(){
    let x = 5;
    let y = &x; //set y to a reference to x
    assert_eq!(5, x);
    assert_eq!(5, *y); // dereference y
}
