use std::error::Error;

use defi_wallet_connect::session::SessionInfo;
use defi_wallet_connect::{Client, Metadata, WCMiddleware};
use defi_wallet_connect::{ClientChannelMessage, ClientChannelMessageType};
use ethers::prelude::Middleware;
use ethers::types::H160;
use eyre::eyre;
use std::fs::File;
use std::io::prelude::*;
use url::form_urlencoded;

/// remove session.json to start new session
const G_FILENAME: &str = "sessioninfo.json";

///  temporary session is stored to session.json
async fn make_client() -> Result<Client, Box<dyn Error>> {
    let filename = G_FILENAME;
    if let Ok(mut file) = File::open(filename) {
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let session: SessionInfo = serde_json::from_str(&contents)?;
        let client = Client::restore(session).await?;
        println!("restored client");
        Ok(client)
    } else {
        let client = Client::new(
            Metadata {
                description: "Defi WalletConnect example.".into(),
                url: "http://localhost:8080/".parse().expect("url"),
                icons: vec![],
                name: "Defi WalletConnect Web3 Example".into(),
            },
            // Some(25), // cronos mainnet
            None, // up to wallet
        )
        .await?;
        println!("created client");
        Ok(client)
    }
}

fn write_session_to_file(info: &SessionInfo, filename: &str) -> eyre::Result<()> {
    let mut file = std::fs::File::create(filename)?;
    let buffer = serde_json::to_string(&info)?;
    // write buffer to file
    file.write_all(buffer.as_bytes())?;
    Ok(())
}

async fn eth_sign(client: Client, address: Vec<H160>) -> Result<(), Box<dyn Error>> {
    let middleware = WCMiddleware::new(client);
    // Note that `sign` on ethers middleware translate to `eth_sign` JSON-RPC method
    // which in Metamask docs is marked as "(insecure and unadvised to use)"
    // and some wallets may reject it
    let sig2 = middleware
        .sign("world".as_bytes().to_vec(), &address[0])
        .await;
    match sig2 {
        Ok(value) => println!("sig2: {:?}", value),
        Err(_error) => println!("not erorr, eth_sign not supported in the wallet"),
    }
    Ok(())
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let filename = G_FILENAME;

    let mut client = make_client().await?;

    client
        .run_callback(Box::new(
            move |message: ClientChannelMessage| -> eyre::Result<()> {
                match message.state {
                    ClientChannelMessageType::Connected => {
                        println!("Connected");
                        if let Some(info) = message.session {
                            println!("session info: {:?}", info);
                            write_session_to_file(&info, filename)
                        } else {
                            Err(eyre!("no session info"))
                        }
                    }
                    ClientChannelMessageType::Disconnected => {
                        println!("Disconnected");
                        if let Some(info) = message.session {
                            println!("session info: {:?}", info);
                            Ok(())
                        } else {
                            Err(eyre!("no session info"))
                        }
                    }
                    ClientChannelMessageType::Connecting => {
                        println!("Connecting");
                        if let Some(info) = &message.session {
                            info.uri().print_qr_uri();
                            write_session_to_file(info, filename)
                        } else {
                            Err(eyre!("no session info"))
                        }
                    }
                    ClientChannelMessageType::Updated => {
                        println!("Updated");
                        if let Some(info) = &message.session {
                            write_session_to_file(info, filename)
                        } else {
                            Err(eyre!("no session info"))
                        }
                    }
                }
            },
        ))
        .await?;

    // qrcode display
    let uri = client.get_connection_string().await?;
    println!("connection string = {}", uri);

    let encoded: String = form_urlencoded::Serializer::new(String::new())
        .append_pair("uri", &uri)
        .finish();

    println!("Crypto.com Wallet: cryptowallet://wc?{}", encoded);

    let (address, chain_id) = client.ensure_session().await?;
    println!("address: {:?}", address);
    println!("chain_id: {}", chain_id);

    // personal_sign is signing with document
    let sig1 = client.personal_sign("Hello World", &address[0]).await?;
    println!("sig1: {:?}", sig1);

    // eth_sign  is signing directly with hash of message
    // because it's not secure and not recommended to use it
    // metamask and etc. will reject it, so that is not an error
    eth_sign(client, address).await?;
    Ok(())
}
