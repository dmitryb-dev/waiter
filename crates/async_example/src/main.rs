use waiter_di::*;
use async_trait::async_trait;

#[async_trait]
trait AsyncInterface {
    async fn async_int(&self);
}

#[component]
struct Comp {}

#[async_trait]
#[provides]
impl AsyncInterface for Comp {
    async fn async_int(&self) {
        println!("Async Interface");
    }
}

#[async_std::main]
async fn main() {
    let mut container = Container::<profiles::Default>::new();

    let comp = Provider::<Comp>::get_ref(&mut container);
    comp.async_int().await;

    let comp = Provider::<dyn AsyncInterface>::get_ref(&mut container);
    comp.async_int().await;
}
