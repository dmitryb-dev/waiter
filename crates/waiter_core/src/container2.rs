use std::collections::HashMap;
use std::any::{TypeId, Any};


struct Factories {
    components: HashMap<TypeId, Box<dyn Any>>
}

pub trait Factory<T: ?Sized> {
    fn get(&mut self) -> &T;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Deref;

    trait Interface {
        fn call(&self) -> &str;
    }
    trait Interface2 {
        fn call2(&self) -> &str;
    }

    #[derive(Debug)]
    struct Dep {}
    impl Factory<Dep> for Factories {
        fn get(&mut self) -> &Dep {
            if !self.components.contains_key(&TypeId::of::<Dep>()) {
                self.components.insert(TypeId::of::<Dep>(), Box::new(Dep {}));
            }
            let any = self.components.get(&TypeId::of::<Dep>())
                .unwrap()
                .deref();

            return any
                .downcast_ref::<Dep>()
                .unwrap();
        }
    }

    #[derive(Debug)]
    struct Comp<'a> {
        dep: &'a Dep
    }
    impl<'a> Comp<'a> {
        fn call(&self) -> &str { return "comp"; }
    }
    impl<'a> Interface for Comp<'a> {
        fn call(&self) -> &str { return "interface"; }
    }
    impl<'a> Interface2 for Comp<'a> {
        fn call2(&self) -> &str {
            println!("{:?}", self);
            return "interface 2"; }
    }

    impl<'a> Factory<Comp<'a>> for Factories {
        fn get(&mut self) -> &Comp<'a> {
            if !self.components.contains_key(&TypeId::of::<Comp>()) {
                let dep = unsafe { (<Factories as Factory<Dep>>::get(self) as *const Dep).as_ref().unwrap() };

                self.components.insert(TypeId::of::<Comp>(), Box::new(Comp { dep }));

            }
            let any = self.components.get(&TypeId::of::<Comp>())
                .unwrap()
                .deref();

            return any
                .downcast_ref::<Comp>()
                .unwrap();
        }
    }
    impl Factory<dyn Interface> for Factories {
        fn get(&mut self) -> &(dyn Interface + 'static) {
            return <Factories as Factory<Comp>>::get(self);
        }
    }
    impl Factory<dyn Interface2> for Factories {
        fn get(&mut self) -> &(dyn Interface2 + 'static) {
            return <Factories as Factory<Comp>>::get(self);
        }
    }

    #[test]
    fn test() {
        let mut f = Factories { components: HashMap::new() };
        let comp = <Factories as Factory<Interface2>>::get(&mut f);
        let comp = Factory::<Interface2>::get(&mut f);
        println!("{}", comp.call2());
    }
}