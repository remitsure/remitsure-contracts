<p align="center"><img src="docs/logo.svg" width="96" alt="RemitSure logo"></p>

# remitsure-contracts

Soroban smart contracts for **RemitSure**: send money home for a purpose, and be sure it is used for it.

People abroad send money home for school fees, rent and hospital bills, and too often it never reaches the bill it was sent for. RemitSure locks the payment for one named payee. Only that school, landlord or hospital can claim it. If they do not claim it in time, the sender gets it back. Every step leaves an on chain receipt.

## Repos

| Repo                                        | What it is                                          |
| ------------------------------------------- | --------------------------------------------------- |
| **remitsure-contracts** (this repo)         | Soroban contract that locks, releases and refunds   |
| [remitsure-backend](../remitsure-backend)   | API, event indexer, payee directory                 |
| [remitsure-frontend](../remitsure-frontend) | Next.js app with Stellar wallet                     |

## Contract API

| Function         | Who     | What                                                 |
| ---------------- | ------- | ---------------------------------------------------- |
| `register_payee` | payee   | Join the payee directory                             |
| `set_verified`   | admin   | Add or remove the verified badge                     |
| `send`           | sender  | Lock funds for a payee with a reference and deadline |
| `claim`          | payee   | Collect before the deadline                          |
| `reject`         | payee   | Turn it down, sender refunded at once                |
| `refund`         | sender  | Take back unclaimed funds after the deadline         |
| `get_remittance` | anyone  | Full record, the receipt                             |
| `get_payee`      | anyone  | Payee profile                                        |

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the state machine, permissions, storage and events.

## Quick start

```bash
rustup target add wasm32v1-none
make test     # unit tests
make build    # wasm in target/wasm32v1-none/release
make check    # everything CI runs
```

Deploy to testnet (creates and funds an admin key on first run):

```bash
make deploy-testnet
```

## Testnet deployment

| | |
| --- | --- |
| Contract | [`CBMGPXSBSDKU55QGZTILUOZLUB5IBVBQ2YLJPXDYYCR6CXV42W7LABKF`](https://stellar.expert/explorer/testnet/contract/CBMGPXSBSDKU55QGZTILUOZLUB5IBVBQ2YLJPXDYYCR6CXV42W7LABKF) |
| Network | Stellar testnet |

## Status

MVP on testnet. Not audited. See [SECURITY.md](SECURITY.md).

## License

MIT
