use aws_config::BehaviorVersion;
use aws_types::region::Region;
use futures_util::{future, pin_mut, StreamExt}; // TODO:
use tokio::io::{AsyncReadExt, AsyncWriteExt}; // TODO:
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message}; // TODO: // TODO:

#[derive(PartialEq, Debug)]
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

pub async fn start_session(prop: &SessionManagerProp) -> Result<(), aws_sdk_ssm::Error> {
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
                .parameters("host", vec![prop.remote_host.clone().unwrap().to_string()])
                .parameters("portNumber", vec![prop.remote_port.unwrap().to_string()])
                .parameters("localPortNumber", vec![prop
                    .local_port
                    .unwrap()
                    .to_string()])
                .send()
                .await?
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
                .await?
        },
        _ => {
            get_client(&prop.region)
                .await
                .start_session()
                .target(&prop.instance_id)
                .send()
                .await?
        },
    };

    log::info!("Session ID: {}", resp.session_id().unwrap());
    log::info!("Stream URL: {}", resp.stream_url().unwrap()); // TODO:
    log::info!("Token value: {}", resp.token_value().unwrap()); // TODO:

    let connect_addr = resp.stream_url().unwrap();
    let url = url::Url::parse(&connect_addr).unwrap();
    let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
    tokio::spawn(read_stdin(stdin_tx));
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (write, read) = ws_stream.split();

    let stdin_to_ws = stdin_rx.map(Ok).forward(write);
    let ws_to_stdout = {
        read.for_each(|message| async {
            let data = message.unwrap().into_data();
            tokio::io::stdout().write_all(&data).await.unwrap();
        })
    };

    pin_mut!(stdin_to_ws, ws_to_stdout);
    future::select(stdin_to_ws, ws_to_stdout).await;

    Ok(())
    // let resp = get_client(&prop.region).await.start_session().document_name(input)
}

// TODO:
// Our helper method which will read data from stdin and send it along the
// sender provided.
async fn read_stdin(tx: futures_channel::mpsc::UnboundedSender<Message>) {
    let mut stdin = tokio::io::stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);
        tx.unbounded_send(Message::binary(buf)).unwrap();
    }
}

// TODO: test
