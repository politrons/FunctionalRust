/**
Cargo configuration matters as projects grow: workspaces, feature flags,
release profiles and MSRV become API and delivery contracts.
*/
pub fn run() {
    println!(
        "Cargo workspace example has {} lines and feature example has {} lines",
        WORKSPACE_TOML_EXAMPLE.lines().count(),
        PACKAGE_TOML_FEATURES_EXAMPLE.lines().count()
    );
}

/**
Use a workspace when several crates belong to the same product. resolver = "3"
is the edition-2024 resolver and avoids feature unification surprises.
*/
const WORKSPACE_TOML_EXAMPLE: &str = r#"
[workspace]
members = ["crates/core", "crates/cli"]
resolver = "3"

[workspace.package]
edition = "2024"
rust-version = "1.85"
license = "MIT"

[profile.release]
lto = "thin"
codegen-units = 1
"#;

/**
Feature flags let a crate expose optional capabilities without forcing every user
to compile every dependency or unstable integration.
*/
const PACKAGE_TOML_FEATURES_EXAMPLE: &str = r#"
[features]
default = ["std"]
std = []
experimental = []
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_example_contains_resolver() {
        assert!(WORKSPACE_TOML_EXAMPLE.contains("resolver = \"3\""));
        assert!(PACKAGE_TOML_FEATURES_EXAMPLE.contains("default"));
    }
}
