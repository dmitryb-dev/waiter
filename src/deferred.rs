use crate::deferred::DeferredValue::{WaitingForValue, Initialized};
use std::ops::Deref;
use std::sync::Mutex;

pub struct Deferred<T> {
    value: Mutex<DeferredValue<T>>
}

pub enum DeferredValue<T> {
    Initialized(T),
    WaitingForValue
}

impl<T> Deferred<T> {
    pub fn new() -> Self {
        Self { value: Mutex::new(WaitingForValue) }
    }
    pub fn init(&self, value: T) {
        *self.value.lock().unwrap() = Initialized(value);
    }
}

impl<T> Deref for Deferred<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            if let Initialized(value) = &*(self.value.lock().unwrap().deref() as *const DeferredValue<T>) {
                value
            } else {
                panic!("Deferred value must be initialized before the first usage")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Deferred;

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