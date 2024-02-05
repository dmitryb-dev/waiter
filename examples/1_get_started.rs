extern crate config;
extern crate serde;
extern crate waiter_di;

use std::rc::Rc;

use waiter_di::*;

trait Interface: Send {
    fn demo(&self);
}

#[component]
struct InterfaceImpl {}

#[provides]
impl Interface for InterfaceImpl {
    fn demo(&self) {
        println!("Dependency");
    }
}

#[component]
struct SomeComp {
    #[prop("i32_prop")] prop: i32,
    interface: Rc<dyn Interface>,
}

fn main() {
    let mut container = Container::<profiles::Default>::new();

    let component = Provider::<SomeComp>::get(&mut container);

    component.interface.demo();
}