use std::sync::OnceLock;

/**
Idiomatic Rust prefers safe standard types and explicit state machines over
global mutable singletons or class hierarchies.
*/

#[derive(Debug)]
struct AppConfig {
    service_name: &'static str,
}

static APP_CONFIG: OnceLock<AppConfig> = OnceLock::new();

/**
OnceLock is the safe standard-library replacement for most unsafe singleton examples.
The value is initialized once and then exposed as an immutable 'static reference.
*/
fn app_config() -> &'static AppConfig {
    APP_CONFIG.get_or_init(|| AppConfig {
        service_name: "functional-rust",
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ConnectionState {
    Disconnected,
    Connecting { attempt: u8 },
    Connected { session_id: String },
    Failed { reason: String },
}

impl ConnectionState {
    fn connect(self) -> Self {
        match self {
            ConnectionState::Disconnected => ConnectionState::Connecting { attempt: 1 },
            ConnectionState::Connecting { attempt } => ConnectionState::Connecting {
                attempt: attempt.saturating_add(1),
            },
            state => state,
        }
    }

    fn mark_connected(self, session_id: impl Into<String>) -> Self {
        match self {
            ConnectionState::Connecting { .. } => ConnectionState::Connected {
                session_id: session_id.into(),
            },
            state => state,
        }
    }

    fn fail(self, reason: impl Into<String>) -> Self {
        match self {
            ConnectionState::Connected { .. } => self,
            _ => ConnectionState::Failed {
                reason: reason.into(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_machine_makes_invalid_transitions_explicit() {
        let state = ConnectionState::Disconnected
            .connect()
            .mark_connected("session-1");

        let failed = ConnectionState::Disconnected.fail("network unavailable");

        app_config();

        println!(
            "Config:{} State:{:?} Failed:{:?}",
            app_config().service_name,
            state,
            failed
        );

        assert_eq!(
            state,
            ConnectionState::Connected {
                session_id: "session-1".to_string()
            }
        );


    }
}
