use std::collections::HashMap;

use serde_json::json;

const SESSION_MANAGER_BIN_NAME: &str = "session-manager-plugin";

#[derive(PartialEq, Debug, Clone, Copy)]
enum SessionMode {
    Direct,
    PortForwarding,
    PortForwardingToRemoteHost,
}

impl SessionMode {
    fn get_document_name(&self) -> Option<String> {
        match self {
            Self::Direct => None,
            Self::PortForwarding => Some(String::from("AWS-StartPortForwardingSession")),
            Self::PortForwardingToRemoteHost => {
                Some(String::from("AWS-StartPortForwardingSessionToRemoteHost"))
            },
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
    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(aws_types::region::Region::new(region.to_owned()))
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
        SessionMode::PortForwardingToRemoteHost
    } else if prop.local_port.is_some() && prop.remote_port.is_some() {
        SessionMode::PortForwarding
    } else {
        SessionMode::Direct
    };

    log::info!("Document name: {}", match mode.get_document_name() {
        Some(val) => val,
        None => "None".to_string(),
    });

    let document_name = match mode {
        SessionMode::Direct => SessionMode::Direct.get_document_name(),
        SessionMode::PortForwardingToRemoteHost => {
            SessionMode::PortForwardingToRemoteHost.get_document_name()
        },
        SessionMode::PortForwarding => SessionMode::PortForwarding.get_document_name(),
    };

    let mut parameters = HashMap::new();
    match mode {
        SessionMode::Direct => (),
        SessionMode::PortForwarding => {
            parameters.insert("portNumber".to_string(), vec![prop
                .remote_port
                .unwrap()
                .to_string()]);
            parameters.insert("localPortNumber".to_string(), vec![prop
                .local_port
                .unwrap()
                .to_string()]);
        },
        SessionMode::PortForwardingToRemoteHost => {
            parameters.insert("host".to_string(), vec![prop
                .remote_host
                .clone()
                .unwrap()
                .to_string()]);
            parameters.insert("portNumber".to_string(), vec![prop
                .remote_port
                .unwrap()
                .to_string()]);
            parameters.insert("localPortNumber".to_string(), vec![prop
                .local_port
                .unwrap()
                .to_string()]);
        },
    };

    let resp = get_client(&prop.region)
        .await
        .start_session()
        .target(&prop.instance_id)
        .set_document_name(document_name)
        .set_parameters(if parameters.is_empty() {
            None
        } else {
            Some(parameters)
        })
        .send()
        .await;

    let resp_json: serde_json::Value;
    match resp {
        Ok(o) => {
            log::info!("Session ID: {}", o.session_id().unwrap());
            resp_json = json!({
                "SessionId": o.session_id().unwrap(),
                "TokenValue": o.token_value().unwrap(),
                "StreamUrl": o.stream_url().unwrap(),
            });
        },
        Err(e) => return Err(e.into()),
    }

    let session_manager_param = match mode {
        SessionMode::Direct => json!({"Target" : &prop.instance_id}),
        SessionMode::PortForwarding => json!({
            "Target" : &prop.instance_id,
            "DocumentName": SessionMode::PortForwarding.get_document_name(),
            "parameters": {
                "portNumber": vec![prop.remote_port.unwrap().to_string()],
                "localPortNumber": vec![prop.local_port.unwrap().to_string()]
            }
        }),
        SessionMode::PortForwardingToRemoteHost => json!({
            "Target" : &prop.instance_id,
            "DocumentName": SessionMode::PortForwardingToRemoteHost.get_document_name(),
            "parameters": {
                "host": vec![prop.remote_host.clone().unwrap().to_string()],
                "portNumber": vec![prop.remote_port.unwrap().to_string()],
                "localPortNumber": vec![prop.local_port.unwrap().to_string()]
            }
        }),
    };

    tokio::spawn(async move {
        // Listen in the background
        tokio::signal::ctrl_c().await.unwrap();
    });
    let exit_status = subprocess::Exec::cmd(SESSION_MANAGER_BIN_NAME)
        .arg(resp_json.to_string())
        .arg(&prop.region)
        .arg("StartSession")
        .arg("")
        .arg(session_manager_param.to_string())
        .arg(format!("https://ssm.{}.amazonaws.com", prop.region))
        .join()?;
    assert!(exit_status.success());

    Ok(())
}

// TODO: test
