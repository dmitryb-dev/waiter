# Dependency injection for Rust

See examples/get_started.rs for list of available injection options

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

For &ref

```rust
#[component]
struct Comp<'a> {
    dependency_ref: &'a Dependency
}

fn main() {
    let mut container = Container::<profiles::Default>::new();
    Provider::<Comp>::get_ref(&mut container);
}
```

For create new struct instead of reference:

```rust
#[component]
struct Comp {
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
#[component]
struct Comp {
    config: Config,
    #[prop("int")] int_prop: usize,
    float_prop: f32
}
```

## Dependency cycle

Use Deferred type:

```rust
#[component]
struct Comp<'a> {
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

```
let comp = inject!(Comp: profiles::Default, profiles::Dev);
```

## Factory functions:

// TODO