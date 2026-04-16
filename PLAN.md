# Implementation Plan — Козацький бізнес (Solana)

Learning-oriented, step-by-step plan. Each step introduces one new concept and ends with a runnable, testable milestone. Do not skip ahead — each step's tests should pass before moving on.

---

## Step 0 — Tooling & "hello world"

**What we do:** Install Rust, Solana CLI, Anchor (via `avm`), Node/pnpm. Run `solana-test-validator` locally. Scaffold a throwaway `anchor init hello` project just to see the generated layout.

**What you learn:**
- What `.so` files are and where `anchor build` puts them.
- The four moving parts: validator (blockchain), CLI (wallet + RPC), Anchor (framework), TypeScript client (tests/scripts).
- How `anchor test` spins up localnet, deploys, runs tests, tears down.

**Exit criteria:** `anchor test` passes on the default template.

---

## Step 1 — Workspace scaffolding

**What we do:** Create the real workspace with all 6 programs as empty crates: `resource_manager`, `search`, `crafting`, `item_nft`, `magic_token`, `marketplace`. Wire up `Anchor.toml` and the workspace `Cargo.toml`.

**What you learn:**
- Anchor multi-program workspace layout.
- How programs get unique Program IDs and how they reference each other.
- The `declare_id!` macro and what regenerating program keys means.

**Exit criteria:** `anchor build` compiles all 6 programs to `.so`. Nothing deployed yet.

---

## Step 2 — `GameConfig` init (first real program)

**What we do:** Implement `initialize_game_config` in `resource_manager`. It creates a PDA at seeds `["game_config"]` storing the admin pubkey.

**What you learn:**
- Anchor's `#[derive(Accounts)]` macro — the heart of the framework.
- The `init` constraint, `seeds`, `bump`, `payer`, `space` — how accounts are born.
- PDA derivation on Rust side and TypeScript client side.
- Signing a transaction, paying rent, inspecting account data after.

**Exit criteria:** TS test calls the instruction, fetches the account, asserts admin pubkey matches. Negative test: calling twice fails.

---

## Step 3 — Resource mints (Token-2022 + MetadataPointer)

**What we do:** Add `initialize_resource_mints` to `resource_manager`. Creates all 6 SPL Token-2022 mints with `MetadataPointer` extension, sets mint authority to a PDA like `["mint_authority", resource_id]`. Store mint pubkeys in `GameConfig`.

**What you learn:**
- First CPI — calling the Token-2022 program from your program.
- Token-2022 extensions (MetadataPointer) and account sizing.
- **Setting mint authority to a PDA your program controls** — the security spine.

**Exit criteria:** 6 mints exist, decimals=0, mint authority == expected PDA. Negative: direct mint with a wallet key fails.

---

## Step 4 — `Player` PDA + Search with timer

**What we do:** In `search`: `initialize_player` (creates Player PDA at `["player", wallet]`), then `search_resources` which reads `Clock`, enforces 60s cooldown, generates 3 resource IDs (pseudo-random via `Clock.slot` + `recent_blockhashes`), CPIs into `resource_manager` to mint them, updates `last_search_timestamp`.

**What you learn:**
- On-chain time via `Clock` sysvar.
- **Program-to-program CPI with PDA signer.**
- Associated Token Accounts (ATAs) — where player balances live.
- Why true on-chain randomness is hard; pragmatic caveats.

**Exit criteria:** First search succeeds, balances up by 3. Second search within 60s fails. After clock bump, succeeds again.

---

## Step 5 — Item NFTs via Metaplex

**What we do:** `item_nft::mint_item_nft` — creates a fresh mint (supply=1, decimals=0), creates Metaplex metadata account, mints 1 to recipient, disables mint authority. Also `burn_item_nft` gated so only `marketplace` can call.

**What you learn:**
- Metaplex Token Metadata CPI.
- Why disabling mint authority makes it a true NFT.
- Cross-program authority verification.

**Exit criteria:** NFT mint with correct metadata, supply=1, mint authority `None`. Non-marketplace burn attempt fails.

---

## Step 6 — Crafting closes the resource → NFT loop

**What we do:** `crafting::craft_item(item_type)` — validate recipe, CPI-burn resources, CPI-mint NFT, create `ItemMetadata` PDA.

**What you learn:**
- Multi-program CPI chain in a single instruction — transactional atomicity.
- Recipe validation patterns.
- Economic conservation in action.

**Exit criteria:** With enough resources, craft succeeds; balances drop, NFT appears. Insufficient resources → whole tx reverts, no partial state.

---

## Step 7 — `magic_token` + `marketplace` closes the item → reward loop

**What we do:** `magic_token` exposes a gated `mint_to_seller` callable only by `marketplace` PDA. `marketplace::sell_item` verifies ownership, CPI-burns NFT, CPI-mints MagicToken to seller.

**What you learn:**
- Full 4-program dance in one tx.
- Gated cross-program mint — the hardest security pattern.
- Self-balancing economy.

**Exit criteria:** Happy path yields MagicToken and destroys NFT. Direct `magic_token::mint_to_seller` call fails.

---

## Step 8 — Security audit & coverage sweep

**What we do:** Add negative tests for every instruction: wrong owner, direct Token-2022 mint attempts on every mint, wrong PDA seeds, wrong bump, wrong signer combos.

**What you learn:**
- Defensive testing mindset — in web3, negative tests matter more than positive.
- Common Anchor foot-guns: missing `has_one`, missing signer checks, seed confusion.

**Exit criteria:** `anchor test` reports 100% coverage; every program exercises happy and adversarial paths.

---

## Step 9 — Devnet deploy + README

**What we do:** `solana airdrop` on devnet, `anchor deploy --provider.cluster devnet` for all 6, record Program IDs, TS demo script that runs the full loop against devnet. Fill in the homework README.

**What you learn:**
- Devnet workflow, airdrop rate limits, upgradeable programs, where bytecode lives.
- Reproducible interaction scripts.

**Exit criteria:** Demo script runs end-to-end against devnet. All Program IDs in README.

---

## Concept ladder (why this order)

- Step 2 → accounts & PDAs.
- Step 3 → CPI + authority.
- Step 4 → time + randomness + cross-program CPI with PDA signer.
- Step 5 → Metaplex + authority revocation.
- Step 6 → multi-CPI atomicity.
- Step 7 → gated cross-program mint.
- Steps 8–9 → rigor and shipping.

If a step feels shaky, slow down and extend it before piling on the next.
