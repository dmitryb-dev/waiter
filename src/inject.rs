#[macro_export]
macro_rules! inject {
    ($comp:path: $($profile:path),*) => {
        {
            let parsed_profile = waiter_di::parse_profile();
            println!("Using profile: {}", parsed_profile);
            $(
                if profile_name::<$profile>().eq(&parsed_profile) {
                    waiter_di::Provider::<$comp>::get(&mut Container::<$profile>::new())
                } else
            )*
            { waiter_di::Provider::<$comp>::get(&mut Container::<waiter_di::profiles::Default>::new()) }
        }
    }
}