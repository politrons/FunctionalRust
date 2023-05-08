use std::cell::{RefCell, RefMut};
use std::ops::Add;
use std::rc::Rc;

/**
RC [Reference Counted] is a single-thread pointer of a specific type using [new]
Once we have the smart pointer, we can [clone].

In rust [{}] create a new scope, and all variables created inside that scope it will have that lifecycle.
 */
pub fn run() {
    struct_type();
    primitive_type();
    comparator_pointer();
    mutable_pointer();
}

/**
In Rc we can create a [clone] from original value.
Using [*] we can unwrap the type from the [Rc]
 */
fn primitive_type() {
    let str_pointer = Rc::new(1981);
    {
        let second_pointer = str_pointer.clone();
        let value = *second_pointer;
        println!("{}", value);
    }
    println!("{}", *str_pointer);
}

/**
Using [Reference counter] we can use all comparison operator over the value in case is a comparable type.
Like here we can use eq,lt,gt,add over the value.
 */
fn comparator_pointer() {
    let str_pointer = Rc::new(1981);
    println!("Equals:{}", str_pointer.eq(&Rc::new(1981)));
    println!("Lower than:{}", str_pointer.lt(&Rc::new(100)));
    println!("Greater than:{}", str_pointer.gt(&Rc::new(100)));
    println!("Greater than:{}", str_pointer.add(100));
}

/**
Rc it work also fine with struct types. But since it does not implement comparable, we cannot use the
previous example operators.
 */
fn struct_type() {
    let type_pointer = Rc::new(HelloType { value: "hello smart pointer world" });
    {
        let second_pointer = type_pointer.clone();
        println!("{}", (*second_pointer).value);
    }
    println!("{}", (*type_pointer).value);

    let type_pointer1 = Rc::new(HelloType { value: 1981 });
    let type_pointer2 = Rc::new(HelloType { value: 100 });
    println!("Greater than{}", type_pointer1.value.gt(&type_pointer2.value));
}

/**
One way to

*/
fn mutable_pointer() {
    let shared_pointer = Rc::new(RefCell::new("Hello".to_string()));
    //Another scope
    {
        let mut str_reference: RefMut<String> = shared_pointer.borrow_mut();
        str_reference.push_str(" Mutable");
    }
    //Second scope
    {
        let mut sec_str_reference = shared_pointer.borrow_mut();
        sec_str_reference.push_str(" World!!");
    }
    println!("{}", shared_pointer.take());
}

struct HelloType<T> {
    value: T,
}