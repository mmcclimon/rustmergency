mod config;
mod errors;
mod logger;
mod merger;
mod remote;
mod step;

use clap::{crate_name, crate_version, App, Arg};
use std::error::Error;

use merger::Merger;

const TEMPLATE: &str = "\
{bin} {version} - {about}

{usage}

{unified}
";

fn main() {
  let matches = App::new(crate_name!())
    .version(crate_version!())
    .about("build git branches from merge requests")
    .template(TEMPLATE)
    .arg(
      Arg::with_name("config")
        .short("c")
        .long("config")
        .value_name("FILE")
        .required(true)
        .help("config file to use")
        .takes_value(true),
    )
    .arg(
      Arg::with_name("auto")
        .long("auto")
        .help("do not run in interactive mode"),
    )
    .get_matches();

  let merger = Merger::from_config_file(matches.value_of("config").unwrap())
    .unwrap_or_else(|e| die(e));

  merger
    .run(matches.is_present("auto"))
    .unwrap_or_else(|e| die(e));
}

fn die(err: impl Error) -> ! {
  println!("{}", err);
  std::process::exit(1);
}
