use std::time::{Duration, Instant};
use crate::circuit_breaker::State::{Close, HalfOpen, Open};

/// Circuit Breaker pattern is to prevent cascading failures and provide a fallback mechanism
/// when a service is unavailable or under heavy load. Instead of continuously attempting
/// to call the remote service and potentially causing performance degradation or timeouts.

/// Constants to set the max errors allowed before we change the state of the Circuit breaker,
/// and the time to wait before we allow to get through a request.
const MAX_ERROR_ALLOWED: u32 = 3;
const RESET_TIMEOUT: Duration = Duration::from_secs(10);

/// Circuit breaker states
#[derive(PartialEq, Debug)]
enum State {
    Open,
    Close,
    HalfOpen,
}

/// Circuit breaker data type, that hold the state of the CB,the number of errors, and thr last failure.
/// We will use the errors and last_failure to change the state from [Close] to [Open], and after some failure time
/// [Half-open]
struct CircuitBreaker {
    state: State,
    errors: u32,
    maybe_last_failure_time: Option<Instant>,
}

/// Implementation of Circuit Breaker.
impl CircuitBreaker {
    /// Create the instance with [Close] as the default state/
    fn new() -> Self {
        CircuitBreaker {
            state: Close,
            errors: 0,
            maybe_last_failure_time: None,
        }
    }

    /// Function responsible to check state and run the execution of the program.
    /// Check the current state of the Circuit breaker is Close or Half-Open to run the execution of the program.
    /// Otherwise if is Open we just return a [Result] of [Err]
    fn watch<F: FnOnce() -> Result<R, String>, R>(&mut self, func: F) -> Result<R, String> {
        match self.check_state().state {
            Open => Err("Circuit is open".to_string()),
            Close | State::HalfOpen => self.run_execution(func)
        }
    }

    /// Run the program function of [Result] return type.
    /// In case is a failure, invoke [mark_as_failure] to change state of CircuitBreaker
    /// In case is success, if the state was not [Close], we [reset] the Circuit breaker state.
    fn run_execution<F: FnOnce() -> Result<R, String>, R>(&mut self, func: F) -> Result<R, String> {
        match func() {
            Ok(result) if (self.errors > 0) => {
                self.reset();
                Ok(result)
            }
            Ok(result) => {
                Ok(result)
            }
            Err(t) => {
                self.mark_as_failure();
                Err(format!("Error occurred. Caused by {}", t))
            }
        }
    }

    /// Increment the number of errors, and set the last_failure time.
    /// If we reach the max number of errors allowed, we change the state from [Close] to [Open]
    /// And we set the [last_failure_time] to start counting for how long we need to wait until we
    /// allow to pass one execution in [Half-open] state
    fn mark_as_failure(&mut self) {
        self.errors += 1;
        if self.errors >= MAX_ERROR_ALLOWED {
            self.state = Open;
            self.maybe_last_failure_time = Some(Instant::now());
        }
    }

    /// Function responsible to check if the state of the Circuit breaker has change.
    /// In case is Open only, we check if the time of the [last_failure_time] wait enough time,
    /// to be consider for change the state into [Half-Open] state
    fn check_state(&mut self) -> &Self {
        match self.state {
            Open  => match self.maybe_last_failure_time {
                Some(last_failure_time) if (last_failure_time.elapsed() < RESET_TIMEOUT) => {
                    self.state = HalfOpen;
                    self
                }
                None => self,
                _ => self,
            }
            _ => self,
        }
    }

    /// Set to 0 number of [errors] and we set to None the [last_failure_time]
    fn reset(&mut self) {
        self.errors = 0;
        self.maybe_last_failure_time = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protect_success() {
        let mut circuit_breaker = CircuitBreaker::new();

        let result = circuit_breaker.watch(|| {
            Ok("Success")
        });

        assert_eq!(result, Ok("Success"));
    }

    #[test]
    fn test_protect_failure() {
        let mut circuit_breaker = CircuitBreaker::new();

        let result: Result<String, String> = circuit_breaker.watch(|| {
            Err("Something went wrong".to_string())
        });

        assert_eq!(result, Err("Error occurred. Caused by Something went wrong".to_string()));
        assert_eq!(circuit_breaker.errors, 1);
        assert_eq!(circuit_breaker.state, Close);
    }

    #[test]
    fn test_protect_open_circuit() {
        let mut circuit_breaker = CircuitBreaker::new();
        circuit_breaker.errors = MAX_ERROR_ALLOWED;

        let result: Result<String, String> = circuit_breaker.watch(|| {
            Err("Something went wrong".to_string())
        });

        assert_eq!(result, Err("Error occurred. Caused by Something went wrong".to_string()));
        assert_eq!(circuit_breaker.errors, MAX_ERROR_ALLOWED + 1);
        assert_eq!(circuit_breaker.state, Open);
    }

    #[test]
    fn test_protect_reset_circuit() {
        let mut circuit_breaker = CircuitBreaker::new();
        circuit_breaker.errors = MAX_ERROR_ALLOWED;
        circuit_breaker.maybe_last_failure_time = Some(Instant::now() - RESET_TIMEOUT);

        let result = circuit_breaker.watch(|| {
            // Simulate a function call after the reset timeout
            Ok(1981)
        });

        assert_eq!(result, Ok(1981));
        assert_eq!(circuit_breaker.errors, 0);
        assert_eq!(circuit_breaker.state, Close);
    }
}
