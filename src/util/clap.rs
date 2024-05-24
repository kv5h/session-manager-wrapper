use clap::{crate_description, crate_name, crate_version, Arg, ArgAction, Command};

pub fn new_clap_command() -> clap::ArgMatches {
    Command::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .arg(
            Arg::new("interactive")
                .short('I')
                .conflicts_with_all(["instance_id", "local_port", "remote_port", "remote_host"])
                .long("interactive")
                .action(ArgAction::SetTrue)
                .help("Use interactive mode\nConflicts with any other flag"),
        )
        .arg(
            Arg::new("instance_id")
                .short('i')
                .long("instance-id")
                .require_equals(false)
                .required(false)
                .required_unless_present("interactive")
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
                .help("Local port"),
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

// TODO: Add test
