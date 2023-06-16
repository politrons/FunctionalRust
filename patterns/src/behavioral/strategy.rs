/// Strategy pattern allows you to create a Service like an abstraction of the logic you want to run
/// in your program. Then you can decide dynamically under some circumstances, to change this strategy
/// for another, changing the behavior of your program.

/// Contract for all Strategies implementations
pub trait LangStrategy {
    fn speak(&self);
}

/// Strategy Data type
pub struct EnglishStrategy;

/// Implementation of Strategy, implementation here for any business logic.
impl LangStrategy for EnglishStrategy {
    fn speak(&self) {
        println!("Hello there")
    }
}

/// Strategy Data type
pub struct SpanishStrategy;

/// Implementation of Strategy, implementation here for any business logic.
impl LangStrategy for SpanishStrategy {
    fn speak(&self) {
        println!("Hola amigo")
    }
}

/// This service it has a dependency with a Strategy that it will be injected once is created.
pub struct LangService {
    strategy: Box<dyn LangStrategy>,
}

/// Implementation of Service that it has a dependency injection of a Strategy.
/// [set_strategy] we inject a Strategy which it change the behavior of the service.
/// [speak] it will invoke the same function of the strategy, so then it can change the behavior
/// depending of the strategy injected.
impl LangService {
    pub fn new(strategy: Box<dyn LangStrategy>) -> Self {
        LangService { strategy }
    }

    pub fn set_strategy(&mut self, strategy: Box<dyn LangStrategy>) {
        self.strategy = strategy;
    }

    pub fn speak(&self) {
        self.strategy.speak();
    }
}

#[cfg(test)]
mod tests {
    use crate::behavioral::memento::{Event, EventSourcing};
    use crate::behavioral::strategy::{EnglishStrategy, LangService, SpanishStrategy};

    #[test]
    fn strategy_pattern() {
        let english_strategy = Box::new(EnglishStrategy);
        let spanish_strategy = Box::new(SpanishStrategy);
        let mut sorter = LangService::new(english_strategy);
        sorter.speak();
        sorter.set_strategy(spanish_strategy);
        sorter.speak();
    }
}
