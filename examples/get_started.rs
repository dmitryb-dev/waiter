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

#[component]
#[derive(Debug)]
struct Dependency;

#[component]
#[derive(Debug)]
struct Comp<'a> {
    dependency: &'a Dependency
}

impl<'a> Comp<'a> {
    fn int0(&self) {
        println!("i0 {:?}", self);
    }
}

#[provides]
impl<'a> Interface for Comp<'a> {
    fn int(&self) {
        println!("i1 {:?}", self);
    }
}

#[provides]
impl<'a> Interface2 for Comp<'a> {
    fn int2(&self) {
        println!("i2 {:?}", self);
    }
}


fn main() {
    let mut container = Container::<profiles::Default>::new();

    let comp = Provider::<Comp>::get_ref(&mut container);
    comp.int0();
    comp.int();
    comp.int2();

    let comp = Provider::<dyn Interface>::get_ref(&mut container);
    comp.int();

    let comp = Provider::<dyn Interface2>::get_ref(&mut container);
    comp.int2();
}