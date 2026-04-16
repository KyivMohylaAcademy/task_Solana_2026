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

## Implementation

### Program IDs (localnet)

| Program | ID |
|---|---|
| `resource_manager` | `AC4HSs3SakEbMAqefhDamXebxdGi3ZMktRWfyrXg22TR` |
| `search` | `8DHJBMqyodTKcEaix734FAjsLRaMj2q1fnxSmaMVnUfV` |
| `item_nft` | `CJi4wPcNAJmDyaJQ1ybYmF1hKP6Xtm1RT3JR9S2MbGiX` |
| `crafting` | `6iytFXd7TuSKY1oHjBx1P7yWAw3mQodwFHnb147eyV55` |
| `magic_token` | `7wUSeVT8HEm1JnoWnWQBEeKcH5R9sf5uA5bFdCj5JT3M` |
| `marketplace` | `BgCMRC1AvwfXKGx7jmLMPkq7ccZLnVd7EZ78szDFQpjb` |

### Implementation status

| Step | Description | Status |
|---|---|---|
| 0–1 | Tooling, workspace scaffold | ✅ Done |
| 2 | `GameConfig` PDA, admin init | ✅ Done |
| 3 | 6 SPL Token-2022 resource mints with `MetadataPointer` | ✅ Done |
| 4 | `Player` PDA, `search_resources` with 60s cooldown, gated `mint_resource` CPI | ✅ Done |
| 5 | `item_nft` — Metaplex 1-of-1 NFTs, gated burn | ✅ Done |
| 6 | `crafting` — burn resources, mint NFT | Pending |
| 7 | `magic_token` + `marketplace` — sell NFT, receive MagicToken | Pending |
| 8 | Security sweep, 100% negative-path coverage | Pending |
| 9 | Devnet deploy | Pending |

### Architecture highlights

**Gated CPI security pattern** — every cross-program mint/burn is restricted to a single registered caller. The callee validates the caller by checking that an expected PDA (derived with `seeds::program = config.registered_program`) is present as a `Signer`. Since only the registered program can produce that PDA signature via `invoke_signed`, no other program or wallet can bypass the gate.

```
search  ──CPI──▶  resource_manager::mint_resource
                  (search_authority PDA as Signer, seeds::program = game_config.search_program)

marketplace ──CPI──▶  item_nft::burn_item_nft
                      (marketplace_authority PDA as Signer, seeds::program = item_config.marketplace_program)
```

**Token standards:**
- Resources: SPL Token-2022 with `MetadataPointer` + `TokenMetadata` extensions (name/symbol/uri on-chain, no Metaplex).
- Item NFTs: classic SPL Token (required by Metaplex) + `create_master_edition_v3(max_supply = Some(0))` — Metaplex transfers mint authority to the edition PDA internally, making further minting impossible.

**On-chain randomness:** `search_resources` uses a Knuth MMIX LCG seeded from `Clock.slot ^ Clock.unix_timestamp ^ player_pubkey ^ last_search_ts`. Sufficient for a game demo; VRF (e.g. Switchboard) recommended for any real-money mechanic.

### Build & test

Prerequisites: Rust, Solana CLI ≥ 2.x, Anchor CLI 1.0, Node 20+, pnpm.

```bash
cd kozak-business
pnpm install
anchor build
anchor test
```

The test suite requires the Metaplex Token Metadata program binary at `tests/fixtures/mpl_token_metadata.so`. Fetch it once from devnet:

```bash
solana program dump -u devnet metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s \
  tests/fixtures/mpl_token_metadata.so
```

### Interaction examples

**Initialize the game (run once):**
```typescript
await resourceManagerProgram.methods
  .initializeGameConfig()
  .accounts({ admin: wallet.publicKey })
  .rpc();

for (let i = 0; i < 6; i++) {
  await resourceManagerProgram.methods
    .initializeResourceMint(i)
    .accounts({ admin: wallet.publicKey })
    .rpc();
}
```

**Register programs with each other:**
```typescript
await resourceManagerProgram.methods
  .setSearchProgram(searchProgram.programId)
  .accounts({ admin: wallet.publicKey })
  .rpc();

await itemNftProgram.methods
  .setMarketplaceProgram(marketplaceProgram.programId)
  .accounts({ admin: wallet.publicKey })
  .rpc();
```

**Search for resources (60s cooldown):**
```typescript
await searchProgram.methods
  .searchResources()
  .accounts({ player: playerPda, wallet: wallet.publicKey, ... })
  .rpc();
```

**Mint an item NFT:**
```typescript
const mintKeypair = Keypair.generate();
await itemNftProgram.methods
  .mintItemNft("Kozak Sword", "KSWD", "https://example.com/kozak-sword.json")
  .accounts({ mint: mintKeypair.publicKey, recipient: wallet.publicKey, ... })
  .signers([mintKeypair])
  .rpc();
```

