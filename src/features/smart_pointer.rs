use std::cell::{RefCell, RefMut};
use std::ops::Add;
use std::os::unix::raw::nlink_t;
use std::rc::Rc;

/**
`Rc<T>` provides several owners for the same heap value in one thread.
Cloning an `Rc` clones the pointer and increments the reference counter; it does
not clone the inner value.

`Rc<T>` only gives shared immutable access. If several owners must mutate the
same value in one thread, use `Rc<RefCell<T>>`.

For shared ownership across threads, use `Arc<T>`. If the shared value must be
mutated across threads, use `Arc<Mutex<T>>` or `Arc<RwLock<T>>`.
 */

/**
[Box] allow you to store data on the heap rather than the stack.
It´s useful when you have types that you want to extend their side in runtime.
The way we can extract the value from a [Box] is using [*]
 */
#[test]
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
#[test]
fn primitive_type() {
    let int_pointer = Rc::new(1981);
    {
        let second_pointer = int_pointer.clone();
        let value = *second_pointer;
        println!("{}", value);
    }
    println!("{}", *int_pointer);
}

/**
Every variable below owns one `Rc` handle, but all handles point to the same heap value.
`Rc::strong_count` shows how many owners are alive.
*/
#[test]
fn rc_multiple_owners_share_the_same_value() {
    let first_owner = Rc::new(HelloType {
        value: "shared profile",
    });
    assert_eq!(Rc::strong_count(&first_owner), 1);

    let second_owner = Rc::clone(&first_owner);
    let third_owner = Rc::clone(&first_owner);

    assert_eq!(Rc::strong_count(&first_owner), 3);
    assert_eq!(first_owner.value, "shared profile");
    assert_eq!(second_owner.value, "shared profile");
    assert_eq!(third_owner.value, "shared profile");

    drop(third_owner);

    assert_eq!(Rc::strong_count(&first_owner), 2);
}

/**
Using [Reference counter] we can use all comparison operator over the value in case is a comparable type.
Like here we can use eq,lt,gt,add over the value.
 */
#[test]
fn comparator_pointer() {
    let int_pointer = Rc::new(1981);
    println!("Equals:{}", int_pointer.eq(&Rc::new(1981)));
    println!("Lower than:{}", int_pointer.lt(&Rc::new(100)));
    println!("Greater than:{}", int_pointer.gt(&Rc::new(100)));
    println!("Greater than:{}", int_pointer.add(100));
}

/**
Rc it works also fine with struct types. But since it does not implement comparable, we cannot use the
previous example operators.
 */
#[test]
fn struct_type() {
    let type_pointer = Rc::new(HelloType { value: "hello smart pointer world" });
    {
        let second_pointer = &type_pointer;
        println!("{}", (*second_pointer).value);
    }
    println!("{}", (*type_pointer).value);

    let type_pointer1 = Rc::new(HelloType { value: 1981 });
    let type_pointer2 = Rc::new(HelloType { value: 100 });
    println!("Greater than {}", type_pointer1.value.gt(&type_pointer2.value));
}

/**
One way to modify a pointer is to wrap the value of [RC] into [RefCell].
Then using [borrow_mut] operator we can get a [RefMut] that allow modify a type
that is the heap memory like String.

In this example we create [mutable] [borrows] of the original [owner] type, that once we modify,
the original owner type have the change.
 */
#[test]
fn rc_mutable_pointer() {
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

/**
`Rc<RefCell<T>>` means:
- `Rc` gives several owners in the same thread.
- `RefCell` allows mutation through those owners at runtime.

The borrow checker rules still exist, but `RefCell` checks them while the program runs:
many immutable borrows or one mutable borrow, never both at the same time.
*/
#[test]
fn rc_refcell_allows_shared_mutation_in_one_thread() {
    let first_owner = Rc::new(RefCell::new(0));
    let second_owner = Rc::clone(&first_owner);
    let third_owner = Rc::clone(&first_owner);

    *first_owner.borrow_mut() += 1;
    *second_owner.borrow_mut() += 10;
    *third_owner.borrow_mut() += 100;

    assert_eq!(*first_owner.borrow(), 111);
    assert_eq!(*second_owner.borrow(), 111);
    assert_eq!(*third_owner.borrow(), 111);
}

#[test]
fn rc_keeps_value_alive_after_original_scope() {
    let second_owner: Rc<String>;

    {
        let temporary_owner =Rc::new(String::from("temp"));
        second_owner = Rc::clone(&temporary_owner);
        assert_eq!(Rc::strong_count(&second_owner), 2);
    }

    assert_eq!(Rc::strong_count(&second_owner), 1);
    assert_eq!(*second_owner, "temp");

}

/**
This is the smart-pointer version of the example that does not compile with `&mut String`.

The value is created inside an inner scope, but we clone the `Rc` handle into
`shared_text`. When the inner scope ends, `temporary_owner` is dropped, but the
heap value stays alive because `shared_text` is still an owner.

`RefCell` is needed because `Rc<T>` alone only allows shared read-only access.
*/
#[test]
fn rc_refcell_keeps_value_alive_after_original_scope() {
    let second_owner: Rc<RefCell<String>>;

    {
        let temporary_owner = Rc::new(RefCell::new(String::from("hello")));
        second_owner = Rc::clone(&temporary_owner);

        second_owner.borrow_mut().push_str(" rust");

        assert_eq!(Rc::strong_count(&second_owner), 2);
    }

    second_owner.borrow_mut().push_str(" after scope");

    assert_eq!(Rc::strong_count(&second_owner), 1);
    assert_eq!(*second_owner.borrow(), "hello rust after scope");
}

/**
This test is intentionally marked as `should_panic`.
`RefCell` lets us move borrow checking to runtime, but it does not remove the rules.
Trying to create two mutable borrows at the same time fails.
*/
#[test]
#[should_panic(expected = "already borrowed")]
fn refcell_rejects_two_mutable_borrows_at_runtime() {
    let shared_value = RefCell::new(0);
    let _first_mutable_borrow = shared_value.borrow_mut();
    let _second_mutable_borrow = shared_value.borrow_mut();
}

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/**
`Arc<T>` is shared ownership across threads.
Every cloned `Arc` points to the same heap value, but access is read-only unless
the inner type provides its own thread-safe mutability.
*/
#[test]
fn arc_allows_shared_reading_across_threads() {
    let shared_text = Arc::new(String::from("hello from arc"));

    let first_reader = Arc::clone(&shared_text);
    let second_reader = Arc::clone(&shared_text);

    let first = thread::spawn(move || first_reader.len());
    let second = thread::spawn(move || second_reader.contains("arc"));

    assert_eq!(first.join().expect("thread should finish"), 14);
    assert!(second.join().expect("thread should finish"));
    assert_eq!(Arc::strong_count(&shared_text), 1);
}

/**
This example is commented because it does not compile.

`Arc<String>` gives several owners, but it does not give mutable access to the
inner `String`. Without `Mutex`, `RwLock`, or another thread-safe mutability type,
Rust prevents mutation because several threads could access the same value at once.
*/
fn arc_without_mutex_cannot_mutate_shared_value() {
    // let shared_text = Arc::new(String::from("hello"));
    // let writer_text = Arc::clone(&shared_text);
    //
    // let writer = thread::spawn(move || {
    //     writer_text.push_str(" rust");
    // });
    //
    // writer.join().unwrap();
}

#[test]
fn arc_mutex() {
    let shared_value = Arc::new(Mutex::new(0));

    let writer_value = Arc::clone(&shared_value);

    let writer = thread::spawn(move || {
        thread::sleep(Duration::from_secs(1));

        // Block Mutex to allow update the value
        let mut value = writer_value.lock().unwrap();

        *value = 42;

        println!("Writer thread changed value to {}", *value);
    });
    let reader_value = Arc::clone(&shared_value);

    let reader = thread::spawn(move || {
        thread::sleep(Duration::from_secs(2));

        // Block the Mutex to read the updated value
        let value = reader_value.lock().unwrap();

        println!("Reader thread read value: {}", *value);
    });

    writer.join().unwrap();
    reader.join().unwrap();
}

/**
`Arc<Mutex<T>>` is the thread-safe version of `Rc<RefCell<T>>`.
- `Arc` gives several owners across threads.
- `Mutex` allows one thread at a time to mutate the value.
*/
#[test]
fn arc_mutex_allows_shared_mutation_across_threads() {
    let counter = Arc::new(Mutex::new(0));
    let mut workers = Vec::new();

    for _ in 0..4 {
        let worker_counter = Arc::clone(&counter);

        workers.push(thread::spawn(move || {
            let mut value = worker_counter.lock().expect("mutex should not be poisoned");
            *value += 1;
        }));
    }

    for worker in workers {
        worker.join().expect("thread should finish");
    }

    assert_eq!(*counter.lock().expect("mutex should not be poisoned"), 4);
}

struct HelloType<T> {
    value: T,
}
