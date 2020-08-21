#[macro_export]
macro_rules! inject {
    ($comp:path: $($profile:path),*) => {
        {
            let parsed_profile = waiter::parse_profile();
            println!("Using profile: {}", parsed_profile);
            $(
                if profile_name::<$profile>().eq(&parsed_profile) {
                    waiter::Provider::<$comp>::get(&mut Container::<$profile>::new())
                } else
            )*
            { waiter::Provider::<$comp>::get(&mut Container::<waiter::profiles::Default>::new()) }
        }
    }
}