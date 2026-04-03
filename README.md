# Гра "Козацький бізнес" — Версія для Solana

**Solana Smart Contracts для гри "Козацький бізнес"**

## Введення

Це завдання реалізує комплексну гру на блокчейні Solana з використанням Anchor Framework. Гравці можуть:
- Шукати ресурси (з таймером 60 секунд)
- Крафтити предмети з ресурсів
- Торгувати предметами на маркетплейсі за MagicToken

## Архітектура

### Програми (Smart Contracts)

| Програма | Описання |
|----------|---------|
| **resource_manager** | Керування мінтом/спаленням 6 базових ресурсів (SPL Token-2022) |
| **search** | Логіка пошуку ресурсів з таймером 60 секунд |
| **crafting** | Крафт предметів з ресурсів |
| **item_nft** | Керування NFT предметами (Metaplex) |
| **marketplace** | Купівля/продаж предметів за MagicToken |
| **magic_token** | Мінтинг MagicToken (тільки через Marketplace) |

### Базові ресурси (SPL Token-2022)

| ID | Назва | Символ | Decimals |
|----|-------|--------|----------|
| 0 | Дерево | WOOD | 0 |
| 1 | Залізо | IRON | 0 |
| 2 | Золото | GOLD | 0 |
| 3 | Шкіра | LEATHER | 0 |
| 4 | Камінь | STONE | 0 |
| 5 | Алмаз | DIAMOND | 0 |

### Рецепти крафту

| Предмет | Рецепт |
|---------|--------|
| **Козацька шабля** (0) | 3× Залізо + 1× Дерево + 1× Шкіра |
| **Посох старійшини** (1) | 2× Дерево + 1× Золото + 1× Алмаз |
| **Броня характерника** (2) | 4× Шкіра + 2× Залізо + 1× Золото |
| **Бойовий браслет** (3) | 4× Залізо + 2× Золото + 2× Алмаз |

## Вимоги до середовища

### Обов'язкові інструменти

- **Rust** ≥ 1.70 (встановити з https://rustup.rs/)
- **Solana CLI** ≥ 1.18
- **Anchor CLI** 0.29.0
- **Node.js** ≥ 18

### Встановлення

```bash
# Установити Solana CLI
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Встановити Anchor
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install 0.29.0
avm use 0.29.0

# Встановити Node.js залежності
npm install
```

## Збірка

```bash
# Зібрати все
anchor build

# Зібрати конкретну програму
cargo build -p resource_manager
```

## Деплой

### Devnet

```bash
# Налаштувати Devnet
solana config set --url devnet

# Отримати SOL для деплою
solana airdrop 2

# Зробити деплой
anchor deploy

# Запустити тести
anchor test
```

### Localnet

```bash
# Запустити локальний валідатор
solana-test-validator --reset

# У іншому терміналі: налаштувати localnet
solana config set --url localhost

# Зробити деплой
anchor deploy --provider.cluster localnet
```

## Тестування

```bash
# Запустити всі тести
anchor test

# Тести покривають:
# - Мінтинг/спалення ресурсів
# - Крафт предметів
# - Таймер пошуку (60 секунд)
# - Торгівля на маркетплейсі
# - Мінтинг MagicToken
# - Перевірки прав доступу
```

## Адреси програм на Devnet

> **Оновлено:** Успішно задеплоєно на Devnet (April 3, 2026)

```
resource_manager: 2Y2tAWf4DGPhk9kTDHyyProMw4wrNJf6R6U61WL8D4Vv
search:           HDtdF8EjnBeRuVFVA3TUQFi3oM8qA8iGCcfrCJbRar1e
crafting:         CTHKMpMxaV89e4g7a4uwmvPmSYygWvtFn4vv9qRQ5m2t
item_nft:         9GU3Nb13w1YaA8vwfLo2MqWmakbVLF9G6xZiNqCXn8ns
marketplace:      5EyYkXzfHkH278x25q42csiR8FLeGvujpqCYdhncfcUd
magic_token:      4NvPT6ob4cPTGpXDq9TEp5ByuW5HgxAYYCUWW5xDS6dE
```

## Приклади взаємодії

### 1. Пошук ресурсів

```typescript
// Запустити пошук кожні 60 секунд
const txHash = await program.methods
  .searchResources()
  .accounts({
    playerSearch: playerSearchPDA,
    owner: wallet.publicKey,
    clock: SYSVAR_CLOCK_PUBKEY,
    systemProgram: SystemProgram.programId,
  })
  .signers([wallet])
  .rpc();
```

### 2. Крафт предмета

```typescript
// Крафтити Козацьку шаблю
const txHash = await program.methods
  .craftItem(0) // Item type 0
  .accounts({
    itemMetadata: itemMetadataPDA,
    itemMint: itemMint,
    owner: wallet.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([wallet, itemMint])
  .rpc();
```

### 3. Торгівля на маркетплейсі

```typescript
// Виставити предмет на продаж
const txHash = await program.methods
  .listItem(new BN(1000)) // Price in MagicToken
  .accounts({
    listing: listingPDA,
    itemMint: itemMint,
    seller: wallet.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([wallet, itemMint])
  .rpc();

// Купити предмет
const buyTx = await program.methods
  .buyItem()
  .accounts({
    listing: listingPDA,
    itemMint: itemMint,
    buyer: wallet.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([wallet, itemMint])
  .rpc();
```

## Структура проекту

```
.
├── programs/
│   ├── resource_manager/      # Управління ресурсами
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── errors/
│   │       ├── state/
│   │       └── instructions/
│   ├── search/                # Пошук ресурсів
│   ├── crafting/              # Крафт предметів
│   ├── item_nft/              # Управління NFT
│   ├── marketplace/           # Торгівля
│   └── magic_token/           # Управління MagicToken
├── tests/                     # TypeScript тести
├── utils/                     # Допоміжні функції
├── Anchor.toml               # Конфіг Anchor
├── Cargo.toml                # Конфіг Cargo (workspace)
└── package.json              # Node.js залежності
```

## Механіка безпеки

### Контроль доступу

- ✅ Всі операції з ресурсами контролюються PDA та перевіркою прав
- ✅ Мінтинг ресурсів можливий тільки через програми (Crafting, Search)
- ✅ Спалення NFT контролюється Marketplace
- ✅ MagicToken мінтується тільки через Marketplace

### PDA Seeds

```rust
// Game config
seeds = [b"game_config"]

// Player search state
seeds = [b"player_search", owner.key().as_ref()]

// Item metadata
seeds = [b"item_metadata", item_mint.key().as_ref()]

// NFT metadata
seeds = [b"item_nft", mint.key().as_ref()]

// Marketplace listing
seeds = [b"listing", item_mint.key().as_ref()]

// Magic token config
seeds = [b"magic_token_config"]
```

## Критерії оцінювання

| Критерій | Вага |
|----------|------|
| Архітектура програм | 25% |
| Безпека (PDA, контроль доступу) | 25% |
| Покриття тестами | 20% |
| Якість коду (Rust best practices) | 15% |
| Документація | 10% |
| Інновації/оптимізація | 5% |

## Важливі зауваження

- 🔴 **Не використовувати** Solidity або EVM-інструменти
- ✅ Всі програми на **Rust** з Anchor Framework
- ✅ Деплой тільки на **Solana Devnet**
- ✅ Таймер реалізований **он-чейн** за допомогою PDA з timestamp
- ✅ Всі транзакції мають бути **підписані користувачем** (owner check)
- ✅ **100% покриття тестами** через anchor test

## Корисні ресурси

- [Anchor Documentation](https://www.anchor-lang.com/)
- [Solana Developer Docs](https://solana.com/developers)
- [SPL Token-2022 Docs](https://spl.solana.com/token-2022)
- [Metaplex Token Metadata](https://developers.metaplex.com/token-metadata)
- [Solana Program Library](https://github.com/solana-labs/solana-program-library)

## Здача завдання

1. Створіть Fork цього репозиторію
2. Розробіть всі 6 програм
3. Напишіть 100% покриття тестами
4. Оновіть README з адресами програм
5. Зробіть Pull Request
6. Відправте посилання на PR через Distedu

## Ліцензія

MIT

---

**Автор:** Kyiv Mohyla Academy  
**Дата:** 2026  
**Тривалість:** Unlimited  
