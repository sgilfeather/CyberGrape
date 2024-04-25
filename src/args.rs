// Commandline argument parser using clap for CyberGrape

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser, Clone)]
#[clap(version, about)]
pub struct GrapeArgs {
    #[command(subcommand, long_about)]
    /// Which task to perform, serialization or binauralization
    pub command: CommandTask,

    /// Sample rate of the audio files, in gHz. Will often be 44100
    #[arg(short = 's', long = "samp")]
    pub samp_rate: f32,

    /// How often the location of the audio blocks are sampled, in updates per second
    #[arg(short = 'u', long = "update")]
    pub update_rate: f32,
}

#[derive(Debug, Subcommand, Clone)]
pub enum CommandTask {
    /// Encode positional data to a file in the GrapeFile format
    #[command(about)]
    Serial(SerialCommand),

    /// Combine N audio samples into a binauralized WAV file
    #[command(about)]
    Binaural(BinauralCommand),
}

#[derive(Debug, Args, Clone)]
#[command(version, about)]
pub struct SerialCommand {
    /// Filename for serialization output to be written to
    #[arg(short = 'o', long = "out")]
    pub outfile: String,

    /// Number of tags to record spatial data from
    #[arg(short)]
    pub num_tags: u32,
}

#[derive(Debug, Args, Clone)]
#[command(version, about)]
pub struct BinauralCommand {
    /// Number of input files to be assigned to audio blocks
    #[arg(short)]
    pub num_files: u32,

    /// Filename for binaural audio data to be written to
    #[arg(short = 'o', long = "out")]
    pub outfile: String,

    /// List of filenames, which should correspond to the number of input files
    #[arg(short = 'f', long = "files")]
    #[clap(num_args = 1..)]
    pub filenames: Vec<String>,

    /// List of gains, which should correspond to the input files given
    #[arg(short = 'g', long = "gains")]
    #[clap(num_args = 1..)]
    pub gains: Vec<f32>,

    /// List of ranges fields, which should correspond to the input files given
    #[arg(short = 'r', long = "ranges")]
    #[clap(num_args = 1..)]
    pub ranges: Vec<f32>,
}
