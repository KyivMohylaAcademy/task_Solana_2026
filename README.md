# Гра "Козацький бізнес" — Версія для Solana

## Test Result

```text
  Козацький бізнес
    Resource Manager
      ✔ Ініціалізує гру (initialize_game) (994ms)
      ✔ Створює 6 ресурсних мінтів (create_resource_mint) (5919ms)
      ✔ mint_resource is CPI-only at the top level
      ✔ mint_resource rejects invalid resource_id before auth checks
      ✔ burn_resource is CPI-only at the top level
      ✔ burn_resource rejects invalid resource_id before auth checks
      ✔ Не дозволяє створити мінт з невірним resource_id
      ✔ Не дозволяє створити мінт повторно
      ✔ Не дозволяє не-адміну створювати мінти (958ms)
      ✔ Встановлює magic_token_mint (set_magic_token_mint)
    Magic Token
      ✔ Ініціалізує MagicToken (initialize_magic_token) (985ms)
      ✔ Встановлює magic_token_mint в GameConfig (1034ms)
      ✔ Не дозволяє мінтити MagicTokens напряму (без marketplace CPI)
      ✔ Magic Token prevents duplicate initialize
    Item NFT
      ✔ Ініціалізує колекцію предметів (initialize_collection) (925ms)
      ✔ Item NFT prevents duplicate collection initialize
      ✔ Не дозволяє створювати NFT напряму (без crafting CPI)
      ✔ Item NFT rejects invalid item type before CPI auth
    Search
      ✔ Реєструє гравця (register_player) (961ms)
      ✔ Не дозволяє повторну реєстрацію
      ✔ Search enforces a full remaining account set
      ✔ Виконує пошук ресурсів (search_resources) (6934ms)
      ✔ Блокує пошук під час кулдауну (60 сек)
      ✔ Перевіряє оновлення таймстампу після пошуку
      ✔ Search validates resource mint ordering (2941ms)
      ✔ Search validates player PDA ownership (995ms)
    Crafting
      ✔ Крафтить Шаблю козака (item_type=0: 3 Iron + 1 Wood + 1 Leather) (25779ms)
      ✔ Crafting validates writable NFT accounts before CPI
      ✔ Crafting requires nft mint signer
      ✔ Не дозволяє крафт з невірним item_type
    Marketplace
      ✔ Продає предмет на маркетплейсі (sell_item) (21832ms)
      ✔ Не дозволяє продати з невірним item_type
      ✔ Marketplace rejects Token-2022 mint as an NFT mint
      ✔ Marketplace requires initialized sale accounts
      ✔ Перевіряє що MagicTokens мінтяться тільки через marketplace
    Безпека (PDA Authority)
      ✔ Перевіряє що mint_resource вимагає правильний caller_auth
      ✔ Перевіряє що burn_resource вимагає правильний caller_auth
      ✔ Перевіряє що тільки адмін може set_magic_token_mint (947ms)
    Інтеграційний тест (End-to-End)
      ✔ Повний цикл: search → craft → sell (6908ms)

  39 passing (1m)
```


## Архітектура

Гра складається з 6 Anchor-програм, які взаємодіють через CPI (Cross-Program Invocation):

| Програма | Program ID | Призначення |
|----------|-----------|-------------|
| `resource_manager` | `3fNrim2nw2ZZHViVxQgJwJSCfaDf3pW47unrqdrPtvRX` | Керування мінтом/спаленням ресурсів (SPL Token-2022) |
| `magic_token` | `3Q2x255EXm6eHnKznkpot4R7q288h69da8T5RtLZzxtj` | Програма для мінту MagicToken (тільки через Marketplace) |
| `item_nft` | `5eCCbTkFjGTtr7yCZdweFmUCwxxBfkh8WWyJN9APa5yS` | Керування створенням/спаленням NFT-предметів (Metaplex) |
| `search` | `Ga5SJfwaQQ45Xoh91PQu2vSueU6obYknYxX3ztjP9SZc` | Пошук ресурсів з он-чейн таймером 60 секунд |
| `crafting` | `HAxq5QNGtPDU55En9b5AaQjxNhbUF1nfbYQkMSF4Gq1T` | Крафт предметів з ресурсів |
| `marketplace` | `3Bexz7g4D8JT1kGxPANBXsUrBMbGZDToJ5abvSH6oAF3` | Продаж предметів за MagicToken |


### CPI Flow

```
search ──CPI──> resource_manager ──CPI──> SPL Token-2022 (mint)
crafting ──CPI──> resource_manager ──CPI──> SPL Token-2022 (burn)
crafting ──CPI──> item_nft ──CPI──> Metaplex Token Metadata (create NFT)
marketplace ──CPI──> item_nft ──CPI──> SPL Token (burn NFT)
marketplace ──CPI──> magic_token ──CPI──> SPL Token-2022 (mint MagicToken)
```

### Авторизація CPI

Кожна "frontend" програма (search, crafting, marketplace) має PDA `[b"cpi_authority"]`, який використовується як підписант для CPI-викликів. "Backend" програми (resource_manager, magic_token, item_nft) перевіряють що `caller_auth` є валідним PDA авторизованої програми.

---

## Базові ресурси (SPL Token-2022)

| ID | Назва | Символ | Decimals |
|----|-------|--------|----------|
| 0 | Дерево | WOOD | 0 |
| 1 | Залізо | IRON | 0 |
| 2 | Золото | GOLD | 0 |
| 3 | Шкіра | LEATHER | 0 |
| 4 | Камінь | STONE | 0 |
| 5 | Алмаз | DIAMOND | 0 |

Усі ресурси реалізовані як SPL Token-2022 з розширенням MetadataPointer (он-чейн метадані).

---

## Предмети (Metaplex NFT)

| Тип | Предмет | Рецепт |
|-----|---------|--------|
| 0 | Шабля козака | 3× Залізо + 1× Дерево + 1× Шкіра |
| 1 | Посох старійшини | 2× Дерево + 1× Золото + 1× Алмаз |
| 2 | Броня характерника | 4× Шкіра + 2× Залізо + 1× Золото |
| 3 | Бойовий браслет | 4× Залізо + 2× Золото + 2× Алмаз |

---

## PDA-акаунти

| Акаунт | Seeds | Програма |
|--------|-------|----------|
| GameConfig | `[b"game_config"]` | resource_manager |
| MintAuthority | `[b"mint_authority"]` | resource_manager |
| MagicTokenConfig | `[b"magic_config"]` | magic_token |
| MagicMintAuthority | `[b"magic_mint_authority"]` | magic_token |
| ItemCollection | `[b"item_collection"]` | item_nft |
| NftAuthority | `[b"nft_authority"]` | item_nft |
| ItemMetadata | `[b"item_metadata", nft_mint]` | item_nft |
| PlayerAccount | `[b"player", owner]` | search |
| CPI Authority | `[b"cpi_authority"]` | search / crafting / marketplace |

---

## Вимоги

- Rust 1.75+
- Solana CLI 1.18+
- Anchor CLI 0.32.1+
- Node.js 18+
- npm або yarn

---

## Встановлення

```bash
# Клонуємо репозиторій
git clone <url>
cd task_Solana_2026

# Встановлюємо залежності Node.js
npm install

# Налаштовуємо Solana CLI
solana config set --url localhost
solana-keygen new --no-passphrase

# Збираємо програми
anchor build

# Синхронізуємо Program IDs
anchor keys sync

# Перебудовуємо з новими ID
anchor build
```

---

## Запуск тестів

```bash
# Запуск локального валідатора та тестів
anchor test

# Тільки тести (якщо валідатор вже запущений)
anchor test --skip-local-validator
```

---

## Деплой на Devnet

```bash
# Налаштовуємо на Devnet
solana config set --url devnet

# Оновлюємо Anchor.toml

# Деплоїмо
anchor deploy

# Перевіряємо
anchor keys list
```

---

## Приклади взаємодії

### Ініціалізація гри (адмін)

```typescript
import * as anchor from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";

// Обчислюємо PDA
const [gameConfigPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("game_config")],
  resourceManager.programId
);
const [mintAuthorityPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("mint_authority")],
  resourceManager.programId
);

// Ініціалізуємо гру
await resourceManager.methods
  .initializeGame(
    [new BN(100), new BN(150), new BN(200), new BN(300)],
    searchProgramId,
    craftingProgramId,
    marketplaceProgramId
  )
  .accounts({
    admin: wallet.publicKey,
    gameConfig: gameConfigPda,
    mintAuthority: mintAuthorityPda,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

### Пошук ресурсів (гравець)

```typescript
// Реєстрація гравця
await search.methods
  .registerPlayer()
  .accounts({
    playerOwner: player.publicKey,
    playerAccount: playerAccountPda,
    systemProgram: SystemProgram.programId,
  })
  .signers([player])
  .rpc();

// Пошук ресурсів (раз на 60 секунд)
await search.methods
  .searchResources()
  .accounts({
    playerOwner: player.publicKey,
    playerAccount: playerAccountPda,
    gameConfig: gameConfigPda,
    mintAuthority: mintAuthorityPda,
    cpiAuthority: searchCpiAuthorityPda,
    resourceManagerProgram: resourceManager.programId,
    token2022Program: TOKEN_2022_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
  })
  .remainingAccounts(resourceMintsAndAtas)
  .signers([player])
  .rpc();
```

### Крафт предмета

```typescript
await crafting.methods
  .craftItem(0, "Шабля козака", "SABER", "https://example.com/saber.json")
  .accounts({
    player: player.publicKey,
    gameConfig: gameConfigPda,
    itemCollection: itemCollectionPda,
    nftAuthority: nftAuthorityPda,
    nftMint: nftMintKeypair.publicKey,
    nftTokenAccount: playerNftAta,
    metadataAccount: metadataPda,
    masterEdition: masterEditionPda,
    itemMetadata: itemMetadataPda,
    cpiAuthority: craftingCpiAuthorityPda,
    resourceManagerProgram: resourceManager.programId,
    itemNftProgram: itemNft.programId,
    token2022Program: TOKEN_2022_PROGRAM_ID,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    metadataProgram: TOKEN_METADATA_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
    rent: SYSVAR_RENT_PUBKEY,
  })
  .remainingAccounts(resourceMintsAndAtas)
  .signers([player, nftMintKeypair])
  .rpc();
```

### Продаж на маркетплейсі

```typescript
await marketplace.methods
  .sellItem(0) // item_type = 0 (Шабля козака)
  .accounts({
    seller: player.publicKey,
    gameConfig: gameConfigPda,
    itemCollection: itemCollectionPda,
    nftMint: nftMintPubkey,
    nftTokenAccount: playerNftAta,
    itemMetadata: itemMetadataPda,
    magicConfig: magicConfigPda,
    magicMintAuthority: magicMintAuthorityPda,
    magicMint: magicMintPubkey,
    sellerMagicAta: playerMagicAta,
    cpiAuthority: marketplaceCpiAuthorityPda,
    itemNftProgram: itemNft.programId,
    magicTokenProgram: magicToken.programId,
    tokenProgram: TOKEN_PROGRAM_ID,
    token2022Program: TOKEN_2022_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
  })
  .signers([player])
  .rpc();
```

---

## Механіка безпеки

- **PDA Authority**: Mint authority для всіх токенів — PDA відповідних програм. Прямий мінтинг неможливий.
- **CPI Authorization**: Кожна "frontend" програма підписує CPI-виклики своїм PDA. "Backend" програми перевіряють підпис.
- **Owner Checks**: Всі транзакції вимагають підпис гравця (власника токенів).
- **Он-чейн таймер**: Кулдаун пошуку (60 сек) реалізований через PDA з timestamp.
- **Token-2022 + MetadataPointer**: Ресурси мають он-чейн метадані без зовнішніх акаунтів.

---

## Структура проєкту

```
├── Anchor.toml              # Конфігурація Anchor
├── Cargo.toml               # Workspace конфігурація
├── package.json             # Node.js залежності
├── tsconfig.json            # TypeScript конфігурація
├── programs/
│   ├── resource_manager/    # Керування ресурсами (SPL Token-2022)
│   ├── magic_token/         # MagicToken (SPL Token-2022)
│   ├── item_nft/            # NFT предмети (Metaplex)
│   ├── search/              # Пошук ресурсів (таймер 60 сек)
│   ├── crafting/            # Крафт предметів
│   └── marketplace/         # Продаж за MagicToken
├── tests/
│   └── kozak-business.ts    # Тести (TypeScript)
└── migrations/
    └── deploy.ts            # Скрипт деплою
```

---

## Корисні ресурси

- [Anchor Documentation](https://www.anchor-lang.com/)
- [Solana Developer Docs](https://solana.com/developers)
- [SPL Token-2022 Docs](https://spl.solana.com/token-2022)
- [Metaplex Token Metadata](https://developers.metaplex.com/token-metadata)
- [Solana Program Library](https://github.com/solana-labs/solana-program-library)

---

## Devnet Deploy

### Program IDs

- `crafting`: `5BLa56P3DuD4GjLboQGKFAVv8gsYhP5A4eHLQHxS6QLR`
- `item_nft`: `9uYdFs7H7iZjRYMB2r3kvGATe5ZUaKvbZrkTijR5sGEw`
- `magic_token`: `CLennJDsGwnsVFAGfju1yH8frdb3QgC8i7mdXFRxhPKx`
- `marketplace`: `FAuToaomcNLiYTJFVoqTiGHqakDBC3T78MNfvCiHjfNh`
- `resource_manager`: `4DPGoF2kGBmnt4ZJTKzB7vdzwMGvDVEBP2pbMDjtbVib`
- `search`: `JCKmHHAQEmsKEi9DfS7VpiSmmGsMDszfLzGc3baFiSAW`

### Explorer Links

- [crafting](https://explorer.solana.com/address/5BLa56P3DuD4GjLboQGKFAVv8gsYhP5A4eHLQHxS6QLR?cluster=devnet)
- [item_nft](https://explorer.solana.com/address/9uYdFs7H7iZjRYMB2r3kvGATe5ZUaKvbZrkTijR5sGEw?cluster=devnet)
- [magic_token](https://explorer.solana.com/address/CLennJDsGwnsVFAGfju1yH8frdb3QgC8i7mdXFRxhPKx?cluster=devnet)
- [marketplace](https://explorer.solana.com/address/FAuToaomcNLiYTJFVoqTiGHqakDBC3T78MNfvCiHjfNh?cluster=devnet)
- [resource_manager](https://explorer.solana.com/address/4DPGoF2kGBmnt4ZJTKzB7vdzwMGvDVEBP2pbMDjtbVib?cluster=devnet)
- [search](https://explorer.solana.com/address/JCKmHHAQEmsKEi9DfS7VpiSmmGsMDszfLzGc3baFiSAW?cluster=devnet)

## Deploy

```bash
anchor deploy --provider.cluster devnet --provider.wallet .config\solana\id.json
```

