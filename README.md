# Dassi Prototype Program for Solana India Hackathon Submission


### Environment Setup
1. Install Rust from https://rustup.rs/
2. Install Solana v1.6.2 or later from https://docs.solana.com/cli/install-solana-cli-tools#use-solanas-install-tool




### For deployment of program 

### Check for rust and cargo installation 
```
$ rustup --version
$ cargo --version
```

### For testing set cluster to devnet
```
$ solana config set --url https://api.devnet.solana.com
```

### To get current cluster
```
$ solana config get
```

### Ensure Versions Match By
```
$ solana --version 
$ solana cluster-version
```

### Generate a new keypair if you don't have one
```
$ mkdir ~/my-solana-wallet
$ solana-keygen new --outfile ~/my-solana-wallet/my_keypair_for_program_deployment.json
```

### Never share my_keypair_for_program_deployment.json with anyone as it contains your private key

### Check Publickey of generated wallet
```
$ solana-keygen pubkey ~/my-solana-wallet/my_keypair_for_program_deployment.json
```

### Verify the address against keypair file if you want
```
$ solana-keygen verify <PUBKEY> ~/my-solana-wallet/my_keypair_for_program_deployment.json
```

### Airdrop yourself
```
$ solana airdrop 1 <RECIPIENT_ACCOUNT_ADDRESS> --url https://api.devnet.solana.com
```

### Check Your Balance 
```
$ solana balance <ACCOUNT_ADDRESS> --url https://api.devnet.solana.com
```

### Build the program, after going in project folder (Million_Dollar_Picture)
```
$ cargo build-bpf 
```

### Deploy the prograrm
```
$ solana program deploy --max-len 200000 <PROGRAM_FILEPATH>
```

### Read more from here about program deployment: https://docs.solana.com/cli/deploy-a-program


### Build and test for program compiled natively
```
$ cargo build
$ cargo test
```

### Build and test the program compiled for BPF
```
$ cargo build-bpf
$ cargo test-bpf
```