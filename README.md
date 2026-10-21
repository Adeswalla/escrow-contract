# Milestone Escrow Contract

A Soroban smart contract on Stellar that enables trustless milestone-based escrow between a client, freelancer, and arbiter.

## Overview

This contract allows a client to fund a job broken into milestones. Funds are locked in the contract and released per milestone only when the client approves delivery. Disputes can be raised by either party and resolved by a designated arbiter.

## Trust Model

The escrow's protection depends on **four distinct roles** assigned at `initialize` time:

- **Admin** — the platform operator. Holds admin-only endpoints (whitelist management, pause/resume, admin overrides, upgrades). The admin is expected to be a neutral platform address, not the client or freelancer.
- **Client** — funds the job and approves milestone releases.
- **Freelancer** — delivers milestones and receives payouts.
- **Arbiter** — a neutral third party that resolves disputes via `resolve_dispute` and `apply_dispute_arbitration_split`.

`initialize` enforces party distinctness: it rejects any collision among `admin`, `client`, `freelancer`, and `arbiter` with `Error::InvalidParties`. This prevents the client from resolving their own disputes (arbiter == client), the freelancer from ruling in their own favour (arbiter == freelancer), or the client from holding every admin override (admin == client). The frontend must pass four genuinely distinct addresses — it must not reuse the connected wallet as both admin and client.

## Contract Functions

| Function | Caller | Description |
|---|---|---|
| `initialize` | Anyone | Set up job with milestone amounts and spec hashes, a 30-day dispute timeout and Upfront funding |
| `initialize_with_dispute_timeout` | Anyone | Set up job with a custom, non-zero dispute timeout |
| `initialize_with_funding_mode` | Anyone | Set up job with Upfront or PerMilestone funding |
| `fund` | Client | Deposit total amount into contract (Upfront mode) |
| `fund_milestone` | Client | Deposit one milestone (PerMilestone mode) |
| `mark_delivered` | Freelancer | Mark a milestone as delivered |
| `request_revision` | Client | Return a delivered milestone to Pending (max 3) |
| `approve_milestone` | Client | Release funds for a delivered milestone |
| `raise_dispute` | Client or Freelancer | Freeze a milestone for arbitration |
| `escalate_dispute` | Client or Freelancer | After the timeout, restore a disputed milestone to its previous status |
| `resolve_dispute` | Arbiter | Release to freelancer or refund to client |
| `get_job` | Anyone | View current job state |
| `sweep_excess` | Client or admin | Recover surplus tokens not backing obligations |
| `emergency_unpause_parties` | Client and freelancer | Jointly lift a party-initiated pause |

## Event topics

Each event keeps its current event name in topic[0]. The following addresses
are indexed in subsequent topics, in this order:

| Event | Topic layout |
|---|---|
| `init` | `init`, client, freelancer, arbiter |
| `fund` | `fund`, client, freelancer, arbiter |
| `deliver` | `deliver`, freelancer |
| `claim` | `claim`, freelancer |
| `approve` | `approve`, client |
| `dispute` | `dispute`, caller |

The backend's by-wallet query can use these topics to retrieve relevant escrow
events without scanning all event payloads. See the [backend by-wallet follow-up
issue](https://github.com/Goldii-locks/escrow-backend/issues/1).

## Milestone States

Pending → Delivered → Released

↓

Disputed → Released (arbiter favors freelancer)

→ Refunded (arbiter favors client)

The full state machine, including every entry point, its required signers and the escrow-level locks, is specified in [`contracts/milestone-escrow/SPEC.md`](contracts/milestone-escrow/SPEC.md).

## Security

Roles, trust assumptions, admin powers and known risks are described in [`THREAT_MODEL.md`](THREAT_MODEL.md).

## Prerequisites

- https://rustup.rs/ (the toolchain is pinned to 1.96.0 by `rust-toolchain.toml`)
- https://developers.stellar.org/docs/smart-contracts/getting-started/setup (stellar-cli 28.0.0)
- wasm32 target: `rustup target add wasm32v1-none`
- Docker, for reproducible builds

## Build

For development, from the repository root:

```bash
stellar contract build --manifest-path contracts/milestone-escrow/Cargo.toml
```

Anything that will be deployed must come from the reproducible build instead:

```bash
scripts/build-reproducible.sh
```

It compiles a clean export of `HEAD` (or of the commit given as the first
argument) inside a pinned container (`build/reproducible.Dockerfile`: Rust
1.96.0 by image digest, stellar-cli 28.0.0 by release checksum, linux/amd64)
and writes `dist/milestone_escrow.wasm`, its SHA-256 and `dist/build-info.json`.
The same commit gives the same hash on any machine. CI runs the same script on
every commit and publishes the hash in the job summary and the wasm as an
artifact.

## Test

From the repository root:

```bash
cargo test -p milestone-escrow
```

## Project Status

- Mainnet release gates are tracked in the [mainnet readiness checklist](MAINNET.md)
- The admin authorization direction and migration decision are recorded in [ADMIN_MULTISIG_DECISION.md](ADMIN_MULTISIG_DECISION.md)
- Contract implementation completed in `contracts/milestone-escrow`
- Includes milestone delivery, approval, dispute, and arbitration flows
- Initialize now rejects empty milestone lists, zero/negative amounts, zero addresses, zero auto-release windows, and **party collisions** (`admin == client`, `admin == freelancer`, `arbiter == client`, `arbiter == freelancer`, `client == freelancer`) with explicit contract errors
- `raise_dispute` now performs zero-address check, `require_auth`, and identity validation **before** writing the re-entrancy lock (issue #566), matching the authorization pattern used by `fund`, `approve_milestone`, and `resolve_dispute`; includes the five precondition guard assertions present in all other mutating functions
- Disputes may only be raised on delivered or partially released milestones. The default dispute timeout is 30 days; `initialize_with_dispute_timeout` allows a custom non-zero timeout. If neither party gets an arbitration decision by that deadline, either party may call `escalate_dispute` to return the milestone to its pre-dispute status; a restored delivered milestone gets a fresh auto-release window
- `lock_escrow_interest_yield` now rejects unauthorized callers with `Unauthorized` and an illegal source state with `InvalidStatus` **before** it reads or writes the ledger entry (issue #464); authorization runs first via `require_admin_from_instance`, so a non-admin cannot probe whether an escrow is locked, and an already-locked configuration is refused rather than silently re-locked
- Contract tests and snapshots are provided under `contracts/milestone-escrow/test_snapshots`

## Deploy (Testnet)

Configure `stellar-cli` with a funded testnet source account and set these
environment variables before running the script:

| Variable | Meaning |
|---|---|
| `SOURCE_ACCOUNT` | `stellar-cli` key name used to sign and pay for upload/deploy/initialize |
| `ADMIN_ADDRESS` | Stellar address that will be the escrow admin |
| `CLIENT_ADDRESS` | Client Stellar address |
| `FREELANCER_ADDRESS` | Freelancer Stellar address |
| `ARBITER_ADDRESS` | Arbiter Stellar address |
| `TOKEN_ADDRESS` | Token contract ID accepted by the escrow |
| `AUTO_RELEASE_SECONDS` | Positive auto-release interval in seconds |
| `MILESTONE_AMOUNTS` | JSON array of positive integer token amounts, for example `["10000000","25000000"]` |
| `MILESTONE_SPEC_HASHES` | JSON array of 32-byte hex hashes, one for each milestone, for example `["0123...cdef"]` (each value must be exactly 64 hex characters) |

The four party addresses must be valid and distinct. Amounts use the token's
smallest unit. `SOURCE_ACCOUNT` must be funded on testnet and available to
`stellar-cli`; the admin address can be a different account.

```bash
export SOURCE_ACCOUNT=testnet-deployer
export ADMIN_ADDRESS=G...
export CLIENT_ADDRESS=G...
export FREELANCER_ADDRESS=G...
export ARBITER_ADDRESS=G...
export TOKEN_ADDRESS=C...
export AUTO_RELEASE_SECONDS=2592000
export MILESTONE_AMOUNTS='["10000000"]'
export MILESTONE_SPEC_HASHES='["0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"]'
scripts/deploy-testnet.sh
```

The script builds with the pinned reproducible toolchain, uploads the wasm,
prints its on-chain hash, deploys and initializes the contract, then appends
the date, network, wasm hash, contract ID, and admin address to
`deployments/testnet.json`. Review and commit that deployment record after a
successful run. The detailed source commit and build tool versions remain in
`dist/build-info.json`.

After upgrading an existing deployment to this version, the admin should call
`migrate_cold_config` once to move the whitelist, multisig, yield and fee
configuration out of instance storage (issue #772). The contract keeps working
before that call; the migration only makes every later call cheaper.

## Verifying the deployed contract

Anyone can check which source a deployed contract was built from:

1. Read the installed wasm hash:

   ```bash
   stellar contract fetch --id <contract-id> --network testnet --out-file deployed.wasm
   sha256sum deployed.wasm
   ```

   (`stellar contract info` and the explorer show the same hash.)
2. Look up `source_commit` for that contract in `deployments/<network>.json`.
3. Rebuild that commit and compare:

   ```bash
   scripts/build-reproducible.sh <source_commit>
   cat dist/milestone_escrow.wasm.sha256
   ```

The two hashes must be identical. If they differ, the deployed code is not the
reviewed source at that commit.

## License

MIT

## Deployed Contract

| Network | Contract ID |
|---|---|
| Testnet | `CDD5WKK3WT3QVKXMXTJNDIXE4T73FK6GGXDSD6UTJAH6YYZU52SQ4MUH` |

Deployment records, including the installed wasm hash, are in
[`deployments/`](deployments/). The current testnet deployment predates
reproducible builds, so its source commit is unknown.

Explorer: `https://stellar.expert/explorer/testnet/contract/CDD5WKK3WT3QVKXMXTJNDIXE4T73FK6GGXDSD6UTJAH6YYZU52SQ4MUH`
