# Козацький Бізнес — Solana

Шість програм Anchor, що реалізують повний ігровий цикл: збір ресурсів, крафтинг NFT-предметів і їх продаж за ігрову валюту MagicToken.

## Програми

| Програма | Відповідальність |
|---|---|
| `resource_manager` | Token-2022 мінти для 6 ресурсів, `GameConfig` PDA, спалення/карбування через CPI auth |
| `magic_token` | Мінт MagicToken; карбувати може лише маркетплейс |
| `item_nft` | Metaplex NFT-мінти та `ItemMetadata` PDA; карбує крафтинг, спалює маркетплейс |
| `search` | Реєстрація гравця, 60-с кулдаун, псевдовипадкові дропи ресурсів |
| `crafting` | Перевірка рецептів, спалення ресурсів, CPI до `item_nft::mint_nft` |
| `marketplace` | Продаж NFT грі — спалення + виплата MagicToken |

```
search      ──► resource_manager::mint_resource
crafting    ──► resource_manager::burn_resource + item_nft::mint_nft
marketplace ──► item_nft::burn_nft + magic_token::mint_magic_token
```

**CPI auth.** Кожна привілейована інструкція перевіряє PDA `["cpi_auth"]` викликача проти константи в `constants/mod.rs` — ідентифікація без гаманця адміна.

**Два стандарти.** Ресурси і MagicToken — Token-2022 (потрібен `TokenMetadata`). NFT-предмети — класичний SPL Token (Metaplex не підтримує Token-2022).

## Ресурси та рецепти

Ресурси (id 0–5): Wood, Iron, Gold, Leather, Stone, Diamond.

| `item_type` | Предмет | Рецепт | Ціна |
|---|---|---|---|
| 0 | Козацька шабля | 1× Wood + 3× Iron + 1× Leather | 10 |
| 1 | Посох старійшини | 2× Wood + 1× Gold + 1× Diamond | 15 |
| 2 | Обладунок характерника | 2× Iron + 1× Gold + 4× Leather | 20 |
| 3 | Бойовий браслет | 4× Iron + 2× Gold + 2× Diamond | 25 |

## Локальне тестування

Передумови: Rust nightly (`rust-toolchain.toml`), Solana CLI ≥ 1.18, Anchor CLI 0.30.1, Node ≥ 18.

```bash
npm install
anchor build
anchor test
```

`anchor test` піднімає локальний валідатор, завантажує Metaplex (`mpl_token_metadata.so`) через genesis-конфіг і послідовно виконує тестові файли через `tests/pipeline.ts`. Пошукові тести використовують `anchor-bankrun` для маніпуляцій годинником — їхній стан відокремлений.

## Розгортання на Devnet

1. **Метадані NFT.** Замініть плейсхолдери URI у `programs/item_nft/src/constants/mod.rs` (`NFT_URIS`) на реальні JSON за схемою Metaplex, потім `anchor build`.

2. **Гаманець.** `deploy.json` вже в `.gitignore`.
   ```bash
   solana-keygen new -o deploy.json
   solana airdrop 2 $(solana-keygen pubkey deploy.json) --url devnet
   ```
   У `Anchor.toml` виставте `cluster = "Devnet"`, `wallet = "deploy.json"`.

3. **Двофазне розгортання.** `item_nft` йде **останнім** — guard-и в `crafting` і `marketplace` мають бути вже скомпільовані.
   ```bash
   anchor deploy --program-name resource_manager --provider.cluster devnet
   anchor deploy --program-name magic_token      --provider.cluster devnet
   anchor deploy --program-name search           --provider.cluster devnet
   anchor deploy --program-name crafting         --provider.cluster devnet
   anchor deploy --program-name marketplace      --provider.cluster devnet
   anchor deploy --program-name item_nft         --provider.cluster devnet
   ```

4. **Ініціалізація стану.** Після розгортання один раз запустіть `init.ts` — створить 6 ресурсних мінтів, MagicToken мінт і `GameConfig`, запише адреси в `devnet-accounts.json`.
   ```bash
   ANCHOR_PROVIDER_URL=https://api.devnet.solana.com \
   ANCHOR_WALLET=./deploy.json \
   npx ts-node app/scripts/init.ts
   ```

## Демонстрація

`app/scripts/play.ts` проганяє повний цикл: `register_player` → `search_resources` (×2 з кулдауном) → `admin_mint_resource` → `craft_item(0)` → `sell_item`.

```bash
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com \
ANCHOR_WALLET=./deploy.json \
npx ts-node app/scripts/play.ts
```

Приклади виклику кожної інструкції окремо дивіться в `app/scripts/play.ts` та у тестах під `tests/cases/`. Для транзакцій крафтингу і продажу обов'язково додавайте `ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 })` — вкладені CPI перевищують стандартний ліміт 200k CU.

## Адреси програм (localnet)

| Програма | Program ID |
|---|---|
| `resource_manager` | `DFtQE4puDvEMk1vYHhx3gQvfjUieWj1YtkhDKoyGCG1y` |
| `magic_token` | `5sk7gq8TwXpGFe7bxCsgWJ2k7StymKfXzkUD7HUfcMaY` |
| `item_nft` | `2DqgLTXd1joDVbtu3DSbocd8C9zExybcdzYH7a6gUXno` |
| `search` | `8idBXvmxQEwn8BCVe5W8nzJqktRsgubP1eFUJ6XQLuRc` |
| `crafting` | `YR3AszQR5gP98pMuzFb81Apb5KCsFi7U1gsSxfFeocF` |
| `marketplace` | `6mYp9XMhdaqcRq9xh4EDBmRDGaDEEphzEJzpPF5KEpvX` |

Metaplex Token Metadata: `metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s` (завантажується з `mpl_token_metadata.so`). Devnet ID впишіть сюди після розгортання.
