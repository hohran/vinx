use crate::variable::{Stack, Variable, VariableType};
use crate::translator::ast;

// TODO: add milliseconds
#[derive(Debug, PartialEq, Eq)]
pub enum TimeUnit {
    Frame,
}

/// Time accumulator, which is supposed to activate at certain trigger time.
/// Use `step` to count up time and `activate` to activate it.
/// If it is a `onetime`, it will get disabled upon its first activation.
#[derive(Debug)]
pub struct Trigger {
    counter: usize,
    trigger_time: Variable,
    unit: TimeUnit,
    onetime: bool,
    enabled: bool,
}

impl Trigger {
    pub fn new(trigger_time: Variable, unit: TimeUnit, onetime: bool) -> Self {
        assert!(trigger_time.get_type() == VariableType::Int);
        Self { trigger_time, counter: 0, unit, onetime, enabled: true }
    }

    // Create Trigger from its ast representation
    pub fn from(trigger: ast::Trigger, stack: &Stack) -> Self {
        let time = match trigger.time {
            ast::Time::Variable(name) => {
                let Some (var) = stack.get_variable(&name) else {
                    panic!("error: no such variable {name}") // TODO: friendlify
                };
                Variable::Named(name, var.get_type())
            }
            ast::Time::Number(n) => {
                Variable::Static(crate::variable::VariableValue::Int(n as i32)) // TODO: make i64
            }
        };
        let unit = match trigger.unit {
            ast::Unit::Frame => TimeUnit::Frame,
            ast::Unit::Second => panic!("error: time unit `second` not implemented"),
            ast::Unit::Millisecond => panic!("error: time unit `millisecond` not implemented"),
            // x => panic!("error: unexpected time unit `{x:?}`"),
        };
        Self { counter: 0, trigger_time: time, unit, onetime: trigger.onetime, enabled: trigger.active }
    }

    pub fn clear(&mut self) {
        self.counter = 0;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Increase the counter.
    pub fn step(&mut self) {
        if !self.enabled { return; }
        match &self.unit {
            TimeUnit::Frame => self.counter += 1,
        }
    }

    /// If counted up to the trigger time, return `true` and modify the counter.
    pub fn activate(&mut self, stack: &Stack) -> bool {
        if !self.enabled { return false; }
        let t = self.trigger_time.get_value(stack).into_int() as usize;
        if self.counter < t { return false; }
        if self.onetime { self.enabled = false; } // disable onetime triggers
        match &self.unit {
            TimeUnit::Frame => self.counter = 0,
        }
        true
    }

    /// Increase counted time on `step`.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Stop counting time.
    /// The currently accumulated time will not be reset.
    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step_n_times(acc: &mut Trigger, n: usize) {
        for _ in 0..n {
            acc.step();
        }
    }

    fn get_stack() -> Stack {
        let mut s = Stack::new();
        s.add_variable("t".to_string(), crate::variable::VariableValue::Int(5));
        s
    }

    #[test]
    fn test_activate() {
        let mut t = Trigger::new(Variable::new("t", VariableType::Int), TimeUnit::Frame, false);
        let mut s = get_stack();
        assert!(!t.activate(&s));
        step_n_times(&mut t, 4);
        assert!(!t.activate(&s));
        step_n_times(&mut t, 1);
        assert!(t.activate(&s));
        assert!(!t.activate(&s));
        step_n_times(&mut t, 20);
        assert!(t.activate(&s));
        assert!(!t.activate(&s));
        // changing trigger time
        step_n_times(&mut t, 4);
        assert!(!t.activate(&s));
        s.update_variable("t", crate::variable::VariableValue::Int(4));
        assert!(t.activate(&s));
        // onetime
        let mut t = Trigger::new(Variable::new("t", VariableType::Int), TimeUnit::Frame, true);
        step_n_times(&mut t, 15);
        assert!(t.activate(&s));
        step_n_times(&mut t, 15);
        assert!(!t.activate(&s));
    }

    #[test]
    fn test_clear() {
        let mut t = Trigger::new(Variable::new("t", VariableType::Int), TimeUnit::Frame, false);
        step_n_times(&mut t, 10);
        t.clear();
        assert!(!t.activate(&get_stack()));
    }

    #[test]
    fn test_enable_disable() {
        let mut t = Trigger::new(Variable::new("t", VariableType::Int), TimeUnit::Frame, false);
        let s = get_stack();
        t.disable();              // <- Disabled
        step_n_times(&mut t, 10); // Steps will not be registered
        assert!(!t.activate(&s));
        t.enable();               // <- Enabled
        assert!(!t.activate(&s));
        step_n_times(&mut t, 10); // These steps will be registered
        t.disable();
        assert!(!t.activate(&s)); // There is enough time accumulated, but it will not activate
                                  // when disabled
        t.enable();
        assert!(t.activate(&s));
    }
}
