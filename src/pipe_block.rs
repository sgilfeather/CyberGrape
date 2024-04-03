//! TODO: Header

use std::sync::mpsc::{channel, Receiver, SendError, Sender};
use std::thread;

///
/// A stage in the CyberGrape pipeline, which performs a step of the
/// data aggregation, binauralization, or music playback process.
///
pub trait Component<A, B> {
    /// Converts an input of type A into an output of type B
    fn convert(self: &Self, input: A) -> B;
}

///
/// Runs on its own thread. On receiving data of type A on input, the
/// PipeBlock converts them to data of type B, and sends it to its output
/// channel.
///
pub struct PipeBlock<'a, A, B> {
    // component is an object of trait Component, and Rust cannot know
    // how big component is at runtime. any 'dynamically typed' trait
    // object must be marked with the `dyn` keyboard, post 2021

    // the Box ensures that this PipeBlock owns the heap-alloc'd ref to the
    // underlying Component

    // the lifetime 'a indicates that the inner Component cannot live longer
    // than the PipeBlock!
    component: Box<dyn Component<A, B> + 'a>,
    input: Receiver<A>,
    output: Sender<B>,
}

///
impl<'a, A, B> PipeBlock<'a, A, B> {
    ///
    fn new(comp: Box<dyn Component<A, B> + 'a>, rx: Receiver<A>, tx: Sender<B>) -> Self {
        Self {
            component: comp,
            input: rx,
            output: tx,
        }
    }

    ///
    fn run(self: &Self) {
        while let Ok(data) = self.input.recv() {
            let out_data = self.component.convert(data);
            if let Err(error) = self.output.send(out_data) {
                // TODO: log error
            }
        }
        // TODO: log successful complete of run
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Null MockComponent for compilation testing
    struct MockComponent { 
        a: Option<()>,
        b: Option<()>
    }

    impl MockComponent {
        fn new() -> Self {
            Self { a: None, b: None }
        }
    }

    /// Null MockComponent for compilation testing
    impl<A, B> Component<A, B> for MockComponent {
        fn convert(self: &Self, input: Option<()>) -> Option<()> {
            Option::None
        }
    }
}
