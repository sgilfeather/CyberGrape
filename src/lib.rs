//! CyberGrape consists of a system of tactile blocks that represent audio
//! sources, or audio blocks, that track their angle and movement relative
//! to a central block, called the listener block. As audio blocks and the
//! listener block move relative to each other, their relative angles are
//! recorded. A software pipeline should be able to either encode the
//! streamed positional data into a well-defined serialization format or
//! create spatial audio directly using user-provided audio data.
//!
//! This is the host-side software that makes all of that happen, to see the
//! embedded software, take a look at [this GitHub repository](https://github.com/sgilfeather/CyberGrapeEmbedded).
//!
//! You can find our [final report](report) in this documentation site.
//!
//! This is the 2023/2024 senior capstone project for Team CyberGrape, which is
//! comprised of Ayda Aricanli, Skylar Gilfeather, Liam Strand, and Tyler
//! Thompson.

#![warn(missing_docs)]
pub mod args;
pub mod component;
pub mod dummy_hdm;
pub mod gui;
pub mod hardware_data_manager;
pub mod hardware_message_decoder;
pub mod hdm;
pub mod hound_helpers;
pub mod localizer;
pub mod report;
pub mod saf;
mod saf_raw;
pub mod spatial_data_format;
pub mod sphericalizer;
pub mod time_domain_buffer;
pub mod update_accumulator;

/// An iterator function that transposes the order of iteration based on
/// [this StackOverflow answer](https://stackoverflow.com/a/75477884/17443903).
/// I think this might stall on an empty iterator...not sure
pub struct TransposeIter<I, T>
where
    I: IntoIterator<Item = T>,
{
    iterators: Vec<I::IntoIter>,
}

#[allow(missing_docs)]
pub trait TransposableIter<I, T>
where
    Self: Sized,
    Self: IntoIterator<Item = I>,
    I: IntoIterator<Item = T>,
{
    fn transpose(self) -> TransposeIter<I, T> {
        let iterators: Vec<_> = self.into_iter().map(|i| i.into_iter()).collect();
        TransposeIter { iterators }
    }
}

impl<I, T> Iterator for TransposeIter<I, T>
where
    I: IntoIterator<Item = T>,
{
    type Item = Vec<T>;
    fn next(&mut self) -> Option<Self::Item> {
        let output: Option<Vec<T>> = self.iterators.iter_mut().map(|iter| iter.next()).collect();
        output
    }
}

impl<I, T, Any> TransposableIter<I, T> for Any
where
    Any: IntoIterator<Item = I>,
    I: IntoIterator<Item = T>,
{
}
