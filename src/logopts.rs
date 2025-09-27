use clap::Args;
#[derive(Args)]
pub struct LogOpts {
    /// Suppress most output
    #[arg(long, global = true, short = 'q')]
    pub quiet: bool,

    /// Repeat this option for more verbose logging
    #[arg(long, action=clap::ArgAction::Count, global = true, short = 'v')]
    pub verbose: u8,
}
