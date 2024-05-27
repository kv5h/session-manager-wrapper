use clap::{crate_description, crate_name, crate_version, Arg, Command};

pub fn new_clap_command() -> clap::ArgMatches {
    Command::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .arg(
            Arg::new("instance_id")
                .short('i')
                .long("instance-id")
                .require_equals(false)
                .required(true)
                .value_name("instance id")
                .help("Instance ID"),
        )
        .arg(
            Arg::new("local_port")
                .short('l')
                .long("local-port")
                .require_equals(false)
                .required(false)
                .value_name("local port")
                .value_parser(clap::value_parser!(u16).range(0..=65535))
                .help("Local port\nIf `0` is specified, an arbitrary free port will be assigned."),
        )
        .arg(
            Arg::new("remote_port")
                .short('p')
                .long("remote-port")
                .require_equals(false)
                .required(false)
                .value_name("remote port")
                .value_parser(clap::value_parser!(u16).range(0..=65535))
                .help("Remote port"),
        )
        .arg(
            Arg::new("remote_host")
                .short('r')
                .long("remote-host")
                .require_equals(false)
                .required(false)
                .value_name("remote host")
                .help("Remote host\nRequired only for port forwarding over bastion server"),
        )
        .get_matches()
}
