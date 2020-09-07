# Dependency injection for Rust

See `examples/get_started.rs` for list of available injection options

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

Create container:

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
}
```

## Properties

It uses `config` crate under the hood, for example it tries to find `float_prop` 
in environment, after that tries `config/default.toml`, after that `config/{profile}.toml`

```rust
#[derive(Debug, Deserialize)]
struct ConfigObject {
    i32_prop: i32
}

#[component]
struct Comp {
    config: Config,
    #[prop("int")] int_prop: usize,
    float_prop: f32,
    #[prop] config_object: ConfigObject
}
```

## Dependency cycle

Use Deferred type:

```rust
#[component]
struct Comp {
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

## Get profile from environment or `config/default.toml`

Just define property named `profile` and use `inject!` macro:

```rust
fn main() {
    let comp = inject!(Comp: profiles::Default, profiles::Dev);
}
```

## Factory functions:

If you can't use `#[component]` annotation, use factory function instead:

```rust
#[provides]
fn create_dependency(bool_prop: bool) -> Dependency {
    Dependency { prop: bool_prop }
}
```

Deferred args in factory functions is unsupported. In the rest it can accept 
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

For convenience you can use `#[wrapper]` attribute to implement Deref automatically:

```rust
#[wrapper]
struct HashMap(std::collections::HashMap<i32, i32>);

#[provides]
fn create_external_type_dependency() -> HashMap {
    return HashMap(std::collections::HashMap::<i32, i32>::new());
}
```