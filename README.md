A very basic actix-web server that takes a token and an Ethereum account and returns as JSON:

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

`http://localhost:8080/{token_address}/{owner_account}`

### Deployment

Currently deployed to Digital Ocean at http://209.97.181.57/
