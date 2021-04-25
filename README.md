# Dependency injection for Rust

How to use:

`Cargo.toml`:
```toml
waiter_di = "1.6.5"
```
`lib.rs` or any other file, that uses library:
```rust
use waiter_di::*;
```

See 
[examples/1_get_started.rs](https://github.com/dmitryb-dev/waiter/blob/master/examples/1_get_started.rs) 
for minimal example of usage.

See 
[examples/2_modules.rs](https://github.com/dmitryb-dev/waiter/blob/master/examples/2_modules.rs) 
for example with modules and constructors.

See 
[examples/3_inject_options_list.rs](https://github.com/dmitryb-dev/waiter/blob/master/examples/3_inject_options_list.rs) 
for the demo of all available injection options.

## How to use

Annotate structure with `#[component]`

```rust
#[component]
struct Comp {}
```

Annotate impl blocks with `#[provides]`

```rust
#[provides]
impl Interface for Comp {}
```

Create a container:

```rust
fn main() {
    let mut container = Container::<profiles::Default>::new();
}
```

Get dependency ref:

```rust
fn main() {
    let comp = Provider::<dyn Interface>::get(&mut container);
}
```

## Inject references

For Rc:

```rust
#[component]
struct Dependency;

#[component]
struct Comp {
    dependency_rc: Rc<Dependency>
}

fn main() {
    let mut container = Container::<profiles::Default>::new();
    Provider::<Comp>::get(&mut container);
}
```


to use `Arc` instead of `Rc` you need to add `async` feature in cargo:
```toml
waiter_di = { version = "...", features = [ "async" ] }
```

Also, you can use `waiter_di::Wrc` type that will be compiled to `Rc` or `Arc` depending on `async` feature.

To create new struct instead of getting reference:

```rust
#[component]
struct Comp {
    dependency: Dependency,
    dependency_box: Box<Dependency>
}

fn main() {
    let mut container = Container::<profiles::Default>::new();
    Provider::<Comp>::create(&mut container);
    Provider::<Comp>::create_boxed(&mut container);
}
```

## Properties

It uses `config` crate under the hood, for example it tries to find `float_prop` 
in args as `--float_prop <value>`, if not found it tries to find it in environment variables, 
after that tries `config/{profile}.toml`, after that `config/default.toml`

```rust
#[derive(Debug, Deserialize)]
struct ConfigObject {
    i32_prop: i32
}

#[component]
struct Comp {
    config: Config,
    #[prop("int")] int_prop: usize,
    #[prop("int")] int_prop_opt: Option<usize>,
    #[prop("int" = 42)] int_prop_with_default_value: usize,
    float_prop: f32,
    #[prop] config_object: ConfigObject
}
```

## Dependency cycle

Use Deferred type:

```rust
#[component]
struct Comp {
    dependency_def: Deferred<Dependency>,
    dependency_def_rc: Deferred<Rc<Dependency>>,
    dependency_def_box: Deferred<Box<Dependency>>
}
```

## Profiles

You can use predefined profiles from `waiter_di::profile" or create custom:

```rust
struct CustomProfile;

#[provides(profiles::Dev, CustomProfile)]
impl Interface for Comp {}

fn main() {
    let mut container = Container::<profiles::Default>::new();
    let mut container = Container::<profiles::Dev>::new();
    let mut container = Container::<CustomProfile>::new();
}
```

## Get profile from args, environment or `config/default.toml`

Just define property named `profile` as `--profile <profile>` arg, `profile` env variable or 
`profile` property in `config/default.toml` and use `inject!` macro:

```rust
fn main() {
    let comp = inject!(Comp: profiles::Default, profiles::Dev);
}
```

`inject!` macro can't be used for several components, so it's recommended to use it with modules:

```rust
#[module]
struct SomeModule {
    component: Component
}
#[module]
struct RootModule {
    some_module: SomeModule
}
fn main() {
    let root_module = inject!(RootModule: profiles::Default, profiles::Dev);
}
```

In this case `#[module]` is just a synonym for `#[component]`

## Factory functions:

If you can't use `#[component]` annotation, use factory function instead:

```rust
#[provides]
fn create_dependency(bool_prop: bool) -> Dependency {
    Dependency { prop: bool_prop }
}
```

To use it like a constructor, use it with `#[component]` on impl block:

```rust
struct Comp();

#[component]
impl Comp {
    #[provides]
    fn new() -> Self {
        Self()
    }
}
```

`Deferred` args in factory functions is unsupported. In the rest it can accept 
the same arg types as `#[component]`.

External types isn't supported for factory functions:

```rust
#[provides] // won't compile
fn create_external_type_dependency() -> HashMap<i32, i32> {
    HashMap::new()
}
```

So you need to create crate-local wrapper:

```rust
struct Wrapper(HashMap<i32, i32>);

#[provides]
fn create_external_type_dependency() -> Wrapper {
    Wrapper(HashMap::new())
}
```

For convenience, you can use `#[wrapper]` attribute to implement Deref automatically:

```rust
#[wrapper]
struct HashMap(std::collections::HashMap<i32, i32>);

#[provides]
fn create_external_type_dependency() -> HashMap {
    return HashMap(std::collections::HashMap::<i32, i32>::new());
}
```