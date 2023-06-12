use eyre::Result;
use image::Luma;
use qrcode::QrCode;

use ethers::abi::Address;
use ethers::core::types::transaction::eip2718::TypedTransaction;
//use ethers::ethers_providers::Middleware;
use defi_wallet_connect::v2::WCMiddleware;
use ethers::prelude::*;

use std::str::FromStr;

use defi_wallet_connect::v2::{Client, ClientOptions, Metadata, RequiredNamespaces, SessionInfo};
use std::error::Error;
use std::io::BufRead;

#[derive(Debug, Default)]
pub struct WalletConnectTxCommon {
    pub gas_limit: String,   // decimal string, "1"
    pub gas_price: String,   // decimal string
    pub nonce: String,       // decimal string
    pub chainid: u64,        // integer u64
    pub web3api_url: String, // string
}

/// wallet connect cronos(eth) eip155-tx signing info
#[derive(Debug, Default)]
pub struct WalletConnectTxEip155 {
    pub to: String,    // hexstring, "0x..."
    pub value: String, // decimal string, in wei units
    pub data: Vec<u8>, // data, as bytes

    pub common: WalletConnectTxCommon,
}

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

async fn sign_typed_tx(
    client: Client,
    tx: &TypedTransaction,
    address: Address,
) -> Result<Signature> {
    let middleware = WCMiddleware::new(client);
    let signature = middleware.sign_transaction(tx, address).await?;
    Ok(signature)
}

async fn send_typed_tx(client: Client, tx: TypedTransaction, address: Address) -> Result<TxHash> {
    let middleware = WCMiddleware::new(client).with_sender(address);
    let receipt = middleware.send_transaction(tx, None).await?.tx_hash();
    Ok(receipt)
}

pub async fn sign_eip155_transaction_blocking(
    client: &mut Client,
    userinfo: &WalletConnectTxEip155,
    address: [u8; 20],
) -> Result<Vec<u8>> {
    let signeraddress = Address::from_slice(&address);

    let mut tx = Eip1559TransactionRequest::new();

    if !userinfo.to.is_empty() {
        tx = tx.to(NameOrAddress::Address(Address::from_str(&userinfo.to)?));
    }
    if !userinfo.data.is_empty() {
        tx = tx.data(userinfo.data.as_slice().to_vec());
    }
    if !userinfo.common.gas_limit.is_empty() {
        tx = tx.gas(U256::from_dec_str(&userinfo.common.gas_limit)?);
    }
    if !userinfo.common.gas_price.is_empty() {
        tx = tx
            .max_priority_fee_per_gas(U256::from_dec_str(&userinfo.common.gas_price)?)
            .max_fee_per_gas(U256::from_dec_str(&userinfo.common.gas_price)?);
    }
    if !userinfo.common.nonce.is_empty() {
        tx = tx.nonce(U256::from_dec_str(&userinfo.common.nonce)?);
    }
    if userinfo.common.chainid != 0 {
        // tx = tx.chain_id(userinfo.common.chainid);
    }
    if !userinfo.value.is_empty() {
        tx = tx.value(U256::from_dec_str(&userinfo.value)?);
    }
    let newclient = client.clone();
    let typedtx = TypedTransaction::Eip1559(tx);

    let mut sig = sign_typed_tx(newclient, &typedtx, signeraddress)
        .await
        .map_err(|e| eyre::eyre!("sign_typed_transaction error {}", e.to_string()))?;

    // eip155 v == chainid*2 + 35 + recovery (0 or 1), for mainnet 37 or 38
    // non eip155 v == 27 + recovery (0 or 1)
    if sig.v == 27 || sig.v == 28 {
        let recovery = sig.v - 27;
        sig.v = recovery + 35 + userinfo.common.chainid * 2;
    }

    let signed_tx = &typedtx.rlp_signed(&sig);
    Ok(signed_tx.to_vec())
}

pub async fn send_eip155_transaction_blocking(
    client: &mut Client,
    userinfo: &WalletConnectTxEip155,
    address: [u8; 20],
) -> Result<Vec<u8>> {
    let signeraddress = Address::from_slice(&address);

    let mut tx = Eip1559TransactionRequest::new();

    if !userinfo.to.is_empty() {
        tx = tx.to(NameOrAddress::Address(Address::from_str(&userinfo.to)?));
    }
    if !userinfo.data.is_empty() {
        tx = tx.data(userinfo.data.as_slice().to_vec());
    }
    if !userinfo.common.gas_limit.is_empty() {
        tx = tx.gas(U256::from_dec_str(&userinfo.common.gas_limit)?);
    }
    if !userinfo.common.gas_price.is_empty() {
        tx = tx
            .max_priority_fee_per_gas(U256::from_dec_str(&userinfo.common.gas_price)?)
            .max_fee_per_gas(U256::from_dec_str(&userinfo.common.gas_price)?);
    }
    if !userinfo.common.nonce.is_empty() {
        tx = tx.nonce(U256::from_dec_str(&userinfo.common.nonce)?);
    }
    if userinfo.common.chainid != 0 {
        // tx = tx.chain_id(userinfo.common.chainid);
    }
    if !userinfo.value.is_empty() {
        tx = tx.value(U256::from_dec_str(&userinfo.value)?);
    }

    let newclient = client.clone();
    let typedtx = TypedTransaction::Eip1559(tx);

    // print typedtx
    println!("typedtx: {:?}", typedtx);
    println!("send tx: {:?}", typedtx);
    let tx_bytes = send_typed_tx(newclient, typedtx, signeraddress)
        .await
        .map_err(|e| eyre::eyre!("send_typed_transaction error {}", e.to_string()))?;

    //Ok(tx_bytes.0.to_vec())
    Ok(tx_bytes.0.to_vec())
}

async fn make_qrcode(uri: &str) -> Result<()> {
    // Generate the QR code for the data you want
    let code = QrCode::new(uri)?;

    // Create an empty image buffer
    let image = code.render::<Luma<u8>>().build();
    image.save("qrcode.png")?;

    Ok(())
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("walletconnect v2.0");
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

    let test_ping = false;
    let test_signing = false;
    let test_tx = true;
    let test_event_listening = false;

    let uri = client.get_connection_string().await;
    // make qrimage with uri
    make_qrcode(&uri).await?;

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

    if test_tx {
        // read env MYTOADDRESS
        let to = std::env::var("MYTOADDRESS").expect("MYTOADDRESS not set");
        let fromaddress = namespaces.get_ethereum_addresses()[0].address.clone();
        // print fromaddress
        println!("fromaddress= {:?}", fromaddress);
        let txinfo = WalletConnectTxEip155 {
            common: WalletConnectTxCommon {
                chainid: 5,
                gas_limit: "21000".into(),
                gas_price: "1000000000".into(),
                nonce: "0".into(),
                web3api_url: "".into(),
            },
            to: to.into(),
            data: vec![],
            value: "1000".into(),
        };
        let sig = sign_eip155_transaction_blocking(
            &mut client,
            &txinfo,
            (*fromaddress.as_fixed_bytes()).into(),
        )
        .await?;
        let sig_hex = hex::encode(sig.as_slice());
        let sig_hex_length = sig_hex.len();
        println!("signature length {sig_hex_length} 0x{sig_hex}");
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
