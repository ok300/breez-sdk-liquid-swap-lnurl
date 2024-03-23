# breez-sdk-liquid-swap-lnurl


## Prerequisites

This assumes the instance that runs this is accessible via a public domain.


## Setup

Setup `breez-sdk-liquid`:

```bash
# Clone specific branch
git clone -b ok300-consolidate-rev-swap-onchain-amt --single-branch https://github.com/breez/breez-sdk-liquid
```

Next to `breez-sdk-liquid`, setup this repo:

```bash
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

Set `mnemonic` to a 12 or 24 word mnemonic.

Start it:

```bash
cargo run
```

and use a reverse proxy to map the local port 8000 to the public port 443.