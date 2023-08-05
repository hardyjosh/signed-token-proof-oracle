use actix_web::{get, web, App, HttpServer};
use ethers::prelude::*;
use std::{sync::Arc, env};
use serde::Serialize;
use actix_cors::Cors;

#[derive(Serialize)]
struct SignedBalance {
    message: Vec<u8>,
    signature: Vec<u8>,
    signer_address: Vec<u8>,
    token: Vec<u8>,
    owner: Vec<u8>,
    balance: U256,
    block: U64,
}

#[get("{_chain}/{_token}/{_owner}")]
async fn hello(path: web::Path<(u32, String, String)>) -> web::Json<SignedBalance> {
    let (_chain, _token, _owner) = path.into_inner();
    let token: Address = _token.parse().ok().unwrap();
    let owner: Address = _owner.parse().ok().unwrap();
    let provider = get_provider(_chain).unwrap();

    let balance: U256 = get_balance_of(provider.clone(), token, owner).await.unwrap();
    let block_number: U64 = provider.get_block_number().await.unwrap();
    let signed_balance: SignedBalance = produce_signed_balance(token, owner, balance, block_number).await.unwrap();

    web::Json(signed_balance)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let address = env::var("BIND_ADDRESS")
        .unwrap_or_else(|_err| "localhost:8080".to_string());

    HttpServer::new(|| {
        let cors = Cors::default().allow_any_origin().send_wildcard();
        App::new()
            .wrap(cors)
            .service(hello)
    })
    .bind(address)?
    .run()
    .await
}

fn get_provider(chain: u32) -> Result<Provider::<Http>, Box<dyn std::error::Error>> {
    let rpc_url = match chain {
        1 => "https://eth.llamarpc.com",
        5 => "https://goerli-rpc.linkpool.io",
        137 => "hhttps://polygon.llamarpc.com",
        80001 => "https://rpc-mumbai.maticvigil.com",
        _ => {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Invalid chain")));
        }
    };

    if let Ok(provider) = Provider::<Http>::try_from(
        rpc_url
    ) {
        return Ok(provider);
    } else {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Couldn't parse provider")));
    }
}

async fn get_balance_of(provider: Provider::<Http>, token: Address, owner: Address) -> Result<U256, Box<dyn std::error::Error>>{
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

    let client = Arc::new(provider);
    let contract = IERC20::new(token, client);

    if let Ok(balance) = contract.balance_of(owner).call().await {
        return Ok(balance);
    } else {
        // return an error here
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "")));
    }
}

async fn produce_signed_balance(token: Address, owner: Address, balance: U256, block: U64) -> Result<SignedBalance, Box<dyn std::error::Error>>{
    let wallet: LocalWallet = "380eb0f3d505f087e438eca80bc4df9a7faa24f868e69fc0440261a0fc0567dc"
    .parse()?;

    // Converting balance into [u8; 32]
    let mut balance_bytes = [0u8; 32];
    balance.to_big_endian(&mut balance_bytes);

    // Converting block into [u8; 8]
    let mut block_bytes = [0u8; 8];
    block.to_big_endian(&mut block_bytes);

    // Converting token into [u8; 20]
    let token_bytes= token.as_bytes();

    // Converting owner into [u8; 20]
    let owner_bytes= owner.as_bytes();

    // Concatenating all together
    let mut result: Vec<u8> = Vec::with_capacity(80); // Pre-allocating exact size
    result.extend_from_slice(&token_bytes);
    result.extend_from_slice(&owner_bytes);
    result.extend_from_slice(&balance_bytes);
    result.extend_from_slice(&block_bytes);


    let hash = ethers::utils::keccak256(result.as_slice());

    let signature = wallet.sign_message(hash).await.ok().unwrap();
    let signature_bytes = signature.to_vec();

    return Ok(SignedBalance {
        message: result,
        token: token.as_bytes().to_vec(),
        owner: owner.as_bytes().to_vec(),
        block: block,
        signature: signature_bytes,
        signer_address: wallet.address().as_bytes().to_vec(),
        balance: balance,
    });
}