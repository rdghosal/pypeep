use clap;

pub fn make_cli() -> clap::Command {
    tracing::debug!("initializing command");
    clap::Command::new(env!("CARGO_CRATE_NAME"))
        .about("Downloads a specified Python package and stores information about the current state of its dependencies.")
        .arg(
            clap::Arg::new("package")
                .long("package")
                .help("The Python package whose requirements (i.e., dependencies) we are checking.")
                .required(true),
        )
}
