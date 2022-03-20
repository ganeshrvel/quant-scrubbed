# Quant | Blockchain auto-trading and sniping platform [rust]

- Author: [Ganesh Rathinavel](https://www.linkedin.com/in/ganeshrvel "Ganesh Rathinavel")
- License: [GPLv3](https://github.com/ganeshrvel/quant-scrubbed/blob/master/LICENSE "GPLv3")
- Repo URL: [https://github.com/ganeshrvel/quant-scrubbed](https://github.com/ganeshrvel/quant-scrubbed/ "https://github.com/ganeshrvel/quant-scrubbed")
- Contacts: ganeshrvel@outlook.com

# Introduction
Quant is an auto-trading and sniping platform built using the Rust language. It is capable of sniping a token at the very first block itself. 

Note: **This is a scrubbed version of my private repo, meaning: there will be no git history**.

### Be warned!
 - Stay within your spending limits, sniping a token is a very risky gamble. Remember, don't invest more than what you could afford to lose
 - DYOR, Don't invest your money into Honeypots. You know how it goes
 - Always start with a smaller amount

### Features
- Snipe at the very first block itself
- Separate flows for Buying, Selling and Buy & Dump
- CLI to choose between the trading modes
- Time limits
- Time delays
- Auto approve the token
- Auto trade after a time
- Auto trade after a value
- Stop trade after a time limit
- Stop trade after a percentage of value increase
- Stop trade after a pre-set number fo retries
- Increase the gas limits after each attempt
- Automatically calculate the minimum amount of tokens required to execute a trade, which includes the gas and number of attempts.
- Insert in a token at the T0th second of the IDO, via CLI

## Building from Source

Requirements: [rust lang](https://www.rust-lang.org/tools/install "Install rust")

### Clone

```shell
$ git clone https://github.com/ganeshrvel/quant-scrubbed.git quant

$ cd quant
```

### Run

```shell
# debug mode
$  cargo run

# release mode
$  cargo run --release

# Run
$ ./quant
```

### Build

```shell
$ cargo build --release
$ ./target/release/quant
```

### The scripts directory
- Find the scripts for all sorts of actions and trades in the `./scripts` directory

### Tools YAML to Rust classes converter
  - Use this bundled tool to generate Rust classes from YAML
```shell
# install
$ cd tools/yaml_converter
$ yarn
$ cd ../..
```

  - Run
```shell
$ node ./tools/yaml_converter --inputfile=sample.config.yaml
$ node ./tools/yaml_converter --inputfile=sample.secrets.yaml
```
  - Read the [Readme](https://github.com/ganeshrvel/quant-scrubbed/blob/main/LICENSE) file of the yaml_converter tool for more

### Configuration

**Secret**
  - Copy and rename **sample.secrets.yaml** as **secrets.yaml**
  - *secrets.yaml* contains the wallets' SECRET information.
  - **DO NOT COMMIT THIS FILE**

**Config**
  - Copy and rename **sample.config.yaml** as **config.yaml**
  - *config.yaml* contains the config information for buy/sell
  - **DO NOT COMMIT THIS FILE**
  
### Distribution
  - Copy `./target/release/quant` to a directory
  - Copy `secrets.yaml` to the same directory as above
  - Copy `config.yaml` to the same directory as above
  - Make sure that `secrets.yaml` isn't accessible to the others
    
**OR** simple run `./scripts/build-release.sh`

### Warranty
Read the [license](https://github.com/ganeshrvel/quant-scrubbed/blob/master/LICENSE "GPLv3 License") carefully. The license makes it clear that the project is offered "as-is", without warranty, and disclaiming liability for damages resulting from using this project.

### Contribute
If you are interested in fixing issues and contributing directly to the code base.

### Contacts
Please feel free to contact me at ganeshrvel@outlook.com or [LinkedIn](https://www.linkedin.com/in/ganeshrvel)

### Support
Help me keep my works FREE and open for all.
- Donate Via PayPal: [paypal.me/ganeshrvel](https://paypal.me/ganeshrvel "https://paypal.me/ganeshrvel")
- Buy Me A Coffee (UPI, PayPal, Credit/Debit Cards, Internet Banking): [buymeacoffee.com/ganeshrvel](https://buymeacoffee.com/ganeshrvel "https://buymeacoffee.com/ganeshrvel")

### License
Quant | Blockchain autotrading and sniping platfrorm [rust] is released under [GPLv3 License](https://github.com/ganeshrvel/sirius-proxima/blob/master/LICENSE "GPLv3 License").

Copyright © 2018-Present Ganesh Rathinavel
