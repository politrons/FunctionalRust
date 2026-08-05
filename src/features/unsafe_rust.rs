/**
Unsafe Rust should be small, explicit, and documented. The goal is to expose
a safe API around carefully constrained unsafe blocks.
*/

/**
repr(C) fixes memory layout for FFI or binary protocols.
Without it, Rust can reorder fields and no C-compatible layout is guaranteed.
*/
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WireHeader {
    len: u16,
    kind: u16,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn run() {
       let first =  unsafe {[1,2,3].get_unchecked(0)} ;
        let header = WireHeader { len: 3, kind: 1 };

        println!(
            "Unsafe encapsulated header:{:?} first value:{}",
            header, first
        );
    }

}
