

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
fn reference_dereference() {
    let x = 5;
    let y = &x; //set y to a reference to x
    assert_eq!(5, x);
    let i = *y;
    assert_eq!(5, i); // dereference y
}

// Read and mutable references
//----------------------------

/**
`&T` creates a read-only borrow.
You can have many read-only references at the same time, and the owner is still usable
after those references are no longer used.
*/
fn read_only_references() -> usize {
    let text = String::from("read only borrow");

    let first_reference = &text;
    let second_reference = &text;

    println!("{} / {}", first_reference, second_reference);

    text.len()
}

/**
`&mut T` creates a mutable borrow.
The owner must be declared as `mut`, and while the mutable borrow is alive, nobody
else can read or write the same value.
*/
fn mutable_reference_changes_owner() -> String {
    let mut text = String::from("hello");

    {
        let writable_reference = &mut text;
        writable_reference.push_str(" rust");
    }

    text.push_str("!");
    text
}

/**
This example is commented because it does not compile.

Even if `temporary_text` is mutable and the reference is `&mut String`, the
reference cannot live longer than the owner. When the inner scope ends,
`temporary_text` is destroyed, so `writable_reference` would point to invalid memory.

If the same value must stay alive through several owners outside one scope, that
is shared ownership. In single-thread code, that usually means `Rc<T>` for shared
read-only ownership or `Rc<RefCell<T>>` for shared mutable ownership.
*/
fn mutable_reference_cannot_outlive_owner_scope() {
    // let writable_reference: &mut String;
    //
    // {
    //     let mut temporary_text = String::from("hello");
    //     writable_reference = &mut temporary_text;
    //     writable_reference.push_str(" rust");
    // }
    //
    // writable_reference.push_str(" after scope");
    // println!("{}", writable_reference);
}

/**
The reference binding does not need to be `mut` to modify the pointed value.
`&mut text` means the pointed value can be changed.
`let mut writable_reference` would only mean the reference variable can be reassigned.
*/
fn mutable_reference_binding_does_not_need_mut() -> String {
    let mut text = String::from("hello");

    let writable_reference = &mut text;
    writable_reference.push_str(" rust");

    text
}

/**
A read-only borrow must finish before a mutable borrow starts.
The scope makes that boundary explicit.
*/
fn immutable_borrow_then_mutable_borrow() -> String {
    let mut text = String::from("hello");
    
    {
        let read_only_reference = &text;
        println!("Read first: {}", read_only_reference);
    }
    {
        let writable_reference = &mut text;
        writable_reference.push_str(" rust");
    }

    text
}

fn read_and_mutable_references() {
    println!("Read-only length: {}", read_only_references());
    println!("Mutable reference: {}", mutable_reference_changes_owner());
    mutable_reference_cannot_outlive_owner_scope();
    println!(
        "Mutable reference binding: {}",
        mutable_reference_binding_does_not_need_mut()
    );
    println!(
        "Read then mutable borrow: {}",
        immutable_borrow_then_mutable_borrow()
    );

    // This would not compile because `read_only_reference` is `&String`,
    // not `&mut String`.
    //
    // let mut text = String::from("hello");
    // let read_only_reference = &text;
    // read_only_reference.push_str(" rust");

    // This would not compile because the owner is not declared as mutable.
    //
    // let text = String::from("hello");
    // let writable_reference = &mut text;

    // This would not compile because a mutable borrow cannot coexist with
    // another active read-only borrow of the same value.
    //
    // let mut text = String::from("hello");
    // let read_only_reference = &text;
    // let writable_reference = &mut text;
    // println!("{} {}", read_only_reference, writable_reference);
}

// Lifetimes with borrowed structs
//--------------------------------

#[derive(Debug, Eq, PartialEq)]
struct UserRecord {
    name: String,
    role: String,
}

#[derive(Debug, Eq, PartialEq)]
struct UserView<'a> {
    name: &'a str,
    role: &'a str,
}

/**
`UserRecord` owns the Strings.
`UserView<'a>` only borrows string slices from a `UserRecord`.

The lifetime `'a` does not make anything live longer. It tells the compiler:
"this view is valid only while the borrowed record is still valid".
*/
fn user_view(record: &UserRecord) -> UserView<'_> {
    UserView {
        name: record.name.as_str(),
        role: record.role.as_str(),
    }
}

/**
The returned reference comes from one of the two input references.
Using the same lifetime `'a` tells Rust that the output cannot outlive the inputs.

"Inputs" here means the borrowed string data behind `left` and `right`, not the
local reference variables themselves. `left` and `right` disappear when the
function returns, but the strings they point to may still be alive outside.
*/
fn longest<'a>(left: &'a str, right: &'a str) -> &'a str {
    if left.len() >= right.len() {
        left
    } else {
        right
    }
}

fn lifetimes_with_borrowed_structs() {
    let record = UserRecord {
        name: "Politrons".to_string(),
        role: "rust developer".to_string(),
    };

    let view = user_view(&record);
    let most_descriptive_field = longest(view.name, view.role);

    println!("Borrowed user view: {:?}", view);
    println!("Longest borrowed field: {}", most_descriptive_field);

    // This would not compile because `view` would outlive `temporary_record`.
    //
    // let view;
    // {
    //     let temporary_record = UserRecord {
    //         name: "temporary".to_string(),
    //         role: "admin".to_string(),
    //     };
    //     view = user_view(&temporary_record);
    // }
    // println!("{:?}", view);

    longest_lifetime_restriction_examples();
}

fn longest_lifetime_restriction_examples() {
    let long_lived = String::from("borrow checker");

    {
        let short_lived = String::from("rust");
        let result = longest(long_lived.as_str(), short_lived.as_str());

        println!("Valid longest result inside the short scope: {}", result);
    }

    // This would not compile because `result` could point to `short_lived`,
    // but `short_lived` is destroyed before the final println.
    //
    // let result;
    // {
    //     let short_lived = String::from("rust");
    //     result = longest(long_lived.as_str(), short_lived.as_str());
    // }
    // println!("{}", result);

    // This would not compile either. The function would return a reference to
    // `text`, but `text` is destroyed when the function returns.
    //
    // fn invalid_reference<'a>() -> &'a str {
    //     let text = String::from("created inside the function");
    //     text.as_str()
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn run_example() {
        owner_variable();
        borrow_variable();
        reference_dereference();
        read_and_mutable_references();
        lifetimes_with_borrowed_structs();
    }
    
    #[test]
    fn user_view_borrows_from_record() {
        let record = UserRecord {
            name: "Politrons".to_string(),
            role: "rust developer".to_string(),
        };

        let view = user_view(&record);

        assert_eq!(
            view,
            UserView {
                name: "Politrons",
                role: "rust developer"
            }
        );
    }


    #[test]
    fn longest_returns_one_of_the_input_references() {
        assert_eq!(longest("rust", "borrow checker"), "borrow checker");
    }

    #[test]
    fn read_only_references_keep_owner_available() {
        assert_eq!(read_only_references(), "read only borrow".len());
    }

    #[test]
    fn mutable_reference_changes_the_original_owner() {
        assert_eq!(mutable_reference_changes_owner(), "hello rust!");
    }

    #[test]
    fn mutable_reference_binding_can_be_immutable() {
        assert_eq!(mutable_reference_binding_does_not_need_mut(), "hello rust");
    }

    #[test]
    fn read_borrow_can_finish_before_mutable_borrow() {
        assert_eq!(immutable_borrow_then_mutable_borrow(), "hello rust");
    }
}
