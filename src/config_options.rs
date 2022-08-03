// Copyright (c) Mike Nye, Fred Clausen
//
// Licensed under the MIT license: https://opensource.org/licenses/MIT
// Permission is granted to use, copy, modify, and redistribute the work.
// Full license information available in the project LICENSE file.
//

use clap::Parser;
use log::LevelFilter;

#[derive(Parser, Debug, Clone)]
#[clap(name = "ACARS Router", author, version, about, long_about = None)]
pub struct Input {
    // Output Options
    /// Set the log level. debug, trace, info are valid options.
    #[clap(short, long, action = clap::ArgAction::Count)]
    pub(crate) verbose: u8,
    /// Semi-colon separated list of host:port to grab readsb data from
    #[clap(long, value_parser, value_delimiter = ';', required = true)]
    pub(crate) get_readsb: Vec<String>,
    /// Semi-Colon separated list of host:port to send readsb data to
    #[clap(long, value_parser, value_delimiter = ';', required = true)]
    pub(crate) send_readsb: Vec<String>,
}

pub trait SetupLogging {
    fn set_logging_level(self) -> LevelFilter;
}

impl SetupLogging for u8 {
    fn set_logging_level(self) -> LevelFilter {
        match self {
            0 => LevelFilter::Info,
            1 => LevelFilter::Debug,
            2..=u8::MAX => LevelFilter::Trace,
        }
    }
}
