use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use ethers::prelude::*;
use std::{sync::Arc, error::Error, f32::consts::E};
use serde::Serialize;
use ethers::utils::hex::ToHex;
use actix_cors::Cors;

const RPC_URL: &str = "https://eth.llamarpc.com";

#[derive(Serialize)]
struct SignedBalance {
    signature: Vec<u8>,
    signer_address: Vec<u8>,
    balance: U256,
}

#[get("/{_token}/{_owner}")]
async fn hello(path: web::Path<(String, String)>) -> web::Json<SignedBalance> {
    // let block_number: U64 = get_block_number().await.unwrap();
    // let token: Address = "0x6c6EE5e31d828De241282B9606C8e98Ea48526E2".parse().ok().unwrap();
    let (_token, _owner) = path.into_inner();
    let token: Address = _token.parse().ok().unwrap();
    let owner: Address = _owner.parse().ok().unwrap();
    // let owner: Address = "0x87b11cB8bf0052f1C9F6780D56102E4a1779db41".parse().ok().unwrap();
    let balance: U256 = get_balance_of(token, owner).await.unwrap();
    let signed_balance: SignedBalance = produce_signed_balance(balance).await.unwrap();
    web::Json(signed_balance)
    // HttpResponse::Ok().body(balance.to_string())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let cors = Cors::default().allow_any_origin().send_wildcard();
        App::new()
            .wrap(cors)
            .service(hello)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn get_balance_of(token: Address, owner: Address) -> Result<U256, Box<dyn std::error::Error>>{
    // The abigen! macro expands the contract's code in the current scope
    // so that you can interface your Rust program with the blockchain
    // counterpart of the contract.
    abigen!(
        IERC20,
        r#"[
            function totalSupply() external view returns (uint256)
            function balanceOf(address account) external view returns (uint256)
            function transfer(address recipient, uint256 amount) external returns (bool)
            function allowance(address owner, address spender) external view returns (uint256)
            function approve(address spender, uint256 amount) external returns (bool)
            function transferFrom( address sender, address recipient, uint256 amount) external returns (bool)
            event Transfer(address indexed from, address indexed to, uint256 value)
            event Approval(address indexed owner, address indexed spender, uint256 value)
        ]"#,
    );

    const RPC_URL: &str = "https://eth.llamarpc.com";

    let provider = Provider::<Http>::try_from(RPC_URL)?;
    let client = Arc::new(provider);
    let contract = IERC20::new(token, client);

    if let Ok(balance) = contract.balance_of(owner).call().await {
        return Ok(balance);
    } else {
        // return an error here
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "")));
    }
    // return contract.total_supply().call().await;
}

async fn produce_signed_balance(balance: U256) -> Result<SignedBalance, Box<dyn std::error::Error>>{
    let wallet: LocalWallet = "380eb0f3d505f087e438eca80bc4df9a7faa24f868e69fc0440261a0fc0567dc"
    .parse()?;
    let mut bytes = [0u8; 32];
    balance.to_big_endian(&mut bytes);

    let hash = ethers::utils::keccak256(&bytes);

    let signature = wallet.sign_message(hash).await.ok().unwrap();
    let signature_bytes = signature.to_vec();

    return Ok(SignedBalance {
        signature: signature_bytes,
        signer_address: wallet.address().as_bytes().to_vec(),
        balance: balance,
    });
}