extern crate config;
extern crate serde;
extern crate waiter_di;

use std::rc::Rc;

use config::Config;
use serde::Deserialize;

use waiter_di::*;

trait Interface {
    fn int(&self);
}

trait Interface2 {
    fn int2(&self);
}

struct Dependency {
    map: HashMap,
}

impl Dependency {
    fn dep(&self) {
        println!("Dep {:?}", self.map);
    }
}

#[provides]
fn create_dependency(map: HashMap) -> Dependency {
    println!("dep factory");
    Dependency { map }
}

#[derive(Debug)]
struct HashMap(std::collections::HashMap<i32, i32>);

#[provides]
fn create_external_type_dependency() -> HashMap {
    HashMap(std::collections::HashMap::<i32, i32>::new())
}

#[derive(Debug, Deserialize)]
struct ConfigObject {
    i32_prop: i32,
}

#[component]
struct Comp {
    dependency: Dependency,
    dependency_rc: Rc<Dependency>,
    dependency_box: Box<Dependency>,
    dependency_def: Deferred<Dependency>,
    dependency_def_rc: Deferred<Rc<Dependency>>,
    dependency_def_box: Deferred<Box<Dependency>>,
    cyclic: Deferred<Wrc<dyn Interface>>,
    config: Config,
    #[prop("int_v")] int_prop: usize,
    #[prop("float_v" = 3.14)] float_prop: f32,
    str_prop: String,
    bool_prop: Option<bool>,
    #[prop] config_object: ConfigObject,
}

impl Comp {
    fn comp(&self) {
        self.dependency.dep();
        self.dependency_rc.dep();
        self.dependency_box.dep();
        self.dependency_def.dep();
        self.dependency_def_rc.dep();
        self.dependency_def_box.dep();
        self.config.get_string("prop").unwrap();
        println!("Comp, {}, {}, {}, {:?}, {}", self.int_prop, self.float_prop, self.str_prop,
                 self.bool_prop, self.config_object.i32_prop);
    }
}

#[provides]
impl Interface for Comp {
    fn int(&self) {
        println!("Interface");
    }
}

#[provides(profiles::Dev)]
impl Interface2 for Comp {
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


    println!("Using profile: {}", APP_PROFILE.as_str());
    let comp = inject!(Comp: profiles::Default, profiles::Dev);
    comp.comp();
}