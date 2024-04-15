use clap:: {
    Args, 
    Parser,
    Subcommand
};

#[derive(Debug, Parser)]
#[clap(version, about)]
pub struct GrapeArgs {
    /// First arg
    pub arg_1: String,
}


/// Initial test for arguments
#[test]
fn test_args() {

    let args = GrapeArgs::parse();
    print!("{:?}", args);
}