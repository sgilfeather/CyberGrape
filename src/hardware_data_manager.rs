//! An interface definition for the hardware/software barrier.

// This is just like the typedef you've seen in C. `pub` means that someone
// who imports this module (everything in this file) will have access to those
// type aliases.
/// [Wikipedia's description](https://en.wikipedia.org/wiki/Radian)
pub type Radian = f64;
/// We will identify each object with a unique pointer-sized integer...hopefully
/// that will be enough!
pub type Id = usize;

// The `#[derive(...)]` bit here is actually a macro. We are saying we want the
// compiler to derive a few trait implementations for us.
//
// The first, `Debug` says that the type can be printed in an exhaustive way
// for debugging. The compiler can derive this because base types already
// implement `Debug`. We can derive `Debug` on any combination of types that
// already implement it.
//
// The second, `Clone`, means that the `.clone()` method can be called on the
// struct to make a copy. All nested types must implement `Clone` for this
// to work.

/// This struct represents a radial measurement taken from `src` to `dst`.
#[derive(Debug, Clone)]
pub struct Update {
    /// The identifier of the object that we are measuring from
    pub src: Id,
    /// The identifier of the object that we are measuring to
    pub dst: Id,
    /// The angle from `src` to `dst` _above_ the x/y plane, this is called "elevation".
    pub elv: Radian,
    /// The radians of the angle from `src` to `dst` in the x/y plane, which we call the "azimuth".
    pub azm: Radian,
}

// A `trait` is like an Interface in Java or an abstract Class in C++. It
// represents functionality that a struct can implement, then be treated
// generically. We've defined the `HardwareDataManager` trait to also require
// that the

/// A typed, clearable iterator that emits [`Update`]s when iterated upon. Designed
/// to be maximally flexable to allow various implementations.
pub trait HardwareDataManager: Iterator<Item = Update> {
    /// Empties the message queue contained within the `HardwareDataManager`.
    /// This is helpful when the consumer of this queue is unable to keep up
    /// with the `Update`s and wants to skip forward to the most recent items.
    fn clear(&mut self);
}
