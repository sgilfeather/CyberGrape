//! Implements the GrapeBlock, a monitor wrapper for each CyberGrape
//! processing module. Enforces a common interface between module such that
//! each module can consume data from the preceding module, process it, and
//! pass new data to the subsequent module in the CyberGrape pipeline.

use log::{info, warn};
use std::fmt;
use std::any::Any;
use std::sync::mpsc::{Receiver, Sender};

///
/// A stage in the CyberGrape pipeline, which performs a step of the data
/// aggregation, binauralization, or music playback process. All structs
/// that perform a processing step in the CyberGrape system must implement
/// Component, so that they can be integrated into the pipeline.
///
pub trait Component: fmt::Display {
    type InData;
    type OutData;
    /// Converts an input of type A into an output of type B
    fn convert(self: &Self, input: Self::InData) -> Self::OutData;
}


///
/// Runs on its own thread. On receiving data of type A on input, the
/// GrapeBlock converts them to data of type B, and sends it to its output
/// channel.
///
pub struct GrapeBlock<'a, I, O> {
    // component is an object of trait Component, and Rust cannot know
    // how big component is at runtime. any 'dynamically typed' trait
    // object must be marked with the `dyn` keyboard, post 2021

    // the Box ensures that this GrapeBlock owns the heap-alloc'd ref to the
    // underlying Component

    // the lifetime 'a indicates that the inner Component cannot live longer
    // than the GrapeBlock!
    component: Box<dyn Component<InData = I, OutData = O> + 'a>,
    input: Receiver<I>,
    output: Sender<O>,
}

impl<'a, I, O> GrapeBlock<'a, I, O> {
    /// Instantiates a new GrapeBlock. The generic types on the GrapeBlock
    /// enforce that a GrapeBlock's Receiver type must match the InData
    /// type of its associated Component; and, that its Sender type must
    /// match the OutData type of its associated Component.
    pub fn new(
        component: Box<dyn Component<InData = I, OutData = O> + 'a>,
        input: Receiver<I>,
        output: Sender<O>,
    ) -> Self {
        Self {
            component,
            input,
            output,
        }
    }

    /// Runs the GrapeBlock: when it receives data on its input channel,
    /// it calls convert() from its associated Component on the input data,
    /// and writes the result to its output channel. The GrapeBlock
    /// terminates when it receives an Err on its input channel.
    pub fn run(self: &Self) {
        while let Ok(data) = self.input.recv() {
            let out_data = self.component.convert(data);
            if let Err(error) = self.output.send(out_data) {
                warn!("{} : received error {}.", self.component.to_string(), error);
            }
        }

        info!("{} : terminated.", self.component.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::mpsc::channel;

    /// Null MockComponent for compilation testing
    struct MockComponent {}

    impl MockComponent {
        fn new() -> Self {
            Self {}
        }
    }

    impl Component for MockComponent {
        type InData = i32;
        type OutData = i32;

        fn convert(self: &Self, input: i32) -> i32 {
            input + 1
        }
    }

    impl fmt::Display for MockComponent {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "MockComponent")
        }
    }

    /// Checks that Component can be implemented for custom inner types
    #[test]
    fn test_mock_component() {
        let mock_comp = MockComponent::new();
        assert_eq!(mock_comp.convert(0), 1);
    }

    /// Checks that a GrapeBlock's generic input and output types can be
    /// specified. Checks that writing a value to the GrapeBlock's input
    /// produces that value, converted, in the GrapeBlock's output
    #[test]
    fn test_mock_grape_block() {
        let mock_comp = MockComponent::new();
        let (test_tx, block_rx) = channel::<i32>();
        let (block_tx, test_rx) = channel::<i32>();

        thread::spawn(move || {
            let mock_grape_block = GrapeBlock::new(Box::new(mock_comp), block_rx, block_tx);
            mock_grape_block.run()
        });

        assert_eq!(test_tx.send(0), Ok(()));
        // TODO: how can we create GrapeBlock inside the closure and still be able to access tx and rx down here?
        assert_eq!(test_rx.recv(), Ok(1));
    }

    #[test]
    fn test_chained_grape_block() {
        let mock_comp_a = MockComponent::new();
        let mock_comp_b = MockComponent::new();

        let (test_tx, block_a_rx) = channel::<i32>();
        let (block_a_tx, block_b_rx) = channel::<i32>();
        let (block_b_tx, test_rx) = channel::<i32>();

        thread::spawn(move || {
            let mock_grape_block = GrapeBlock::new(Box::new(mock_comp_a), block_a_rx, block_a_tx);
            mock_grape_block.run()
        });

        thread::spawn(move || {
            let mock_grape_block = GrapeBlock::new(Box::new(mock_comp_b), block_b_rx, block_b_tx);
            mock_grape_block.run()
        });

        assert_eq!(test_tx.send(0), Ok(()));
        assert_eq!(test_rx.recv(), Ok(2));
    }
}
