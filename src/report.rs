//! Our final report for the CyberGrape project
//! 
//! 
//! # Background
//!
//! Our project consists of a system of tactile blocks that represent audio
//! sources, or audio blocks, that track their angle and movement relative
//! to a central block, called the listener block. As audio blocks and the
//! listener block move relative to each other, their relative angles are
//! recorded. A software pipeline should be able to either encode the
//! streamed positional data into a well-defined serialization format or
//! create spatial audio directly using user-provided audio data.
//!
//! This was achieved through the use of two Bluetooth 5.1 XPLR
//! angle-of-arrival antenna boards and their sister tags. The tags, which
//! emit Bluetooth signals, were placed in the audio blocks. Two antenna
//! boards were placed in the listener block to encode a 360-degree range of
//! positional data capture, managed by a Raspberry Pi Pico W. The
//! angle-of-arrival data is transmitted from the listener block to a
//! central computer, at which point it can be interpreted by a Rust
//! software pipeline. Depending on user-provided instruction, the software
//! pipeline either writes the recorded positional data to our serialization
//! format or uses it to mix provided audio files into a single, two-channel
//! binaural audio file.
//!
//! # Requirements
//!
//! 1.  Sample and record positional data using an embedded system that
//!     takes advantage of Bluetooth 5.1 angle-of arrival capabilities
//!
//! 2.  Design a serialized file format that can encode positional data in
//!     an efficient and logical way for use in sound spatialization work,
//!     both within our system and without
//!
//! 3.  Deliver a software pipeline that, given recorded positional data,
//!     can binauralize separate audio streams, framewise, into a single
//!     stereo audio file
//!
//! # Minimum Viable Product
//!
//! The minimal version of this project has a hardware component and a
//! software component.
//!
//! The hardware system consists of a central listener block and multiple
//! source blocks. The hardware must be able to communicate the positions of
//! each source block relative to the listener block with enough accuracy to
//! convince a human listener that audio is coming from the direction they
//! placed the source block in during playback.
//!
//! The software component must be able to receive positional data from the
//! hardware system, then use a binauraliser to mix multiple audio files
//! into a final sound file. The mixed file should provide a convincing
//! surround sound experience where each input audio stream is pinned to the
//! location of a particular tag, following its position in the spatial
//! audio landscape.
//!
//! On startup, the software must provide some kind of interface to set up
//! the input audio files and their mappings to the source blocks. This
//! interface can be either a TUI or a proper GUI.
//!
//! # Deliverables
//!
//! The sponsor received the listener block, tags, and access to the
//! CyberGrape codebase. The listener block is built out of laser cut 3mm
//! plywood, and contains two U-blox AoA antennas, a Raspberry Pi Pico, and
//! a serial-to-USB adapter. The electronics are soldered to a stripboard.
//! The "tags" are bluetooth-emitting devices provided by U-blox, and are
//! tracked by the AoA antennas; they serve as the source blocks.
//!
//! The listener block can be connected to a computer with a USB cable, then
//! the CyberGrape software can interface with the hardware. The software is
//! a Rust codebase that can collect serial data from the listener block,
//! then either output serialized position data or mix together multiple
//! audio streams using the SAF binauralization library.
//!
//! The sponsor provided full funding for the hardware, and is receiving
//! both the hardware and the software final products. The codebase contains
//! documentation about its different components and instructions on how to
//! use it with the hardware.
//!
//! # System Architecture
//!
//! ![](https://www.eecs.tufts.edu/~lstran01/capstone_report_images/architecture.png)
//!
//! # Documentation
//!
//! Once you have the codebase locally via `git clone`, run `cargo doc --open` to
//! generate a full webpage of the codebase documentation. Documentation for
//! the command line can be accessed via the `--help` flag:
//!
//! ```shell
//! cargo run --bin cybergrape --release -- --help
//! ```
//!
//! # Interface Design
//!
//! Our startup interface is via the command line, which provides two
//! options. The first, given information from the user, will record
//! positional data according to the length of provided mono audio files and
//! binauralize the audio files, according to the recorded positions, into a
//! stereo audio file.
//!
//! The following is an example command that samples position 40 times per
//! second, and expects an input of 2 audio files, x.wav and y.wav, with
//! sample rates of 44,100 gHz (this is the default value if `--samp` is not
//! provided), gains of 1, and ranges of 3 and 4, respectively. The final
//! binaural audio will be written to a file called out.wav.
//!
//! ```shell
//! cargo run --bin cybergrape --release -- --update 40 binaural --samp \
//! 44100 -n 2 --outfile out.wav --gains 1 1 --ranges 3 4 --files x.wav \
//! y.wav
//! ```
//!
//! The second type of command allows a user to record positional data for
//! an arbitrary length of time and encode it in our serial file format to a
//! filename of the user's choosing.
//!
//! The following is an example command that samples position 40 times per
//! second from 2 audio tags, and outputs the final serialized positional
//! data to a file named out.grape.
//!
//! ```shell
//! cargo run --bin cybergrape --release -- --update 40 serial -n 2 -o out.grape
//! ```
//!
//! Upon running either of these startup commands successfully, a TUI prompt
//! will appear resembling the following, at which point the user needs to
//! select the CyberGrape listener block from their computer's serial
//! device list (not in the list in the image).
//!
//! ![](https://www.eecs.tufts.edu/~lstran01/capstone_report_images/tui.png)
//!
//! If in binaural mode, upon device selection, the system will then begin
//! recording and continue to do so until the length of the longest audio
//! file provided. Move those audio tags around!
//!
//! If in serial mode, upon device selection, the system will record
//! positional data until the user hits any key, at which point recording
//! will cease.
//!
//! # Milestone Timeline
//!
//! ### 1. Hardware Block Designs and Prototypes
//!
//! This is the hardware component, a network of physical blocks, that will
//! use Bluetooth signals to generate raw positional and angle-of-arrival
//! data for use by our software components.
//!
//! - Preliminary Design and CAD Model: Early February
//!
//! - Preliminary prototype: Early April
//!
//! - Final Hardware Bringup: Mid-April
//!
//! Following the preliminary design, the hardware prototyping and bringup
//! involved flashing firmware on the antenna boards and reading data off of
//! the antenna boards using Raspberry Pi Pico W's. Due to poor
//! documentation of the antenna boards, this process took much longer than
//! anticipated, which delayed its completion until mid-April.
//!
//! ### 2. Hardware Data Manager
//!
//! This is an internal process within the software system that manages
//! communications between the Cyber Grape listener block and its sound
//! source blocks.
//!
//! - Design Hardware Data Manager: Mid October
//!
//! - Write Mock Hardware: Late October
//!
//! - Implement Real Hardware Data Manager: Mid April
//!
//! - Plumbing from Hardware Data Manager to Hardware Localization Interface: Late April
//!
//! Due to the hardware issues we experienced, the hardware data manager
//! timeline also was affected. Once the hardware bringup was achieved, we
//! interleaved data read from the antennas into messages on a Pico W, which
//! our software interprets as position updates.
//!
//! ### 3. Binauralization API
//!
//! This is an internal software component that, given positional data from
//! our hardware localization algorithm, will produce the final mixed
//! spatial audio.
//!
//! - Design interface: Early December
//!
//! - Initial Implementation: Mid-February
//!
//! - Final Implementation: Mid-March
//!
//! This module was implemented and tested early on. This module consists of
//! a Rust wrapper that links an existing binauralization library written in
//! C. Creating a safe interaction between the two languages is nontrivial,
//! but the module was completed according to our original plan.
//!
//! ### 4. Serialization Standard File Format and API
//!
//! This is an internal software component that encodes interleaved
//! positional information as read by the antennas in a custom serialized
//! data format.
//!
//! - Design specification: Mid-February
//!
//! - Final API Implementation: Mid March
//!
//! Given positional data provided by our hardware system, the serialization
//! format encodes metadata about the positional data in a header, and
//! encodes interleaved channels of positional data in binary for space
//! efficiency. This milestone did not depend on the hardware bringup and
//! was implemented on time.
//!
//! ### 5. User Terminal Interface
//!
//! This is a user-facing terminal interface, which will enable users to
//! configure audio blocks, activate audio mixing mode, and select an output
//! channel for the produced binauralized audio.
//!
//! - Ratatui interface design and preliminary implementation: Mid-April
//!
//! - Final implementation: Late April
//!
//! This was a stretch goal that was not wholly realized due to hardware
//! problems resulting in time constraints. We designed a robust commandline
//! parser and included a smaller-scale selection TUI to configure the
//! hardware system and record audio data intuitively. Further work could be
//! done on providing a fuller TUI or a GUI for this system.
//!
//! # Acceptance Testing
//!
//! ### Criterion 1: Exportable Serialization Format
//!
//! We required that the serialization format encodes positional data
//! correctly, is space-efficient, is exportable, and has clear
//! documentation. This criterion passed all acceptance tests. We designed
//! the format to be space-efficient by using a binary encoding and
//! exportable via only requiring system knowledge of Rusty Object Notation.
//! We implemented the format correctly, as confirmed by unit testing, and
//! documented thoroughly.
//!
//! ### Criterion 2: Spatialization of Audio
//!
//! We required the audio spatialization to meet the following criteria:
//! provide audible sonic feedback of positional information, multi-channel
//! support, and output on stereo. The testing process for this involved
//! minute unit tests, along with end-to-end tests utilizing various audio
//! samples and sets of positional data to ensure audio spatialization
//! sounded as a user would expect.
//!
//! ### Criterion 3: Hardware Collection of Positional Data
//!
//! We required that hardware blocks communicate with each other via
//! Bluetooth, the Listener block and the central computer interface
//! successfully, and the Hardware Data Manager to communicate positional
//! data.
//!
//! ### Criterion 4: Low Latency Data Collection
//!
//! This criterion required that we would be able to hear live feedback of
//! audio spatialization with no audible delay. Due to a reduction in our
//! project scope over the course of the semester due to time constraints,
//! this acceptance test became not applicable. However, with some more
//! work, this system could be adjusted to implement live audio feedback of
//! the spatialization, since we incorporated this goal into the design of
//! the system.
//!
//! # Ethics and Social Impact
//!
//! The vision for social impact of this project was to develop a spatial
//! audio creation technology that is intuitive to everyone and easy to use,
//! in addition to being accessible for those who cannot use a visual
//! interface. In addition, this technology could be used to recreate
//! synesthetic experiences.
//!
//! There are not many ethical concerns to consider for this project, since
//! it is an audio creation tool not intended for any particular purpose.
//! However, legally, the project could be implicated in relation to
//! intellectual property and protected audio content, if it is used to
//! spatialize protected content without proper licensing. Clear guidelines
//! stating the usage, distribution, and ownership of audio content created
//! with this system would prevent the individuals or organization
//! distributing this system from incurring any liability.
//!
//! From a security perspective, the use of Bluetooth could be an avenue for
//! malicious actors to eavesdrop on, disrupt, or hijack with their own
//! messages, especially if the central computer is eventually connected to
//! the listener block with a wireless connection.
//!
//! # Reflection
//!
//! This project had many triumphs and pitfalls to navigate. We successfully
//! completed all the important points in our MVP and succeeded in creating
//! a functional end-to-end system that reads positional data from hardware
//! components and spatializes audio streams according to that input.
//! Despite having to reduce our scope due to unforeseen challenges such as
//! poor hardware documentation, scheduling conflicts, and long-term
//! illness, we are proud of the technology we've built and the idea we've
//! brought to life.
//!
//! Guided by our sponsor, Professor James Intriligator, we designed the
//! system to encompass more use cases than originally imagined, including
//! recording positional data to a file format, and making sure the listener
//! block was portable enough to be moved itself.
//!
//! Our team consisted of four undergraduate senior Computer Science majors,
//! Ayda Aricanli, Skylar Gilfeather, Liam Strand, and Tyler Thompson.
//!
//! We all learned a great deal over the course of the project, both
//! personally and technically. We often juggled how to balance work as a
//! team over time, collectively manage a project of this scope, and choose
//! a project that we were all equally interested in. For each of us, there
//! were aspects of this project that none of us had ever attempted
//! before---flashing hardware, Rust, how to correct soldering mistakes, and
//! more.
//!
//! Many thanks to Professor Intriligator for his guidance, Professor
//! Lillethun for organizing the course, the entire team for pouring their
//! time into this project over the past year, and to Tyler's roommate Greg,
//! for helping us with the hardware in a time of need.
//!
//! # Appendix
//!
//! We used the [XPLR-AOA-1 Product summary](https://content.u-blox.com/sites/default/files/XPLR-AOA-1_ProductSummary_UBX-21015378.pdf)
//! to get an idea of the capabilities and limitations of the U-block AoA
//! antennas.
//!
//! The [XPLR-AOA-1 and XPLR-AOA-2 explorer kits: Bluetooth direction finding User guide](https://content.u-blox.com/sites/default/files/XPLR-AOA-Explorer-kits_UserGuide_UBX-21004616.pdf)
//! from U-blox was the primary source of information that was used to build
//! up the hardware portion of the project.
//!
