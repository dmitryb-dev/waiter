extern crate waiter;
extern crate waiter_core;

use waiter::*;
use waiter::Provider;

trait Interface {
    fn int(&self);
}

trait Interface2 {
    fn int2(&self);
}

#[derive(Debug)]
struct Dependency;
impl waiter::Component for Dependency {
    fn __waiter_create<T>(container: &mut Container<T>) -> Self {
        return Dependency;
    }
}
impl<T> Provider<Dependency> for Container<T> {
    fn get_ref(&mut self) -> &Dependency {
        let type_id = std::any::TypeId::of::<Dependency>();
        if !self.components.contains_key(&type_id) {
            let component = Box::new(Dependency::__waiter_create(self));
            self.components.insert(type_id, component);
        }
        let any = self.components.get(&type_id).unwrap();
        return any.downcast_ref::<Dependency>().unwrap();
    }
}

#[derive(Debug)]
struct Comp<'a> {
    dependency: &'a Dependency
}
impl<'a> waiter::Component for Comp<'a> {
    fn __waiter_create<T>(container: &mut Container<T>) -> Self {
        let dep_0 = unsafe {
            (Provider::<Dependency>::get_ref(container) as *const Dependency)
                .as_ref()
                .unwrap()
        };
        return Comp { dependency: dep_0 };
    }
}
impl<'a, T> Provider<Comp<'a>> for Container<T> {
    fn get_ref(&mut self) -> &Comp<'a> {
        let type_id = std::any::TypeId::of::<Comp>();
        if !self.components.contains_key(&type_id) {
            let component = Box::new(Comp::__waiter_create(self));
            self.components.insert(type_id, component);
        }
        let any = self.components.get(&type_id).unwrap();
        return any.downcast_ref::<Comp>().unwrap();
    }
}

impl<'a> Comp<'a> {
    fn int0(&self) {
        println!("i0 {:?}", self);
    }
}

impl<'a> Interface for Comp<'a> {
    fn int(&self) {
        println!("i1 {:?}", self);
    }
}
impl<T> Provider<dyn Interface> for Container<T> {
    fn get_ref(&mut self) -> &(dyn Interface + 'static) {
        return Provider::<Comp>::get_ref(self);
    }
}

impl<'a> Interface2 for Comp<'a> {
    fn int2(&self) {
        println!("i2 {:?}", self);
    }
}
impl Provider<dyn Interface2> for Container<profiles::Dev> {
    fn get_ref(&mut self) -> &(dyn Interface2 + 'static) {
        return Provider::<Comp>::get_ref(self);
    }
}

fn main() {
    let mut container = Container::<profiles::Dev>::new();

    let comp = Provider::<Comp>::get_ref(&mut container);
    comp.int0();
    comp.int();
    comp.int2();

    let comp = Provider::<dyn Interface>::get_ref(&mut container);
    comp.int();

    let comp = Provider::<dyn Interface2>::get_ref(&mut container);
    comp.int2();
}