use std::error::Error;

use defi_wallet_connect::v2::{Client, ClientOptions, Metadata, RequiredNamespaces};

async fn make_client() -> Result<Client, relay_client::Error> {
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
    };

    Client::new(opts).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // TODO: qrcode display
    let mut client = make_client().await?;
    let uri = client.get_connection_string().await;
    println!("connection string = {}", uri);
    let namespaces = client.ensure_session().await?;
    println!("namespaces = {:?}", namespaces);
    let address1 = namespaces.get_ethereum_addresses()[0].address;
    let sig1 = client.personal_sign("Hello Crypto", &address1).await?;
    println!("sig1: {:?}", sig1);
    Ok(())
}
