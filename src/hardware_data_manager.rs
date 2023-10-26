pub type Radian = f64;
pub type Id = usize;

#[derive(Debug, Clone)]
pub struct Update {
    pub src: Id,
    pub dst: Id,
    pub elv: Radian,
    pub azm: Radian,
}

/// `HardwareDataManager`
///
/// A typed, clearable iterator that emits `Update`s when iterated upon. Designed
/// to be maximally flexable to allow various implementations.
pub trait HardwareDataManager: Iterator<Item = Update> {
    fn new() -> Self;
    fn clear(&mut self);
}
