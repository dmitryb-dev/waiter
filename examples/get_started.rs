extern crate waiter_di;
extern crate config;
extern crate serde;

use waiter_di::*;
use std::rc::Rc;

trait Interface {
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
    interface_impl: InterfaceImpl,
    interface: Rc<dyn Interface>,
    #[prop("i32_prop")] prop: i32
}

fn main() {
    let mut container = Container::<profiles::Default>::new();

    let component = Provider::<SomeComp>::get(&mut container);

    component.interface.demo();
}