//! Defines the Component trait, to be used by each CyberGrape processing
//! module. This enforces a common interface between modules, so that each
//! module can consume data from the preceding module, process it, and pass
//! new data to the subsequent module in the CyberGrape pipeline.

use log::{info, warn};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self, JoinHandle};

#[derive(Debug)]
pub enum ComponentError {
    HoundError(hound::Error),
}

///
/// A stage in the CyberGrape pipeline, which performs a step of the data
/// aggregation, binauralization, or music playback process. All structs
/// that perform a processing step in the CyberGrape system must implement
/// Component, so that they can be integrated into the pipeline.
///
pub trait Component: ToString {
    type InData;
    type OutData;

    /// Converts an input of type A into an output of type B
    fn convert(self: &mut Self, input: Self::InData) -> Self::OutData;

    /// Cleans up at termination of pipeline
    fn finalize(self: &mut Self) -> Result<(), ComponentError>;
}

/// Runs the given Component on its own thread. On receiving data of type
/// InData on the input channel, the Component converts them to data of type
/// OutData and sends it to the output channel.
pub fn run_component<C: Component + std::marker::Send + 'static>(
    mut component: Box<C>,
    input: Receiver<<C as Component>::InData>,
    output: Sender<<C as Component>::OutData>,
) -> JoinHandle<()>
where
    <C as Component>::InData: Send + 'static,
    <C as Component>::OutData: Send + 'static,
{
    thread::spawn(move || {
        while let Ok(data) = input.recv() {
            let out_data = component.convert(data);
            if let Err(error) = output.send(out_data) {
                warn!("{} : received error {}.", component.to_string(), error);
            }
        }

        if let Err(component_error) = component.finalize() {
            warn!(
                "{} : error during terminating : {component_error:?}.",
                component.to_string(),
            );
        }
        info!("{} : terminated.", component.to_string());
    })
}

#[cfg(test)]
mod tests {
    use super::*;
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

        fn convert(self: &mut Self, input: i32) -> i32 {
            input + 1
        }

        fn finalize(self: &mut Self) -> Result<(), ComponentError> {
            Ok(())
        }
    }

    impl ToString for MockComponent {
        fn to_string(&self) -> String {
            "MockComponent".to_string()
        }
    }

    /// Checks that a Component's generic input and output types can be
    /// specified. Checks that writing a value to the Component's input
    /// produces that value, converted, in the Component's output
    #[test]
    fn test_mock_component() {
        let mock_comp = MockComponent::new();
        let (test_tx, block_rx) = channel::<i32>();
        let (block_tx, test_rx) = channel::<i32>();

        run_component(Box::new(mock_comp), block_rx, block_tx);

        assert_eq!(test_tx.send(0), Ok(()));
        // TODO: how can we create Component inside the closure and still be able to access tx and rx down here?
        assert_eq!(test_rx.recv(), Ok(1));
    }

    #[test]
    fn test_chained_component() {
        let mock_comp_a = MockComponent::new();
        let mock_comp_b = MockComponent::new();

        let (test_tx, block_a_rx) = channel::<i32>();
        let (block_a_tx, block_b_rx) = channel::<i32>();
        let (block_b_tx, test_rx) = channel::<i32>();

        run_component(Box::new(mock_comp_a), block_a_rx, block_a_tx);

        run_component(Box::new(mock_comp_b), block_b_rx, block_b_tx);

        assert_eq!(test_tx.send(0), Ok(()));
        assert_eq!(test_rx.recv(), Ok(2));
    }
}
