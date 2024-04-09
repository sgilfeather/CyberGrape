//! TODO: Header

use std::sync::mpsc::{Receiver, Sender};
use log::{error, info};

/// A stage in the CyberGrape pipeline, which performs a step of the
/// data aggregation, binauralization, or music playback process.
pub trait Component {
    type InData;
    type OutData;
    /// Converts an input of type A into an output of type B
    fn convert(self: &Self, input: Self::InData) -> Self::OutData;
}

/// Runs on its own thread. On receiving data of type A on input, the
/// PipeBlock converts them to data of type B, and sends it to its output
/// channel.
pub struct PipeBlock<'a, I, O> {
    // component is an object that implments trait Component, and Rust cannot
    // know how big component is at runtime. any 'dynamically typed' trait
    // object must be marked with the `dyn` keyboard, post 2021

    // the Box ensures that this PipeBlock owns the heap-alloc'd ref to the
    // underlying Component

    // the lifetime 'a indicates that the inner Component cannot live longer
    // than the PipeBlock!
    component: Box<dyn Component<InData = I, OutData = O> + 'a>,
    output: Sender<O>,
    input: Receiver<I>,
}

impl<'a, I, O> PipeBlock<'a, I, O> {
    /// TODO
    fn new(
        comp: Box<dyn Component<InData = I, OutData = O> + 'a>,
        tx: Sender<O>,
        rx: Receiver<I>,
    ) -> Self {
        Self {
            component: comp,
            input: rx,
            output: tx,
        }
    }

    /// TODO
    fn run(self: &Self) {
        while let Ok(data) = self.input.recv() {
            let out_data = self.component.convert(data);
            if let Err(error) = self.output.send(out_data) {
                error!("{}", error);
            }
        }
        info!("Block complete");
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

    /// Checks that Component can be implemented for custom inner types
    #[test]
    fn test_mock_component() {
        let mock_comp = MockComponent::new();
        assert_eq!(mock_comp.convert(0), 1);
    }

    /// Checks that a PipeBlock's generic input and output types can be
    /// specified. Checks that writing a value to the PipeBlock's input
    /// produces that value, converted, in the PipeBlock's output
    #[test]
    fn test_mock_pipe_block() {
        let mock_comp = MockComponent::new();
        let (test_tx, block_rx) = channel::<i32>();
        let (block_tx, test_rx) = channel::<i32>();

        thread::spawn(move || {
            let mock_pipe_block = PipeBlock::new(Box::new(mock_comp), block_tx, block_rx);
            mock_pipe_block.run()
        });

        assert_eq!(test_tx.send(0), Ok(()));
        // TODO: how can we create PipeBlock inside the closure and still be able to access tx and rx down here?
        assert_eq!(test_rx.recv(), Ok(1));
    }

    #[test]
    fn test_chained_pipe_block() {
        let mock_comp_a = MockComponent::new();
        let mock_comp_b = MockComponent::new();

        let (test_tx, block_a_rx) = channel::<i32>();
        let (block_a_tx, block_b_rx) = channel::<i32>();
        let (block_b_tx, test_rx) = channel::<i32>();

        thread::spawn(move || {
            let mock_pipe_block = PipeBlock::new(Box::new(mock_comp_a), block_a_tx, block_a_rx);
            mock_pipe_block.run()
        });

        thread::spawn(move || {
            let mock_pipe_block = PipeBlock::new(Box::new(mock_comp_b), block_b_tx, block_b_rx);
            mock_pipe_block.run()
        });

        assert_eq!(test_tx.send(0), Ok(()));
        assert_eq!(test_rx.recv(), Ok(2));
    }
}
