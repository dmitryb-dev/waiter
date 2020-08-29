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