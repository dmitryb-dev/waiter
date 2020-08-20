use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::rc::Rc;
use std::marker::PhantomData;

pub mod profiles {
    pub struct Default;
    pub struct Dev;
}

pub trait Component {
    fn __waiter_create<P>(container: &mut Container<P>) -> Self;
    fn __waiter_inject_deferred<P>(container: &mut Container<P>, component: Rc<Self>);
}

pub trait Provider<T: ?Sized> {
    fn get(&mut self) -> Rc<T>;
    fn get_ref(&mut self) -> &T;
    fn create(&mut self) -> Box<T>;
}

pub struct Container<P> {
    profile: PhantomData<P>,
    pub components: HashMap<TypeId, Rc<dyn Any>>
}

impl<T> Container<T> {
    pub fn new() -> Container<T> {
        Container {
            profile: PhantomData::<T>,
            components: HashMap::new()
        }
    }
}