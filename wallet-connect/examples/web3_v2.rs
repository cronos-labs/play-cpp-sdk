use defi_wallet_connect::v2::{Client, ClientOptions, Metadata, RequiredNamespaces, SessionInfo};
use std::error::Error;
use std::io::BufRead;
async fn make_client(
    callback_sender: Option<tokio::sync::mpsc::UnboundedSender<String>>,
) -> Result<Client, relay_client::Error> {
    let opts = ClientOptions {
        relay_server: "wss://relay.walletconnect.com".parse().expect("url"),
        project_id: std::env::args().skip(1).next().expect("project_id"),
        required_namespaces: RequiredNamespaces::new(
            vec![
                "eth_sendTransaction".to_owned(),
                "eth_signTransaction".to_owned(),
                "eth_sign".to_owned(),
                "personal_sign".to_owned(),
                "eth_signTypedData".to_owned(),
            ],
            vec!["eip155:5".to_owned()],
            vec!["chainChanged".to_owned(), "accountsChanged".to_owned()],
        ),
        client_meta: Metadata {
            description: "Defi WalletConnect v2 example.".into(),
            url: "http://localhost:8080/".parse().expect("url"),
            icons: vec![],
            name: "Defi WalletConnect Web3 Example".into(),
        },
        callback_sender,
    };

    Client::new(opts).await
}

async fn load() -> eyre::Result<SessionInfo> {
    let file_path = "session.bin";
    let contents = tokio::fs::read(file_path).await?;
    let session_info: SessionInfo = bincode::deserialize(&contents)?;
    Ok(session_info)
}
async fn save(info: &SessionInfo) -> eyre::Result<()> {
    let binary_data = bincode::serialize(&info)?;
    tokio::fs::write("session.bin", binary_data).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            println!("callback: {}", msg);
        }
    });
    let callback_sender = Some(tx);

    // TODO: qrcode display
    // if session.json exists
    let mut client: Client = if let Ok(session_info) = load().await {
        Client::restore(session_info, callback_sender).await?
    } else {
        make_client(callback_sender).await?
    };

    let test_ping = true;
    let test_signing = true;
    let test_event_listening = false;

    let uri = client.get_connection_string().await;
    println!("uri= {}", uri);
    let namespaces = client.ensure_session().await?;
    println!(
        "namespaces= {}",
        serde_json::to_string(&namespaces).expect("convert json")
    );
    let sessioninfo = client.get_session_info().await;
    save(&sessioninfo).await?;
    if test_ping {
        let response = client.send_ping().await?;
        println!("ping response= {}", response);
    }
    if test_signing {
        // 0xaddress
        let address1 = namespaces.get_ethereum_addresses()[0].address.clone();
        let sig1 = client.personal_sign("Hello Crypto", &address1).await?;
        println!("sig1: {:?}", sig1);
    }

    if test_event_listening {
        println!("press anykey to exit");
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            let stdin = std::io::stdin();
            let stdin_lock = stdin.lock();
            if let Some(_line) = stdin_lock.lines().next() {
                break;
            }
        }
    }

    Ok(())
}
