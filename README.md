# Козацький бізнес — Solana Blockchain Game

Блокчейн-гра з управлінням ресурсами та крафтом на Solana з використанням Anchor Framework. Реалізує повну ігрову економіку: SPL Token-2022 ресурси, NFT предмети та маркетплейс.

## Program IDs (Devnet)

| Програма           | Program ID                                           |
|--------------------|------------------------------------------------------|
| **crafting**       | `2hG2aS4hLrTPcKh5dLaRxW3Yd9brJMaGKQZYM9wTnfBF`     |
| **item_nft**       | `5KoKMBpkpNYQBaht7e55RzH31x7zyEXHGN19GAnz9m4s`     |
| **magic_token**    | `Cnp9S3UtAQEKmhtgVErCVFedtDXfwbZY8kgNYbugGLtn`     |
| **marketplace**    | `5pZRWZiRXhd6NJ67v1B4TzAGDrVzrH6cFysU8KME3rdE`     |
| **resource_manager** | `EfF5M1kLuDPQEvyGjB2WFaAj2epuWJTEHgVUN9UrAXs6`   |
| **search**         | `3v5wDujpABMtXCgN6F6YaG5z2LacM2FWNpj31qmgTzjN`     |

## Solana Explorer

crafting: https://explorer.solana.com/address/2hG2aS4hLrTPcKh5dLaRxW3Yd9brJMaGKQZYM9wTnfBF?cluster=devnet

item_nft: https://explorer.solana.com/address/5KoKMBpkpNYQBaht7e55RzH31x7zyEXHGN19GAnz9m4s?cluster=devnet

magic_token: https://explorer.solana.com/address/Cnp9S3UtAQEKmhtgVErCVFedtDXfwbZY8kgNYbugGLtn?cluster=devnet

marketplace: https://explorer.solana.com/address/5pZRWZiRXhd6NJ67v1B4TzAGDrVzrH6cFysU8KME3rdE?cluster=devnet

resource_manager: https://explorer.solana.com/address/EfF5M1kLuDPQEvyGjB2WFaAj2epuWJTEHgVUN9UrAXs6?cluster=devnet

search: https://explorer.solana.com/address/3v5wDujpABMtXCgN6F6YaG5z2LacM2FWNpj31qmgTzjN?cluster=devnet

### Результати тестів — 43 passing

```
  crafting
    initialize
      ✔ should initialize crafting config (368ms)
    craft_item
      ✔ should craft a Cossack Saber (item type 0) (426ms)
      ✔ should craft an Elder Staff (item type 1) (426ms)
      ✔ should craft Characternik Armor (item type 2) (423ms)
      ✔ should craft Battle Bracelet (item type 3) (429ms)
      ✔ should fail with invalid item type
    burn_resource
      ✔ should validate resource ID
      ✔ should validate amount is greater than zero

  integration
    Full Game Flow
      ✔ should complete a full gameplay cycle (2565ms)
      ✔ should enforce all security constraints (437ms)
    Recipe Validation
      ✔ should validate all item recipes (1693ms)

  item_nft
    initialize
      ✔ should initialize item NFT config
    create_item
      ✔ should create a Cossack Saber NFT (type 0) (414ms)
      ✔ should fail with invalid item type
    transfer_item
      ✔ should transfer item ownership (413ms)
      ✔ should fail transfer from non-owner
    burn_item
      ✔ should burn an item NFT (843ms)

  magic_token
    initialize
      ✔ should initialize MagicToken config (428ms)
      ✔ should fail to initialize twice
    mint_tokens
      ✔ should validate mint address

  marketplace
    initialize
      ✔ should initialize marketplace config
    list_item
      ✔ should list a Cossack Saber for sale (394ms)
      ✔ should list with custom price (429ms)
      ✔ should fail with invalid item type
    cancel_listing
      ✔ should cancel an active listing (417ms)
      ✔ should fail to cancel inactive listing
      ✔ should fail when non-seller tries to cancel (428ms)
    update_prices
      ✔ should update item prices (420ms)
      ✔ should fail when non-admin tries to update prices
    sell_item_direct
      ✔ should validate item type

  resource_manager
    initialize
      ✔ should initialize resource manager config (401ms)
      ✔ should fail to initialize twice
    create_resource_mint
      ✔ should create all 6 resource mints (2548ms)
      ✔ should fail with invalid resource ID
    mint_resource
      ✔ should validate mint address
    burn_resource
      ✔ should validate resource ID for burning

  search
    initialize
      ✔ should initialize search config
    init_player
      ✔ should initialize player search account
      ✔ should fail to initialize player twice
    search_resources
      ✔ should allow first search immediately
      ✔ should enforce 60 second cooldown
      ✔ should allow search after cooldown expires (61460ms)
    get_cooldown_remaining
      ✔ should return cooldown time (426ms)


  43 passing (90.56s)
```

## Огляд

Гравці:
1. Шукають базові ресурси кожні 60 секунд (кулдаун на ланцюгу)
2. Крафтять NFT-предмети з зібраних ресурсів
3. Продають предмети на маркетплейсі за MagicToken

```
Пошук (60с кулдаун) → Ресурси (SPL Token-2022)
    → Крафт (спалює ресурси, створює NFT)
    → Маркетплейс (спалює NFT, мінтить MagicToken)
```

## Архітектура

Система складається з 6 спеціалізованих Solana-програм:

**Token Layer:**
- `resource_manager` — 6 ресурсних токенів (SPL Token-2022 з MetadataPointer)
- `magic_token` — ігрова валюта (SPL Token-2022)
- `item_nft` — NFT-система предметів

**Game Logic Layer:**
- `search` — пошук ресурсів з 60-секундним кулдауном
- `crafting` — крафт предметів з рецептами
- `marketplace` — торгівля предметами та економіка

### Міжпрограмна комунікація (CPI)

```
search → resource_manager     (мінт ресурсів)
crafting → resource_manager   (спалювання ресурсів)
crafting → item_nft           (створення NFT)
marketplace → item_nft        (спалювання NFT)
marketplace → magic_token     (мінт MagicToken)
```

## Ігрова механіка

### Базові ресурси

SPL Token-2022 з розширенням MetadataPointer, 0 десяткових:

| ID | Назва    | Символ   |
|----|----------|----------|
| 0  | Wood     | WOOD     |
| 1  | Iron     | IRON     |
| 2  | Gold     | GOLD     |
| 3  | Leather  | LEATHER  |
| 4  | Stone    | STONE    |
| 5  | Diamond  | DIAMOND  |

### Предмети для крафту

NFT-предмети з рецептами (формат: [WOOD, IRON, GOLD, LEATHER, STONE, DIAMOND]):

| Предмет              | Рецепт               | Символ   | Ціна за замовчуванням |
|----------------------|-----------------------|----------|-----------------------|
| Cossack Saber        | [1, 3, 0, 1, 0, 0]   | SABER    | 100 MT                |
| Elder Staff          | [2, 0, 1, 0, 0, 1]   | STAFF    | 150 MT                |
| Characternik Armor   | [0, 2, 1, 4, 0, 0]   | ARMOR    | 200 MT                |
| Battle Bracelet      | [0, 4, 2, 0, 0, 2]   | BRACELET | 250 MT                |

### Пошук ресурсів

- Кулдаун: 60 секунд (зберігається в Player PDA)
- Результат: 3 псевдовипадкові ресурси за пошук
- Випадковість: slot + timestamp + search count

## Програми — API Reference

### search

| Інструкція             | Параметри              | Опис                                      |
|------------------------|------------------------|--------------------------------------------|
| `initialize`           | —                      | Ініціалізація конфігурації пошуку          |
| `init_player`          | —                      | Створення акаунту гравця                   |
| `search_resources`     | —                      | Пошук ресурсів (60с кулдаун)              |
| `mint_resource_token`  | `resource_id`, `amount`| Мінт ресурсного токену                    |
| `get_cooldown_remaining`| —                     | Час до наступного пошуку                   |

**Акаунти:** `SearchConfig` (admin, bump), `Player` (owner, last_search_timestamp, total_searches, bump)

**Помилки:** `SearchCooldownActive`, `InvalidResourceId`, `Unauthorized`

### crafting

| Інструкція         | Параметри                  | Опис                                    |
|--------------------|----------------------------|-----------------------------------------|
| `initialize`       | —                          | Ініціалізація конфігурації крафту        |
| `craft_item`       | `item_type: u8`            | Крафт предмета з ресурсів               |
| `burn_resource`    | `resource_id: u8`, `amount: u64` | Спалювання ресурсу                |
| `create_nft_item`  | `item_type: u8`            | Створення NFT після спалювання ресурсів  |

**Акаунти:** `CraftingConfig` (admin, total_crafted, bump)

**Помилки:** `InvalidItemType`, `InvalidResourceId`, `InsufficientResources`, `InvalidAmount`

### item_nft

| Інструкція       | Параметри                          | Опис                           |
|------------------|------------------------------------|--------------------------------|
| `initialize`     | —                                  | Ініціалізація конфігурації NFT |
| `create_item`    | `item_type: u8`, `uri: String`     | Створення NFT-предмета        |
| `burn_item`      | —                                  | Спалювання NFT                 |
| `transfer_item`  | —                                  | Передача власності             |

**Акаунти:** `ItemConfig` (admin, total_items_minted, bump), `ItemMetadata` (item_type, owner, mint, bump)

**Помилки:** `InvalidItemType`, `Unauthorized`

### magic_token

| Інструкція     | Параметри           | Опис                                        |
|----------------|---------------------|---------------------------------------------|
| `initialize`   | `decimals: u8`      | Ініціалізація мінту MagicToken              |
| `mint_tokens`  | `amount: u64`       | Мінт MagicToken (тільки через маркетплейс)  |

**Акаунти:** `MagicTokenConfig` (admin, mint, bump)

**Помилки:** `InvalidMint`, `Unauthorized`

### marketplace

| Інструкція         | Параметри                                    | Опис                              |
|--------------------|----------------------------------------------|-----------------------------------|
| `initialize`       | `item_prices: [u64; 4]`                      | Ініціалізація маркетплейсу        |
| `list_item`        | `item_type: u8`, `custom_price: Option<u64>` | Виставлення предмета на продаж    |
| `buy_item`         | —                                            | Покупка предмета                  |
| `cancel_listing`   | —                                            | Скасування лістингу               |
| `update_prices`    | `new_prices: [u64; 4]`                       | Оновлення цін (тільки адмін)     |
| `sell_item_direct` | `item_type: u8`                              | Прямий продаж без лістингу        |

**Акаунти:** `MarketplaceConfig` (admin, item_prices, total_sales, bump), `ItemListing` (seller, item_mint, item_type, price, is_active, bump)

**Помилки:** `InvalidItemType`, `ListingNotActive`, `Unauthorized`

### resource_manager

| Інструкція            | Параметри                                    | Опис                                  |
|-----------------------|----------------------------------------------|---------------------------------------|
| `initialize`          | —                                            | Ініціалізація менеджера ресурсів      |
| `create_resource_mint`| `resource_id: u8`, `name`, `symbol`          | Створення мінту ресурсу (Token-2022)  |
| `mint_resource`       | `resource_id: u8`, `amount: u64`             | Мінт ресурсів                         |
| `burn_resource`       | `resource_id: u8`, `amount: u64`             | Спалювання ресурсів                   |

**Акаунти:** `ResourceConfig` (admin, resource_mints: [Pubkey; 6], bump)

**Помилки:** `InvalidResourceId`, `Unauthorized`, `InvalidMint`

## PDA Derivation

```typescript
// Config PDA (всі програми)
const [configPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("config")],
  programId
);

// Player PDA (search)
const [playerPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("player"), playerPubkey.toBuffer()],
  searchProgramId
);

// Item metadata PDA (item_nft)
const [itemMetadataPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("item"), mintPubkey.toBuffer()],
  itemNftProgramId
);

// Listing PDA (marketplace)
const [listingPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("listing"), itemMintPubkey.toBuffer()],
  marketplaceProgramId
);

// Marketplace authority PDA
const [marketplaceAuthority] = PublicKey.findProgramAddressSync(
  [Buffer.from("marketplace_authority")],
  marketplaceProgramId
);

// Mint authority PDA (magic_token)
const [mintAuthority] = PublicKey.findProgramAddressSync(
  [Buffer.from("mint_authority")],
  magicTokenProgramId
);
```

## Приклади взаємодії

### Ініціалізація гри

```typescript
import * as anchor from "@coral-xyz/anchor";
const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const searchProgram = anchor.workspace.Search;
const craftingProgram = anchor.workspace.Crafting;
const itemNftProgram = anchor.workspace.ItemNft;
const marketplaceProgram = anchor.workspace.Marketplace;

// Ініціалізація всіх програм
await searchProgram.methods.initialize().accounts({
  config: searchConfigPda,
  admin: provider.wallet.publicKey,
  systemProgram: anchor.web3.SystemProgram.programId,
}).rpc();

await marketplaceProgram.methods.initialize([
  new anchor.BN(100), new anchor.BN(150),
  new anchor.BN(200), new anchor.BN(250),
]).accounts({
  config: marketplaceConfigPda,
  admin: provider.wallet.publicKey,
  systemProgram: anchor.web3.SystemProgram.programId,
}).rpc();
```

### Повний ігровий цикл

```typescript
// 1. Ініціалізація гравця
await searchProgram.methods.initPlayer().accounts({
  player: playerPda,
  owner: player.publicKey,
  systemProgram: SystemProgram.programId,
}).rpc();

// 2. Пошук ресурсів (кожні 60 секунд)
await searchProgram.methods.searchResources().accounts({
  config: searchConfigPda,
  player: playerPda,
  owner: player.publicKey,
}).rpc();

// 3. Крафт предмета (наприклад, Козацька Шабля — тип 0)
await craftingProgram.methods.craftItem(0).accounts({
  config: craftingConfigPda,
  owner: player.publicKey,
  systemProgram: SystemProgram.programId,
}).rpc();

// 4. Створення NFT
await itemNftProgram.methods.createItem(0, "https://example.com/saber.json")
  .accounts({
    config: itemConfigPda,
    itemMetadata: itemMetadataPda,
    mint: itemMint.publicKey,
    tokenAccount: tokenAccount,
    owner: player.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
    rent: SYSVAR_RENT_PUBKEY,
  }).rpc();

// 5. Лістинг на маркетплейсі
await marketplaceProgram.methods.listItem(0, null).accounts({
  config: marketplaceConfigPda,
  listing: listingPda,
  itemMint: itemMint.publicKey,
  seller: player.publicKey,
  systemProgram: SystemProgram.programId,
}).rpc();

// 6. Прямий продаж за MagicToken
await marketplaceProgram.methods.sellItemDirect(0).accounts({
  config: marketplaceConfigPda,
  marketplaceAuthority: marketplaceAuthorityPda,
  itemMint: itemMint.publicKey,
  itemTokenAccount: itemTokenAccount,
  magicTokenMint: magicTokenMint,
  sellerMagicTokenAccount: sellerMagicTokenAccount,
  seller: player.publicKey,
  tokenProgram: TOKEN_PROGRAM_ID,
  token2022Program: TOKEN_2022_PROGRAM_ID,
}).rpc();
```

## Безпека

- **PDA-авторитет** — усі токенні операції контролюються через PDA
- **Кулдаун на ланцюгу** — 60 секунд між пошуками, зберігається в Player PDA
- **CPI-валідація** — міжпрограмні виклики перевіряють авторитет
- **Перевірка власника** — усі операції з акаунтами перевіряють підпис
- **Constraint-валідація** — Anchor constraints на рівні інструкцій

## Встановлення

### Передумови

- Rust 1.75+
- Solana CLI 1.18+
- Anchor CLI 0.31.1
- Node.js 18+ та Yarn

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Anchor CLI
cargo install --git https://github.com/coral-xyz/anchor --tag v0.31.1 anchor-cli

# Залежності проекту
yarn install
```

## Збірка

```bash
anchor build
```

## Деплой

### Налаштування Solana

```bash
# Встановити кластер Devnet
solana config set --url https://api.devnet.solana.com

# Перевірити конфігурацію
solana config get

# Створити гаманець (якщо немає)
solana-keygen new --outfile ~/.config/solana/id.json

# Отримати SOL для деплою через https://faucet.solana.com
```

### Деплой програм

```bash
anchor deploy
```

## Тестування

```bash
# Запуск усіх тестів
anchor test

# Запуск конкретного тесту
anchor test -- --grep "crafting"

# З детальними логами
RUST_LOG=debug anchor test
```

## Стек технологій

- **Мова:** Rust
- **Фреймворк:** Anchor 0.31.1
- **Мережа:** Solana Devnet
- **Токени:** SPL Token-2022 (ресурси, MagicToken), SPL Token (NFT)
- **Тести:** TypeScript, ts-mocha, chai
- **Залежності:** anchor-lang 0.31.1, anchor-spl 0.31.1, solana-program 2.1.0
