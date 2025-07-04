# Detecting “Hello World” Validity in Rust

This project trains a binary **Logistic Regression** model (via the
[`linfa`](https://crates.io/crates/linfa) ecosystem) that decides whether a small
Rust code snippet prints **“Hello, world!”** correctly (`VALID = 1`) or
contains an error/variation (`INVALID = 0`).

---

## Dataset

* **File**: `hello_rust.csv`
* **Columns**  

  | column | type | description                           |
  | ------ | ---- | ------------------------------------- |
  | `code` | `String` | The Rust snippet (≤ a few lines)  |
  | `label` | `u8` (`0` or `1`) | Ground-truth verdict    |

For demonstration the repo also includes three hard-coded snippets in
`EXAMPLES`, shown again at prediction time:

```rust
fn main() { println!("Hello, world!"); } // VALID
fn mian() { println!("Hello, world!"); } // INVALID (typo)
fn main() { println!("Hola, mundo!"); }  // INVALID (different text)
