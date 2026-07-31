/**
Robust testing is not just checking one happy path. It means validating edge cases,
using table-driven tests, and adding property-style invariants when possible.
*/
pub fn run() {
    println!(
        "Normalized user: {:?}",
        normalize_username(" Politrons_1981 ")
    );
}

fn normalize_username(raw: &str) -> Option<String> {
    let normalized = raw.trim().to_ascii_lowercase();
    let valid = !normalized.is_empty()
        && normalized
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_');

    valid.then_some(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_driven_validation() {
        let cases = [
            (" Politrons ", Some("politrons")),
            ("rust_user", Some("rust_user")),
            ("", None),
            ("not valid", None),
        ];

        for (input, expected) in cases {
            assert_eq!(normalize_username(input).as_deref(), expected);
        }
    }

    #[test]
    fn property_style_idempotence_without_extra_crates() {
        for input in ["Alice", "BOB_42", " rustacean "] {
            let once = normalize_username(input);
            let twice = once.as_deref().and_then(normalize_username);

            assert_eq!(once, twice);
        }
    }
}
