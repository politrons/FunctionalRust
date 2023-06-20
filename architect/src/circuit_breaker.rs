use std::time::{Duration, Instant};
use crate::circuit_breaker::State::{Close, Open};

const MAX_ERROR_ALLOWED: u32 = 3;
const RESET_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(PartialEq, Debug)]
enum State {
    Open,
    Close,
    HalfOpen,
}

struct CircuitBreaker {
    state: State,
    errors: u32,
    last_failure_time: Option<Instant>,
}

impl CircuitBreaker {
    fn new() -> Self {
        CircuitBreaker {
            state: Close,
            errors: 0,
            last_failure_time: None,
        }
    }

    fn watch<F: FnOnce() -> Result<R, String>, R>(&mut self, func: F) -> Result<R, String> {
        match self.check_state().state {
            Open => Err("Circuit is open".to_string()),
            Close | State::HalfOpen => self.run_execution(func)
        }
    }

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

    fn mark_as_failure(&mut self) {
        self.errors += 1;
        if self.errors >= MAX_ERROR_ALLOWED {
            self.state = Open;
            self.last_failure_time = Some(Instant::now());
        }
    }

    fn check_state(&mut self) -> &Self {
        match self.last_failure_time {
            Some(time) if (time.elapsed() < RESET_TIMEOUT) => {
                self.state = Close;
                self
            }
            None => self,
            _ => self,
        }
    }

    fn reset(&mut self) {
        self.errors = 0;
        self.last_failure_time = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protect_success() {
        let mut circuit_breaker = CircuitBreaker::new();

        let result = circuit_breaker.watch(|| {
            // Simulate a successful function call
            Ok("Success")
        });

        assert_eq!(result, Ok("Success"));
    }

    #[test]
    fn test_protect_failure() {
        let mut circuit_breaker = CircuitBreaker::new();

        let result: Result<String, String> = circuit_breaker.watch(|| {
            // Simulate a function call that fails
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
            // Simulate a function call when the circuit is open
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
        circuit_breaker.last_failure_time = Some(Instant::now() - RESET_TIMEOUT);

        let result = circuit_breaker.watch(|| {
            // Simulate a function call after the reset timeout
            Ok("Success")
        });

        assert_eq!(result, Ok("Success"));
        assert_eq!(circuit_breaker.errors, 0);
        assert_eq!(circuit_breaker.state, Close);
    }
}
