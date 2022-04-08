#[macro_export]
macro_rules! inject {
    ($comp:path: $($profile:path),*) => {
        {
            $(
                if profile_name::<$profile>().eq(&waiter_di::APP_PROFILE.as_str()) {
                    waiter_di::Provider::<$comp>::create(&mut waiter_di::Container::<$profile>::new())
                } else
            )*
            { waiter_di::Provider::<$comp>::create(&mut waiter_di::Container::<waiter_di::profiles::Default>::new()) }
        }
    }
}

#[macro_export]
macro_rules! wrap {
    ($wrapped_type:path as $wrapper_name:ident) => {
        pub struct $wrapper_name($wrapped_type);
        impl Deref for $wrapper_name {
            type Target = $wrapped_type;
            fn deref(&self) -> &Self::Target {
                return &self.0;
            }
        }
    };
}
