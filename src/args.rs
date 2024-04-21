// Commandline argument parser using clap for CyberGrape

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(version, about)]
pub struct GrapeArgs {
    #[command(subcommand, long_about)]
    /// Which task to perform, serialization or binauralization
    pub command: CommandTask,
    /// Sample rate of the audio file, in gHz
    #[arg(short, long = "samp")]
    pub samp_rate: u64,
    /// Duration of the audio file, in seconds
    #[arg(short, long = "dur")]
    pub duration: u64,
    // /// Path of the serial device used
    // #[arg(short = 'D', long = "device")]
    // pub device_path: String
}

#[derive(Debug, Subcommand)]
pub enum CommandTask {
    /// Encode positional data to a file in the GrapeFile format
    #[command(about)]
    Serial(SerialCommand),
    /// Combine N audio samples into a binauralized WAV file
    #[command(about)]
    Binaural(BinauralCommand),
}

#[derive(Debug, Args)]
#[command(version, about)]
pub struct SerialCommand {
    /// Filename for serialization output to be written to
    #[arg(short = 'f', long = "file")]
    pub serial_filename: String,
}

#[derive(Debug, Args)]
#[command(version, about)]
pub struct BinauralCommand {
    /// Number of input files to be assigned to audio blocks
    #[arg(short)]
    pub num_files: u32,
    /// Name of the final binaural WAV file to write to
    pub outfile_name: String,
    /// List of filenames, which should correspond to the number of input files. Put this flag last.
    /// This list is of arbitrary length and will parse until it hits another flag or the end of the command
    #[arg(short = 'f', long = "files")]
    #[clap(num_args = 1..)]
    pub filenames: Vec<String>,
}
