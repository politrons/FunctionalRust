use std::borrow::Cow;
use std::sync::atomic::{AtomicUsize, Ordering};

/**
Performance in Rust often starts with API shape: borrow data when possible,
allocate only when required, and make synchronization costs explicit.
*/
pub fn run() {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);

    let label = normalize_label("rust-label");
    let total = sum_without_allocation(&[1, 2, 3]);
    let id = next_id(&COUNTER);
    let header = split_header("content-type: json");

    println!(
        "Label:{:?} Total:{} Id:{} Header:{:?}",
        label, total, id, header
    );
}

/**
Cow returns a borrowed value when the input already fits the target form.
It allocates only when normalization is required.
*/
fn normalize_label(input: &str) -> Cow<'_, str> {
    let trimmed = input.trim();
    let already_normalized = trimmed == input
        && input
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');

    if already_normalized {
        Cow::Borrowed(input)
    } else {
        Cow::Owned(trimmed.to_ascii_lowercase().replace(' ', "-"))
    }
}

/**
Returning string slices keeps the parsed values tied to the original input and avoids allocation.
*/
fn split_header(line: &str) -> Option<(&str, &str)> {
    let (name, value) = line.split_once(':')?;
    Some((name.trim(), value.trim()))
}

fn sum_without_allocation(values: &[u64]) -> u64 {
    values.iter().copied().sum()
}

/**
Relaxed ordering is enough for a unique counter when no cross-thread happens-before
relationship is required.
*/
fn next_id(counter: &AtomicUsize) -> usize {
    counter.fetch_add(1, Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn avoids_allocation_when_label_is_already_normalized() {
        assert!(matches!(normalize_label("rust-label"), Cow::Borrowed(_)));
        assert!(matches!(normalize_label("Rust Label"), Cow::Owned(_)));
    }

    #[test]
    fn parses_header_without_allocating() {
        assert_eq!(
            split_header("content-type: json"),
            Some(("content-type", "json"))
        );
    }
}
