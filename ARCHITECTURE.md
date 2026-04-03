# Архітектура Гри "Козацький бізнес" на Solana

## Огляд системи

```
┌─────────────────────────────────────────────────────────────┐
│                     Game Front-End (TypeScript)              │
│                                                              │
│  - Wallet Connection                                         │
│  - Transaction Signing                                       │
│  - Event Monitoring                                          │
└──────────────┬──────────────────────────────────────────────┘
               │ Web3.js / Anchor.js
               │
        ┌──────▼──────┬─────────────┬──────────┬─────┬────────┐
        │              │             │          │     │        │
    ┌───┴─────┐  ┌────┴───┐  ┌─────┴──┐  ┌───┴──┐  │   ┌───┴──┐
    │  Search │  │ Crafting│  │ Item   │  │Market│  │   │Magic │
    │ Program │  │ Program │  │  NFT   │  │place │  │   │Token │
    │         │  │         │  │Program │  │Program  │   │Program│
    └───┬─────┘  └────┬───┘  └─────┬──┘  └───┬──┘  │   └───┬──┘
        │             │             │        │      │       │
        └─────────────┼─────────────┼────────┼──────┼───────┘
                      │
                  ┌───▼──────────────────┐
                  │  Resource Manager    │
                  │  Program             │
                  │                      │
                  │  - SPL Token-2022    │
                  │  - Mint/Burn Logic   │
                  └─────────────────────┘
```

## Деталі програм

### 1. Resource Manager Program

**Відповідальність:** Керування 6 базовими ресурсами як SPL Token-2022

**Key Functions:**
```rust
initialize_config(admin: Pubkey) -> PDA
mint_resource(resource_index: u8, amount: u64) -> Transfer
burn_resource(resource_index: u8, amount: u64) -> Transfer
```

**PDA:** `["game_config"]`

**Взаємодія:**
- ⬅️ Receives CPI calls from: `search`, `crafting`
- ➡️ Returns: Token transfers via SPL Token-2022

**Безпека:**
- ✅ Тільки авторизовані програми можуть мінтити ресурси
- ✅ PDA як authority для мінт-операцій
- ✅ Перевірка ресурсних індексів

### 2. Search Program

**Відповідальність:** Пошук ресурсів з таймером 60 секунд

**Key Functions:**
```rust
search_resources() -> (PlayerSearch PDA, Event)
```

**PDAs:**
- `["player_search", owner_pubkey]` - Player state з last_search_timestamp

**Механіка:**
1. Player запускає `search_resources()`
2. Програма перевіряє, чи пройшло 60 секунд
3. Якщо так, генерує 3 випадкових ресурсів (0-5)
4. Випускає `ResourcesSearched` event з ресурсами
5. Front-end отримує event та викликає `mint_resource` via CPI

**Таймер:**
```rust
clock = Clock::get()?; // On-chain timestamp
let elapsed = clock.unix_timestamp - player_search.last_search_timestamp;
require!(elapsed >= 60, SearchError::SearchNotReady);
```

**Безпека:**
- ✅ Перевірка 60-секундного інтервалу
- ✅ Player повинен підписати (owner check)
- ✅ Використання on-chain clock (не клієнт)

### 3. Crafting Program

**Відповідальність:** Крафт NFT предметів з ресурсів

**Key Functions:**
```rust
craft_item(item_type: u8) -> ItemMetadata PDA
```

**PDA:** `["item_metadata", item_mint_pubkey]`

**Рецепти:**
```
Item 0 (Козацька шабля): 1 Wood, 3 Iron, 1 Leather
Item 1 (Посох старійшини): 2 Wood, 1 Gold, 1 Diamond
Item 2 (Броня): 2 Iron, 1 Gold, 4 Leather
Item 3 (Браслет): 4 Iron, 2 Gold, 2 Diamond
```

**Процес крафту:**
1. Player запускає `craft_item(0)` з необхідними ресурсами
2. Crafting програма перевіряє баланси ресурсів
3. CPI call до Resource Manager для спалення ресурсів
4. NFT мінтується через Item NFT Program (CPI)
5. ItemMetadata акаунт створюється з посиланням на NFT

**CPI Chains:**
```
crafting::craft_item()
  ├─→ resource_manager::burn_resource() [WOOD]
  ├─→ resource_manager::burn_resource() [IRON]
  ├─→ resource_manager::burn_resource() [LEATHER]
  └─→ item_nft::create_item_nft()
```

### 4. Item NFT Program

**Відповідальність:** Управління NFT як предметами

**Key Functions:**
```rust
create_item_nft(item_type: u8, uri: String) -> NFT
burn_item_nft() -> Burnt
```

**PDA:** `["item_nft", mint_pubkey]`

**Структура:**
```rust
pub struct ItemNFTMetadata {
    pub item_type: u8,      // 0-3
    pub creator: Pubkey,    // Хто крафтив
    pub mint: Pubkey,       // NFT mint address
    pub uri: String,        // Metadata URI (JSON)
    pub bump: u8,
}
```

**Метадата JSON (зовні blockchain):**
```json
{
  "name": "Козацька шабля",
  "description": "Cossack Sabre - A mighty weapon of the Kozaks",
  "image": "https://...",
  "attributes": [
    { "trait_type": "Type", "value": "Weapon" },
    { "trait_type": "Power", "value": 100 }
  ]
}
```

### 5. Marketplace Program

**Відповідальність:** Торгівля предметами за MagicToken

**Key Functions:**
```rust
list_item(price: u64) -> Listing PDA
buy_item() -> Transaction
```

**PDAs:**
- `["listing", item_mint_pubkey]` - Offer інформація

**Торгівля:**
1. Seller запускає `list_item(1000)` з NFT
2. Listing PDA створюється з інформацією про ціну
3. Buyer запускає `buy_item()`
4. CPI до magic_token програми для мінту MagicToken
5. MagicToken йде Seller, NFT спалюється
6. Listing закривається

**CPI Chains:**
```
marketplace::buy_item()
  ├─→ magic_token::mint_magic_token() [для Seller]
  └─→ item_nft::burn_item_nft() [NFT спалюється]
```

### 6. Magic Token Program

**Відповідальність:** Мінтинг MagicToken (тільки через Marketplace)

**Key Functions:**
```rust
mint_magic_token(amount: u64) -> Token Transfer
```

**Контроль доступу:**
```rust
// Тільки Marketplace PDA може викликати
require_eq!(ctx.program_id, &MARKETPLACE_ID);
```

**PDA:** `["magic_token_config"]`

## Деплой та интеграція

### Порядок деплою

1. **Resource Manager** (залежить: ність)
   - Ініціалізує GameConfig з 6 ресурсів
   
2. **Search** (залежить: Resource Manager)
   - Може викликати Resource Manager для мінту
   
3. **Item NFT** (залежить: ність)
   - Просто створює NFT, без залежностей
   
4. **Crafting** (залежить: Resource Manager, Item NFT)
   - Сипалює ресурси та мінтує NFT
   
5. **Marketplace** (залежить: Item NFT, Magic Token)
   - Потребує знати адресу Magic Token
   
6. **Magic Token** (залежить: ність)
   - Потребує знати адресу Marketplace для валідації

### Конфіг файл (Anchor.toml)

```toml
[programs.devnet]
resource_manager = "ResourceMgr..."
search = "Search..."
crafting = "Crafting..."
item_nft = "ItemNFT..."
marketplace = "Marketplace..."
magic_token = "MagicToken..."
```

## Транзакція флоу

### 1️⃣ Пошук ресурсів

```
Player → Search::search_resources()
               ├─ Get Clock (on-chain)
               ├─ Validate 60s timer
               ├─ Generate random resources
               ├─ Update PlayerSearch PDA
               ├─ Emit ResourcesSearched event
               └─ Front-end receives event
                    ↓
                    Front-end → Resource Manager::mint_resource()
                                    ├─ Verify amount
                                    ├─ Mint WOOD/IRON/etc
                                    └─ Transfer to player
```

### 2️⃣ Крафт предмета

```
Player → Crafting::craft_item(0)
            ├─ Validate item type
            ├─ Check recipes
            ├─ CPI: Resource Manager::burn_resource()
            │        (burn Wood, Iron, Leather)
            ├─ CPI: Item NFT::create_item_nft()
            │        (mint NFT)
            ├─ Create ItemMetadata PDA
            └─ Emit ItemCrafted event
```

### 3️⃣ Продаж на Marketplace

```
Seller → Marketplace::list_item(1000)
            ├─ Create Listing PDA
            ├─ Store price & item info
            └─ Emit ItemListed event

Buyer → Marketplace::buy_item()
           ├─ Fetch Listing info
           ├─ Validate price
           ├─ CPI: Magic Token::mint_magic_token()
           │        (pay seller)
           ├─ CPI: Item NFT::burn_item_nft()
           │        (burn seller's NFT)
           ├─ Close Listing PDA
           └─ Emit ItemSold event
```

## Безпека та перевірки

### Owner Check (Signer Verification)

```rust
#[account(mut)]
pub owner: Signer<'info>,  // Повинен підписати транзакцію
```

Забезпечує, що:
- ✅ Тільки власник может витратити свої ресурси
- ✅ Тільки power може крафтити з своїх ресурсів
- ✅ Тільки власник може продати їх NFT

### PDA Authority

```rust
// Game Config PDA контролює мінтинг
pub struct GameConfig {
    pub admin: Pubkey,
    pub resource_mints: [Pubkey; 6],
    pub bump: u8,  // Для побудови PDA
}

// Тільки resource_manager програма може мінтити ресурси
seeds = [b"game_config"]
```

### CPI (Cross-Program Invocation)

```rust
// Crafting викликає Resource Manager
let cpi_accounts = token_2022::Burn {
    mint: ctx.accounts.resource_mint.to_account_info(),
    from: ctx.accounts.player_token_account.to_account_info(),
    authority: ctx.accounts.authority.to_account_info(),
};

let cpi_program = ctx.accounts.token_2022_program.to_account_info();
token_2022::burn(CpiContext::new(cpi_program, cpi_accounts), amount)?;
```

### Перевірки доступу

```rust
// Marketplace контролює MagicToken мінтинг
require_eq!(
    ctx.program_id,
    &MARKETPLACE_ID,
    MagicTokenError::UnauthorizedMinter
);
```

## Масштабованість та оптимізація

### 1. Batch Operations
- Можна крафтити кілька предметів у одній транзакції
- Можна міцти кілька ресурсів одночасно

### 2. Закритті Accounts
```rust
#[account(mut, close = owner)]
pub listing: Account<'info, Listing>,  // Reclaim lamports
```

### 3. Event Streaming
- Events дозволяють front-end слухати зміни без поллінгу
- Зменшує навантаження на RPC

## Тестування компонентів

### Unit Tests
- Кожна програма має свої модульні тести

### Integration Tests
- Тестування CPI interaction між програмами
- Валідація повного флоу (search → craft → sell)

### Load Tests
- Багато паралельних транзакцій
- Вимірювання TPS (Transactions Per Second)

---

**Это架構документація дає подробний огляд системи.**
