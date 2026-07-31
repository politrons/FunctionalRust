/**
Unsafe Rust should be small, explicit, and documented. The goal is to expose
a safe API around carefully constrained unsafe blocks.
*/
pub fn run() {
    let header = WireHeader { len: 3, kind: 1 };
    let first = NonEmptySlice::try_new(&[7, 8, 9])
        .map(|values| *values.first())
        .unwrap_or_default();

    println!(
        "Unsafe encapsulated header:{:?} first value:{}",
        header, first
    );
}

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

struct NonEmptySlice<'a, T> {
    values: &'a [T],
}

impl<'a, T> NonEmptySlice<'a, T> {
    fn try_new(values: &'a [T]) -> Option<Self> {
        if values.is_empty() {
            None
        } else {
            Some(Self { values })
        }
    }

    fn first(&self) -> &'a T {
        // Safety: try_new is the only constructor and it rejects empty slices.
        unsafe { self.values.get_unchecked(0) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_constructor_protects_unsafe_read() {
        assert!(NonEmptySlice::<u8>::try_new(&[]).is_none());

        let values = NonEmptySlice::try_new(&[1, 2, 3]).expect("slice is not empty");
        assert_eq!(*values.first(), 1);
    }
}
