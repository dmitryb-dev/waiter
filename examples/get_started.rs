extern crate waiter;
extern crate waiter_core;

use waiter::*;
use waiter::Provider;
use waiter_core::deferred::Deferred;
use std::rc::Rc;

trait Interface {
    fn int(&self);
}

trait Interface2 {
    fn int2(&self);
}

#[component]
struct Dependency;

impl Dependency {
    fn dep(&self) {
        println!("dep");
    }
}

#[component]
struct Comp<'a> {
    dependency_rc: Rc<Dependency>,
    dependency_ref: &'a Dependency,
    dependency_box: Box<Dependency>,
    dependency_def_rc: Deferred<Rc<Dependency>>,
    dependency_def_box: Deferred<Box<Dependency>>,
    int_prop: i64,
    float_prop: f64,
    str_prop: String,
    bool_prop: bool
}

impl<'a>  Comp<'a>  {
    fn int0(&self) {
        self.dependency_rc.dep();
        self.dependency_ref.dep();
        self.dependency_box.dep();
        self.dependency_def_rc.dep();
        self.dependency_def_box.dep();
        println!("comp int0, {}, {}, {}, {}", self.int_prop, self.float_prop, self.str_prop, self.bool_prop);
    }
}

#[provides]
impl<'a>  Interface for Comp<'a>  {
    fn int(&self) {
        println!("interface int");
    }
}

#[provides(profiles::Dev)]
impl<'a>  Interface2 for Comp<'a>  {
    fn int2(&self) {
        println!("interface int2");
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



    let mut container = Container::<profiles::Dev>::new();
    let comp = Provider::<dyn Interface>::get_ref(&mut container);
    comp.int();

    let comp = Provider::<dyn Interface2>::get_ref(&mut container);
    comp.int2();
}