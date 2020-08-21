use std::cell::Cell;
use deferred::Value::{WaitingForValue, Initialized};
use std::ops::Deref;

pub struct Deferred<T> {
    value: Cell<Value<T>>
}

pub enum Value<T> {
    Initialized(T),
    WaitingForValue
}

impl<T> Deferred<T> {
    pub fn new() -> Self {
        Self { value: Cell::new(WaitingForValue) }
    }
    pub fn init(&self, value: T) {
        self.value.set(Initialized(value));
    }
}

impl<T> Deref for Deferred<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            if let Initialized(value) = &*self.value.as_ptr() {
                value
            } else {
                panic!("Deferred value must be initialized before the first usage")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use deferred::*;

    #[test]
    fn deref_after_init() {
        let deferred = Deferred::<&str>::new();
        deferred.init("Initialized");
        assert_eq!("Initialized", *deferred);
    }

    #[test]
    #[should_panic]
    fn deref_before_init() {
        let deferred = Deferred::<&str>::new();
        assert_eq!("Initialized", *deferred);
    }
}