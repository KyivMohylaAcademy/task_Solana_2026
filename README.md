# Гра "Козацький бізнес" — Версія для Solana

## Введення

Дане тестове завдання було підготовлено компанією WhiteBIT для студентів університету НаУКМА. Це завдання дає змогу компанії оцінити аналітичні, технічні та архітектурні навички кандидатів у екосистемі Solana.

---

## Вимоги до коду

| Параметр | Вимога |
|----------|--------|
| Мова програмування | Rust |
| Фреймворк | Anchor Framework (остання стабільна версія) |
| Мережа для деплою | Solana Devnet |
| Покриття тестами | 100% покриття всіх програм (через anchor test) |
| Інструментарій | Anchor CLI, Solana CLI, TypeScript для скриптів |
| Скрипти | Написані на TypeScript (використовуючи @coral-xyz/anchor) |
| Документація | Коментарі у форматі Rust doc comments (///) |
| README | Містить адреси всіх програм (Program ID), інструкції з деплою, приклади взаємодії |
| Формат здачі | Посилання на Pull Request у репозиторії GitHub, викладене на Distedu |

## Завдання: Гра "Козацький бізнес"

### Базові ресурси (SPL Token-2022)

У грі існує 6 базових ресурсів, реалізованих як SPL Token-2022 з розширенням MetadataPointer:

| ID | Назва | Символ | Decimals |
|----|-------|--------|----------|
| 0 | Дерево | WOOD | 0 |
| 1 | Залізо | IRON | 0 |
| 2 | Золото | GOLD | 0 |
| 3 | Шкіра | LEATHER | 0 |
| 4 | Камінь | STONE | 0 |
| 5 | Алмаз | DIAMOND | 0 |

Примітка: Використовуйте decimals = 0, оскільки ресурси є цілими одиницями.

---

### Унікальні предмети (NFT через Metaplex)

Гравці можуть об'єднувати ресурси та створювати унікальні предмети як NFT (стандарт Metaplex):

| Предмет | Рецепт |
|---------|--------|
| Шабля козака | 3× Залізо + 1× Дерево + 1× Шкіра |
| Посох старійшини | 2× Дерево + 1× Золото + 1× Алмаз |
| Броня характерника (опціонально) | 4× Шкіра + 2× Залізо + 1× Золото |
| Бойовий браслет (опціонально) | 4× Залізо + 2× Золото + 2× Алмаз |

---

## Механіка безпеки та доступу

### SPL Token-2022 / NFT (Metaplex)

- Створення токенів (ресурсів) можливе лише через програми Crafting або Search.
- Прямий мінтинг/спалення через базові Token Accounts — заборонено.
- Контроль доступу реалізується через PDA (Program Derived Addresses) та перевірку підписантів.

### Спалення NFT

- Спалення NFT можливе тільки під час продажу предметів у програмі Marketplace.
- Прямий burn через Token Program — заборонено (контролюється через PDA authority).

---

## Механіка MagicToken (SPL Token-2022)

- Токени MagicToken можна отримати лише через продаж предметів у програмі Marketplace.
- Прямий мінтинг через Token Program — заборонено.
- Мінт викликається виключно з програми Marketplace через CPI (Cross-Program Invocation).
- Отримані MagicToken надходять на токен-акаунт гравця після успішного продажу предмета.

---

## Механіка Crafting / Search

### Пошук ресурсів (Search Program)

- Гравець може запускати пошук ресурсів раз на 60 секунд.
- Пошук генерує 3 випадкових ресурси (SPL Token-2022), які надходять на токен-акаунти гравця.
- Для реалізації таймера використовується он-чейн облік часу в PDA-акаунті гравця.

### Створення предметів (Crafting Program)

Для створення предмета (NFT) через крафт, гравець повинен:
1. Мати необхідну кількість ресурсів на своїх токен-акаунтах.
2. Надати підпис транзакції.

Під час крафту:
- Ресурси спалюються (burn через CPI до Token-2022 Program).
- Створюється предмет (NFT) з унікальним mint address.
- NFT передається на акаунт гравця.

Створені предмети можна:
- Продавати на Marketplace
- Передавати іншим гравцям (standard NFT transfer)

---

## Механіка Marketplace

- Гравці можуть продавати предмети (NFT) за MagicToken.
- Після купівлі предмета:
  - NFT спалюється (burn через CPI).
  - Продавець отримує відповідну кількість MagicToken на свій токен-акаунт.
  - Покупець отримує NFT (або воно спалюється, залежно від логіки — уточнити).

---

## Архітектура програм

### Обов'язкові програми (Programs)

| Програма | Призначення |
|----------|-------------|
| resource_manager | Керування мінтом/спаленням ресурсів (SPL Token-2022) |
| item_nft | Керування створенням NFT-предметів (Metaplex) |
| crafting | Логіка крафту предметів з ресурсів |
| search | Логіка пошуку ресурсів з таймером |
| marketplace | Купівля/продаж предметів за MagicToken |
| magic_token | Програма для мінту MagicToken (тільки через Marketplace) |

### Структура акаунтів (PDA)

```rust
// Гравець (Player Account)
#[account]
pub struct Player {
    pub owner: Pubkey,
    pub last_search_timestamp: i64,
    pub bump: u8,
}

// Налаштування гри (GameConfig Account)
#[account]
pub struct GameConfig {
    pub admin: Pubkey,
    pub resource_mints: [Pubkey; 6],
    pub magic_token_mint: Pubkey,
    pub item_prices: [u64; 4],
    pub bump: u8,
}

// Дані предмета (ItemMetadata Account)
#[account]
pub struct ItemMetadata {
    pub item_type: u8,
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub bump: u8,
}
```

---

## Вимоги до тестування

- 100% покриття всіх програм через anchor test.
- Використовувати Solana Program Test для локального тестування.
- Тести мають покривати:
  - Мінтинг/спалення ресурсів
  - Створення NFT через крафт
  - Таймер пошуку (60 секунд)
  - Продаж/купівля на Marketplace
  - Мінтинг MagicToken тільки через Marketplace
  - Перевірку прав доступу (PDA authority)

## Критерії оцінювання

| Критерій | Вага |
|----------|------|
| Архітектура програм | 25% |
| Безпека (PDA, authority checks) | 25% |
| Покриття тестами | 20% |
| Якість коду (Rust best practices) | 15% |
| Документація (README, коментарі) | 10% |
| Інновації/оптимізація | 5% |

---

## Корисні ресурси

- [Anchor Documentation](https://www.anchor-lang.com/)
- [Solana Developer Docs](https://solana.com/developers)
- [SPL Token-2022 Docs](https://spl.solana.com/token-2022)
- [Metaplex Token Metadata](https://developers.metaplex.com/token-metadata)
- [Solana Program Library](https://github.com/solana-labs/solana-program-library)

---

## Здача завдання

1. Створіть pull request в цьому репозиторії на GitHub.
2. Додайте всі вихідні коди, тести, скрипти та README.
3. Створіть Pull Request з описом реалізації.
4. Відправте посилання на PR через Distedu.

---

## Важливі зауваження

- Не використовуйте Solidity або EVM-інструменти.
- Всі програми мають бути деплоєні на Solana Devnet.
- MagicToken може бути замінений на будь-який інший SPL Token для тестування.
- Таймер 60 секунд має бути реалізований он-чейн (через PDA з timestamp).
- Всі транзакції мають бути підписані користувачем (owner check).

---

## Реалізація

### Технологічний стек

| Параметр | Значення |
|----------|---------|
| Мова | Rust |
| Фреймворк | Anchor Framework 1.0.0 |
| Мережа | Solana Devnet |
| Токен-стандарт | SPL Token-2022 (з розширеннями MetadataPointer + TokenMetadata) |
| NFT-стандарт | Token-2022 (supply=1, decimals=0) — без Metaplex |
| TypeScript SDK | @anchor-lang/core 1.0.0 |

### Адреси програм (Program IDs) — Solana Devnet

| Програма | Program ID |
|----------|-----------|
| resource_manager | `G2FcuoLPQ8kSTTVLmKT9HR7b23em154skEidRVwTRtc9` |
| magic_token | `8KkKfeEqviwfGGnz1jLF5DVvhjC8HuSoXxnWJk4MQeoj` |
| item_nft | `7cSkPT8JwQHibWDnXCqsADcM5euk5RRSffRsz6U5QCkR` |
| search | `8cNt5T19ynDwaV8MkTBodqWnY4HydWZu4SdKDWL6ZU5X` |
| crafting | `9gbQXMjCev1K5GmK4SzBa52zKtXPveZQvVmaA15rvszs` |
| marketplace | `9tTHLWqsKEiVufU7uqeAgSJmJrvXTsCkSvTKgdrb2ofX` |

### Архітектура безпеки

Кожна CPI-програма має власний авторитет-PDA, що перевіряється програмою-одержувачем:

- `search::SearchAuthority` (seeds=`["search_authority"]`) — авторизований мінтинг ресурсів
- `crafting::CraftingAuthority` (seeds=`["crafting_authority"]`) — авторизований burn ресурсів + мінт NFT
- `marketplace::MarketplaceAuthority` (seeds=`["marketplace_authority"]`) — авторизований burn NFT + мінт MagicToken

### Рецепти крафту

| Предмет | Рецепт |
|---------|--------|
| 0. Шабля козака | 3×IRON + 1×WOOD + 1×LEATHER |
| 1. Посох старійшини | 2×WOOD + 1×GOLD + 1×DIAMOND |
| 2. Броня характерника | 4×LEATHER + 2×IRON + 1×GOLD |
| 3. Бойовий браслет | 4×IRON + 2×GOLD + 2×DIAMOND |

### Ціни предметів у MagicToken

| Предмет | Ціна (MagicToken) |
|---------|-----------------|
| Шабля козака | 10 |
| Посох старійшини | 20 |
| Броня характерника | 30 |
| Бойовий браслет | 50 |

---

## Інструкції з деплою

### Передумови

```bash
# Встановити Anchor CLI 1.0.0
avm install 1.0.0 && avm use 1.0.0

# Встановити Solana CLI >= 1.18.x
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"

# Встановити залежності
cd kozatsky_biznes
yarn install
```

### Отримати SOL на Devnet

```bash
# Отримати адресу гаманця
solana address

# Запросити SOL через CLI (якщо доступно)
solana airdrop 5 --url devnet

# Або через браузер: https://faucet.solana.com
# Вставити адресу гаманця та отримати 5 SOL
```

### Збірка та деплой

```bash
cd kozatsky_biznes

# Зібрати всі 6 програм
anchor build

# Задеплоїти на Devnet
anchor deploy --provider.cluster devnet

# Ініціалізувати ігровий стан
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com \
ANCHOR_WALLET=~/.config/solana/devnet.json \
npx ts-node scripts/deploy.ts
```

### Запуск тестів

```bash
# Unit-тести (27 тестів, без мережі)
cargo test -p resource_manager -p magic_token -p item_nft -p search -p crafting -p marketplace

# Або через anchor
anchor test
```

---

## Приклади взаємодії

### Пошук ресурсів (Search)

```typescript
import * as anchor from "@anchor-lang/core";
import { PublicKey } from "@solana/web3.js";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const SEARCH_ID = new PublicKey("8cNt5T19ynDwaV8MkTBodqWnY4HydWZu4SdKDWL6ZU5X");
const [playerPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("player"), provider.wallet.publicKey.toBuffer()],
  SEARCH_ID
);

// Реєстрація гравця
await searchProgram.methods
  .initializePlayer()
  .accountsPartial({ player: playerPda, playerWallet: provider.wallet.publicKey })
  .rpc();

// Пошук ресурсів (1 раз на 60 секунд)
await searchProgram.methods
  .searchResources()
  .accountsPartial({ player: playerPda, resourceMint: ironMintAddress, ... })
  .rpc();
```

### Крафт предмету (Crafting)

```typescript
// Крафт Шаблі козака (item_type=0)
// Рецепт: 3×IRON + 1×WOOD + 1×LEATHER
const nftMint = Keypair.generate();
await craftingProgram.methods
  .craftItem(0)
  .accountsPartial({
    player: playerWallet.publicKey,
    resourceMint0: ironMint,    // IRON ×3
    resourceMint1: woodMint,    // WOOD ×1
    resourceMint2: leatherMint, // LEATHER ×1
    nftMint: nftMint.publicKey,
    ...
  })
  .signers([nftMint])
  .rpc();
```

### Продаж предмету (Marketplace)

```typescript
// Продаж Шаблі козака за 10 MagicToken
await marketplaceProgram.methods
  .sellItem(0) // item_type=0
  .accountsPartial({
    player: playerWallet.publicKey,
    nftMint: saberNftMint,
    magicTokenMint: magicTokenMintAddress,
    ...
  })
  .rpc();
// Гравець отримує 10 MagicToken на свій акаунт
```

