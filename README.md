# breez-sdk-liquid-swap-lnurl


## Prerequisites

This assumes the instance that runs this is accessible via a public domain.


## Setup

Setup `breez-sdk-liquid`:

```bash
# Clone specific branch
git clone -b ok300-consolidate-rev-swap-onchain-amt --single-branch https://github.com/breez/breez-sdk-liquid

# Create a new wallet
cd breez-sdk-liquid/cli
cargo run
# Exit with Ctrl+C

# Print the generated mnemonic, copy it for later
cat .data/phrase ; echo
```

Next to `breez-sdk-liquid`, setup this repo:

```bash
cd ../..
git clone https://github.com/ok300/breez-sdk-liquid-swap-lnurl
cd breez-sdk-liquid-swap-lnurl
```

Create `config.toml` with

```toml
mnemonic = "..."
domain = "..."

ls_sdk_data_dir = "ls-sdk-data-dir"

# They seem to be identical between testnet and mainnet
# https://testnet.boltz.exchange/api/getpairs
# https://api.boltz.exchange/getpairs
min_sendable_msat = 1_000_000
max_sendable_msat = 25_000_000_000
```

Set `domain` to the public domain under which this instance is running.

Set `mnemonic` to the mnemonic copied earlier from the CLI.

Start it:

```bash
ROCKET_PORT=443 cargo run
```
