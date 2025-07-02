use clap::{command, Arg, ArgAction, ArgMatches};

pub fn cli() -> ArgMatches {

    let matches = command!()
    .author("Jakub Němeček")
    .about("DNS Resolver.")
    .arg(
        Arg::new("domain name")
        .required(true)
    )
    .arg(
        Arg::new("server")
        .help("IP address of the DNS server")
        .short('s')
        .long("server")
        .required(false)
        .default_value("8.8.8.8")
    )
    .arg(
        Arg::new("verbose")
        .help("Print payload")
        .short('v')
        .long("verbose")
        .required(false)
        .action(ArgAction::SetTrue)
    )
    .get_matches();
    matches

}
