use std::fmt;

use clap::Args;
use stderrlog::LogLevelNum;
#[derive(Args)]
pub struct Verbosity {
    ///Suppress most output
    #[arg(long, global = true, short = 'q')]
    quiet: bool,

    ///Repeat this option for more verbose logging
    #[arg(long, action=clap::ArgAction::Count, global = true, short = 'v')]
    verbose: u8,
}
impl Verbosity {
    pub fn log_level(&self) -> LogLevelNum {
        if self.quiet {
            return LogLevelNum::Off;
        }
        if self.verbose == 1 {
            return LogLevelNum::Debug;
        }
        if self.verbose > 1 {
            return LogLevelNum::Trace;
        }

        LogLevelNum::Info
    }

    pub fn is_quied(&self) -> bool {
        self.quiet
    }
}
impl fmt::Display for Verbosity {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(match self.log_level() {
            LogLevelNum::Off => "QUIET",
            LogLevelNum::Error => "ERROR",
            LogLevelNum::Warn => "WARN",
            LogLevelNum::Info => "INFO",
            LogLevelNum::Debug => "DEBUG",
            LogLevelNum::Trace => "TRACE",
        })
    }
}
