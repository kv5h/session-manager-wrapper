use std::{io, ops::Deref, process::Stdio};

use aws_config::BehaviorVersion;
use aws_types::region::Region;
use serde_json::json;
use tokio::process::Command;

const SESSION_MANAGER_BIN_NAME: &str = "session-manager-plugin";

#[derive(PartialEq, Debug, Clone, Copy)]
enum DocumentName {
    AwsStartPortForwardingSession,
    AwsStartPortForwardingSessionToRemoteHost,
    None,
}

impl DocumentName {
    fn get_document_name(&self) -> Option<String> {
        match self {
            Self::AwsStartPortForwardingSession => {
                Some(String::from("AWS-StartPortForwardingSession"))
            },
            Self::AwsStartPortForwardingSessionToRemoteHost => {
                Some(String::from("AWS-StartPortForwardingSessionToRemoteHost"))
            },
            Self::None => None,
        }
    }
}

pub struct SessionManagerProp {
    /// AWS Region
    region: String,
    /// Instance ID
    instance_id: String,
    /// Local port
    local_port: Option<u16>,
    /// Remote port
    remote_port: Option<u16>,
    /// Remote host
    remote_host: Option<url::Host>,
}

impl SessionManagerProp {
    pub fn new(
        region: String,
        instance_id: String,
        local_port: Option<u16>,
        remote_port: Option<u16>,
        remote_host: Option<url::Host>,
    ) -> Self {
        Self {
            region,
            instance_id,
            local_port,
            remote_port,
            remote_host,
        }
    }
}

async fn get_client(region: &str) -> aws_sdk_ssm::Client {
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new(region.to_owned()))
        .load()
        .await;

    aws_sdk_ssm::Client::new(&config)
}

fn check_binary_exist() {
    match subprocess::Exec::cmd(SESSION_MANAGER_BIN_NAME).join() {
        Ok(o) => {
            assert!(o.success())
        },
        Err(e) => {
            log::error!("{}. The executable {} not found. Install it at {}",
                e,
                SESSION_MANAGER_BIN_NAME,
                "https://docs.aws.amazon.com/systems-manager/latest/userguide/session-manager-working-with-install-plugin.html"
            );
            panic!()
        },
    }
}

/// Reference: https://github.com/aws/session-manager-plugin/blob/mainline/src/sessionmanagerplugin/session/session.go
pub async fn start_session(prop: &SessionManagerProp) -> Result<(), Box<dyn std::error::Error>> {
    // Assert executable exist
    check_binary_exist();

    let mode = if prop.remote_host.is_some() {
        DocumentName::AwsStartPortForwardingSessionToRemoteHost
    } else if prop.local_port.is_some() && prop.remote_port.is_some() {
        DocumentName::AwsStartPortForwardingSession
    } else {
        DocumentName::None
    };

    log::info!("Document name: {:?}", mode);

    let resp = match mode {
        m if m == DocumentName::AwsStartPortForwardingSessionToRemoteHost => {
            get_client(&prop.region)
                .await
                .start_session()
                .target(&prop.instance_id)
                .document_name(m.get_document_name().unwrap())
                .parameters("host", vec![prop.remote_host.clone().unwrap().to_string()]) // TODO:
                .parameters("portNumber", vec![prop.remote_port.unwrap().to_string()])
                .parameters("localPortNumber", vec![prop
                    .local_port
                    .unwrap()
                    .to_string()])
                .send()
                .await
        },
        m if m == DocumentName::AwsStartPortForwardingSession => {
            get_client(&prop.region)
                .await
                .start_session()
                .target(&prop.instance_id)
                .document_name(m.get_document_name().unwrap())
                .parameters("portNumber", vec![prop.remote_port.unwrap().to_string()])
                .parameters("localPortNumber", vec![prop
                    .local_port
                    .unwrap()
                    .to_string()])
                .send()
                .await
        },
        _ => {
            get_client(&prop.region)
                .await
                .start_session()
                .target(&prop.instance_id)
                .send()
                .await
        },
    };

    let resp_json: serde_json::Value;
    match resp {
        Ok(o) => {
            log::info!("Session ID: {}", o.session_id().unwrap());
            resp_json = json!({
                // TODO: NG
                //"session_id": o.session_id().unwrap(),
                //"token_value": o.token_value().unwrap(),
                //"stream_url": o.token_value().unwrap(),
                "SessionId": o.session_id().unwrap(),
                "TokenValue": o.token_value().unwrap(),
                "StreamUrl": o.stream_url().unwrap(),
            });
        },
        Err(e) => {
            log::error!("{e}");
            return Err(e.into());
        },
    }

    let session_manager_param = match mode {
        m if m == DocumentName::AwsStartPortForwardingSessionToRemoteHost => {
            json!({
                "Target" : &prop.instance_id,
                "DocumentName": m.get_document_name().unwrap(),
                "parameters": {
                    "host": vec![prop.remote_host.clone().unwrap().to_string()],
                    "portNumber": vec![prop.remote_port.unwrap().to_string()],
                    "localPortNumber": vec![prop.local_port.unwrap().to_string()]
                }
            })
        },
        m if m == DocumentName::AwsStartPortForwardingSession => {
            json!({
                "Target" : &prop.instance_id,
                "DocumentName": m.get_document_name().unwrap(),
                "parameters": {
                    "portNumber": vec![prop.remote_port.unwrap().to_string()],
                    "localPortNumber": vec![prop.local_port.unwrap().to_string()]
                }
            })
        },
        _ => {
            json!({"Target" : &prop.instance_id})
        },
    };

    println!("{}", session_manager_param.to_string()); // TODO:
    println!(
        "{} {} {} StartSession '' {} {}",
        SESSION_MANAGER_BIN_NAME,
        resp_json.to_string(),
        prop.region,
        session_manager_param.to_string(),
        format!("https://ssm.{}.amazonaws.com", prop.region)
    );

    // Spawning Subprocess
    tokio::process::Command::new(SESSION_MANAGER_BIN_NAME)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .arg(resp_json.to_string())
        .arg(&prop.region)
        .arg("StartSession")
        .arg("")
        .arg(session_manager_param.to_string())
        .arg(format!("https://ssm.{}.amazonaws.com", prop.region))
        .spawn()?;

    //  subprocess ver.
    // let command = [
    // SESSION_MANAGER_BIN_NAME,
    // &resp_json.to_string(),
    // &prop.region,
    // "StartSession",
    // "",
    // &session_manager_param.to_string(),
    // &format!("https://ssm.{}.amazonaws.com", prop.region),
    // ];
    // let mut p = subprocess::Popen::create(&command, subprocess::PopenConfig {
    //     // stdin: subprocess::Redirection::Pipe,
    //     stdout: subprocess::Redirection::Pipe,
    //     // stdout: subprocess::Redirection::Merge,
    //     stdin: subprocess::Redirection::Pipe,
    //     ..Default::default()
    // })?;
    //
    // let mut p = subprocess::Popen::create(&command, subprocess::PopenConfig::default())?;
    //
    // Since we requested stdout to be redirected to a pipe, the parent's
    // end of the pipe is available as p.stdout.  It can either be read
    // directly, or processed using the communicate() method:
    // let (out, err) = p.communicate(None)?;
    // let _ = p.wait()?;
    //
    // check if the process is still alive
    // if let Some(_) = p.poll() {
    // the process has finished
    // } else {
    // it is still running, terminate it
    // p.terminate()?;
    // }
    //
    // let exit_status = subprocess::Exec::cmd(SESSION_MANAGER_BIN_NAME)
    //    .arg(resp_json.to_string())
    //    .arg(&prop.region)
    //    .arg("StartSession")
    //    .arg("")
    //    .arg(session_manager_param.to_string())
    //    .arg(format!("https://ssm.{}.amazonaws.com", prop.region))
    //    .join()?;
    // assert!(exit_status.success());

    Ok(())
}

// TODO: test
