# Delegating Trait Implementation to a `struct` Member (Automatic Casting)

This is similar to some of the main reasons you may use multiple inheritance,
or mix-ins in an object oriented language.

Imagine you have the following trait `VolumeCubicM`, and trait
`CurrentResistanceOhms`.

```rust
pub trait VolumeCubicM {
    fn volume_cubic_m(&self) -> f64;
}
```

```rust
pub trait CurrentResistanceOhms {
    fn current_resistance_ohms(&self) -> f64;
}
```

Imagine you implement `VolumeCubicM` for struct `Cylinder`, and
`CurrentResistanceOhms` for struct `Resistor` as follows.

```rust
#[derive(Debug, Clone)]
struct Cylinder {
    radius_m: f64,
    length_m: f64,
}

impl VolumeCubicM for Cylinder {
    fn volume_cubic_m(&self) -> f64 {
        std::f64::consts::PI * self.radius_m * self.radius_m * self.length_m
    }
}

#[derive(Debug, Clone)]
struct Resistor {
    current_resistance_ohms: f64,
}

impl CurrentResistanceOhms for Resistor {
    fn current_resistance_ohms(&self) -> f64 {
        self.current_resistance_ohms
    }
}
```

Imagine now you define struct `HeatingElement` as follows.

```rust
#[derive(Debug, Clone)]
struct HeatingElement {
    geometry: Cylinder,
    resistance: Resistor,
}
```

And, you want `HeatingElement` to implement both `VolumeCubicM`, and
`CurrentResistanceOhms`.  One approach would be to add an
`impl VolumeCubicM for HeatingElement`, and an
`impl CurrentResistanceOhms for HeatingElement` definition.  In our example
where each trait has only a single function, and `HeatingElement` is the only
struct for which want to implement the two traits this would be a workable
approach.  However, if the traits had many functions (perhaps, through
supertraits), and we had multiple structs, for which we want to implement
traits like `VolumeCubicM`, and `CurrentResistanceOhms` like in the above
example, such an approach would become repetitive, and hard to maintain.

Instead we're going to define `trait AutoCastAsVolumeCubicM`, and
`trait AutoCastAsCurrentResistanceOhms`.  Any type that implements
`AutoCastAsVolumeCubicM` would automatically also implement `VolumeCubicM`,
and any type that implements `AutoCastAsCurrentResistanceOhms` would
automatically also implement `CurrentResistanceOhms`.

```rust
pub trait AutoCastAsVolumeCubicM {
    fn as_volume_cubic_m_ref(&self) -> &impl VolumeCubicM;
}

impl<T: AutoCastAsVolumeCubicM> VolumeCubicM for T {
    fn volume_cubic_m(&self) -> f64 {
        self.as_volume_cubic_m_ref().volume_cubic_m()
    }
}
```

```rust
pub trait AutoCastAsCurrentResistanceOhms {
    fn as_current_resistance_ohms_ref(&self) -> &impl CurrentResistanceOhms;
}

impl<T: AutoCastAsCurrentResistanceOhms> CurrentResistanceOhms for T {
    fn current_resistance_ohms(&self) -> f64 {
        self.as_current_resistance_ohms_ref()
            .current_resistance_ohms()
    }
}
```

Now all we need to write in order to `impl VolumeCubicM for HeatingElement`
(regardless of how many functions `trait VolumeCubicM` has) is:

```rust
impl AutoCastAsVolumeCubicM for HeatingElement {
    fn as_volume_cubic_m_ref(&self) -> &impl VolumeCubicM {
        &self.geometry
    }
}
```

Simiralry to `impl CurrentResistanceOhms for HeatingElement`:

```rust
impl AutoCastAsCurrentResistanceOhms for HeatingElement {
    fn as_current_resistance_ohms_ref(&self) -> &impl CurrentResistanceOhms {
        &self.resistance
    }
}
```

This is achives approximately the same goals as crates like
[delegate](https://crates.io/crates/delegate), and
[ambassador](https://crates.io/crates/ambassador), but using generics instead
of macros.

Of course, using a macro to derive definitions like
`pub trait AutoCastAsVolumeCubicM`, and
`impl<T: AutoCastAsVolumeCubicM> VolumeCubicM for T` can be helpful.
