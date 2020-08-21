extern crate waiter_di;
extern crate config;

use waiter_di::*;
use std::rc::Rc;
use config::Config;

trait Interface {
    fn int(&self);
}

trait Interface2 {
    fn int2(&self);
}

struct Dependency;

impl Dependency {
    fn dep(&self) {
        println!("Dep");
    }
}

#[provides]
fn create_dependency<P>(_container: &mut Container<P>) -> Dependency {
    Dependency
}

#[component]
struct Comp<'a> {
    dependency_rc: Rc<Dependency>,
    dependency_ref: &'a Dependency,
    dependency_box: Box<Dependency>,
    dependency_def_rc: Deferred<Rc<Dependency>>,
    dependency_def_box: Deferred<Box<Dependency>>,
    config: Config,
    #[prop("int")] int_prop: usize,
    #[prop("float")] float_prop: f32,
    str_prop: String,
    bool_prop: bool
}

impl<'a>  Comp<'a>  {
    fn comp(&self) {
        self.dependency_rc.dep();
        self.dependency_ref.dep();
        self.dependency_box.dep();
        self.dependency_def_rc.dep();
        self.dependency_def_box.dep();
        self.config.get_str("prop").unwrap();
        println!("Comp, {}, {}, {}, {}", self.int_prop, self.float_prop, self.str_prop, self.bool_prop);
    }
}

#[provides]
impl<'a>  Interface for Comp<'a>  {
    fn int(&self) {
        println!("Interface");
    }
}

#[provides(profiles::Dev)]
impl<'a>  Interface2 for Comp<'a>  {
    fn int2(&self) {
        println!("Interface 2");
    }
}


fn main() {
    let mut container = Container::<profiles::Default>::new();

    let comp = Provider::<Comp>::get_ref(&mut container);
    comp.comp();
    comp.int();
    comp.int2();

    let comp = Provider::<dyn Interface>::get_ref(&mut container);
    comp.int();



    let mut container = Container::<profiles::Dev>::new();
    let comp = Provider::<dyn Interface>::get_ref(&mut container);
    comp.int();

    let comp = Provider::<dyn Interface2>::get_ref(&mut container);
    comp.int2();


    let comp = inject!(Comp: profiles::Default, profiles::Dev);
    comp.comp();
}