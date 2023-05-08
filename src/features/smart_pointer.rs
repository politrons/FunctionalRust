use std::cell::{RefCell, RefMut};
use std::ops::Add;
use std::rc::Rc;

/**
RC Also known as ADT(Abstract Data Type) it provides shared ownership of an immutable value.
Allow you create a pointer over a type, and being able to share in multiple contexts.

RC [Reference Counted] is a single-thread pointer of a specific type using [new]
Once we have the smart pointer, we can [clone] to create [borrowers].
 */
pub fn run() {
    boxer_features();
    struct_type();
    primitive_type();
    comparator_pointer();
    mutable_pointer();
}

/**
[Box] allow you to store data on the heap rather than the stack.
It´s useful when you have types that you want to extend their side in runtime.
The way we can extract the value from a [Box] is using [*]
 */
fn boxer_features() {
    let int_box = Box::new(1981);
    println!("{}", int_box.gt(&Box::new(100)));
    let raw_int = *int_box;
    println!("{}", raw_int);
}

/**
In Rc we can create a [clone] from original value.
Using [*] we can unwrap the type from the [Rc]
In rust [{}] create a new scope, and all variables created inside that scope it will have that lifecycle.
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
Rc it works also fine with struct types. But since it does not implement comparable, we cannot use the
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
One way to modify a pointer is to wrap the value of [RC] into [RefCell].
Then using [borrow_mut] operator we can get a [RefMut] that allow modify a type
that is the heap memory like String.

In this example we create [mutable] [borrows] of the original [owner] type, that once we modify,
the original owner type have the change.
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