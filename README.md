A very basic actix-web server that takes a chain id, a token and an Ethereum account and returns as JSON:

```rust
struct SignedBalance {
    message: Vec<u8>,
    signature: Vec<u8>,
    signer_address: Vec<u8>,
    token: Vec<u8>,
    owner: Vec<u8>,
    balance: U256,
    block: U64,
}
```

This can be used to verify an account's balance, in any system that trusts the signer.

To query it, use the following pattern in a GET call:

`http://localhost:8080/{chain_id}/{token_address}/{owner_account}`

### Deployment

Currently deployed to Digital Ocean at http://209.97.181.57/

Example for deploying to a Digital Ocean ubuntu box (not production grade):

```bash
# build for release
cargo build --release
# stop the running screen process
ssh root@209.97.181.57 "screen -X quit"
# copy the binary to the server
scp target/release/server-signer root@209.97.181.57:/opt/server-signer
# start a new screen session for the process
ssh root@209.97.181.57 "BIND_ADDRESS=0.0.0.0:80 screen -d -m ../opt/server-signer/server-signer"
```