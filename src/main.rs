use aws_sdk::start_session::{start_session, SessionManagerProp};

mod aws_sdk;
mod util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Clap: Read command-line options
    let clap = util::clap::new_clap_command();
    let interactive = clap.get_flag("interactive");

    if interactive {
        // TODO:
        return Ok(());
    }

    let instance_id = clap
        .get_one::<String>("instance_id")
        .expect("`instance_id` required.")
        .to_owned();
    let local_port = match clap.try_get_one::<u16>("local_port")? {
        Some(val) => Some(val.to_owned()),
        None => None,
    };
    let remote_port = match clap.try_get_one::<u16>("remote_port")? {
        Some(val) => Some(val.to_owned()),
        None => None,
    };
    let remote_host = match clap.try_get_one::<String>("remote_host")? {
        Some(val) => Some(url::Host::parse(&val).expect(&format!("Failed to parse: {}", val))),
        None => None,
    };

    // Read environment variable
    let region = std::env::var("AWS_REGION").expect("Environment variable `AWS_REGION` not found.");

    // Log
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info);
    builder.init();

    let prop = SessionManagerProp::new(region, instance_id, local_port, remote_port, remote_host);

    match start_session(&prop).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
