# Козацький бізнес — On-chain гра на Solana

Тестове завдання WhiteBIT × НаУКМА. On-chain гра з ресурсами, крафтом предметів NFT та маркетплейсом на базі Solana / Anchor Framework.

---

## Program IDs (Solana Devnet)

| Програма | Program ID |
|----------|------------|
| `resource_manager` | `6oHg42ymyViYzXQh5jfD13kNBEUPWeE73cSZ3buMGjsh` |
| `magic_token` | `GRdyjoQis5Pc7hfiHff8zNWhoZgvrYapcwgJdqFoWQQc` |
| `item_nft` | `EEvis5VXdC5NRn4BrfUYdwcpzgptrfJZ3PsZ2UVxU8WY` |
| `crafting` | `BqUaswbtzh21TNbcCpB2TnZEsVy6W3dAwgNZhDKEpc44` |
| `search` | `3BRTKJk5xnswDXDMKogS43hcJVdpr1XDJrStZyLGduvs` |
| `marketplace` | `DuCW5GarF2Z6Jb2SXTL42hViJ6SwdPvKJFJkq6wtP5eC` |

---

## Архітектура

```
resource_manager  ←── search        (CPI: mint_resource)
      ↑
    crafting       ←── item_nft     (CPI: burn_resource → create_item_nft)
                         ↑
                    marketplace     (CPI: burn_item_nft → mint_magic_token)
                         ↓
                    magic_token
```

### Програми

| Програма | Призначення |
|----------|-------------|
| `resource_manager` | GameConfig PDA; 6 SPL Token-2022 ресурсних мінтів (WOOD/IRON/GOLD/LEATHER/STONE/DIAMOND); mint_authority = GameConfig PDA — прямий мінт заблоковано |
| `magic_token` | MagicToken SPL Token-2022; mint_authority = Marketplace PDA — мінт лише через marketplace |
| `item_nft` | NFT предмети через Metaplex Token Metadata; ItemMetadata PDA; burn лише через marketplace |
| `search` | Player PDA з таймером; пошук 3 рандомних ресурсів раз на 60 секунд (on-chain clock) |
| `crafting` | Крафт 4 предметів за рецептами; burn ресурсів → mint NFT через CPI |
| `marketplace` | Listing PDA + escrow; продаж NFT → MagicToken продавцю через CPI |

### Рецепти крафту

| Предмет | Рецепт |
|---------|--------|
| Шабля козака (0) | 3×IRON + 1×WOOD + 1×LEATHER |
| Посох старійшини (1) | 2×WOOD + 1×GOLD + 1×DIAMOND |
| Броня характерника (2) | 4×LEATHER + 2×IRON + 1×GOLD |
| Бойовий браслет (3) | 4×IRON + 2×GOLD + 2×DIAMOND |

---

## Вимоги

| Інструмент | Версія |
|------------|--------|
| Rust | `stable` (sbpf toolchain) |
| Solana CLI | `≥ 3.1` |
| Anchor CLI | `1.0.0` |
| Node.js | `≥ 18` |
| npm / yarn | будь-яка |

---

## Встановлення

```bash
# 1. Solana CLI
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"

# 2. Anchor CLI 1.0.0
cargo install --git https://github.com/coral-xyz/anchor anchor-cli --locked

# 3. Залежності npm
npm install
```

---

## Білд

```bash
anchor build
```

---

## Деплой на Devnet

```bash
solana config set --url devnet
solana airdrop 5          # за потреби, кілька разів

anchor deploy
```

Після деплою Program IDs фіксуються в `Anchor.toml` та `declare_id!` у кожній програмі.

---

## Тести

Тести потребують запущеного локального валідатора з клонованим Metaplex Token Metadata.

```bash
# Варіант 1 — локальний валідатор (PowerShell від адміністратора)
solana-test-validator --reset

# В окремому терміналі:
anchor test --validator legacy --skip-local-validator

# Варіант 2 — проти devnet (програми мають бути задеплоєні)
anchor test --skip-local-validator
```

---

## TypeScript Scripts

```bash
# 1. Ініціалізація (GameConfig + MagicToken + 6 ресурсних мінтів)
npm run setup        # → setup-result.json

# 2. Повне демо-взаємодії (search → craft → list → buy)
npm run interact

# 3. Rebuild + деплой всіх програм на Devnet
npm run deploy       # → deploy-result.json
```

Скрипти зберігають результати у файли `setup-result.json` / `deploy-result.json` для зручного перегляду Program IDs та адрес.

---

## Приклади взаємодії

### 1. Ініціалізація гравця

```typescript
await searchProgram.methods
  .initializePlayer()
  .accounts({ owner: player.publicKey })
  .signers([player])
  .rpc({ commitment: "confirmed" });
```

### 2. Пошук ресурсів (раз на 60 секунд)

```typescript
// remaining_accounts: [mint_0, ta_0, mint_1, ta_1, ..., mint_5, ta_5]
const remaining = resourceMints.flatMap((mint, i) => [
  { pubkey: mint,         isWritable: true, isSigner: false },
  { pubkey: playerTas[i], isWritable: true, isSigner: false },
]);

await searchProgram.methods
  .searchResources()
  .accounts({ owner: player.publicKey })
  .remainingAccounts(remaining)
  .signers([player])
  .rpc({ commitment: "confirmed" });
```

### 3. Крафт Шаблі козака (item_type = 0)

```typescript
// Потрібно: 3×IRON + 1×WOOD + 1×LEATHER
const nftMintKp = Keypair.generate();
const remaining = [
  { pubkey: ironMint,        isWritable: true, isSigner: false },
  { pubkey: playerIronTa,    isWritable: true, isSigner: false },
  { pubkey: woodMint,        isWritable: true, isSigner: false },
  { pubkey: playerWoodTa,    isWritable: true, isSigner: false },
  { pubkey: leatherMint,     isWritable: true, isSigner: false },
  { pubkey: playerLeatherTa, isWritable: true, isSigner: false },
];

await craftingProgram.methods
  .craftItem(0, "https://arweave.net/saber")
  .accounts({
    player:             player.publicKey,
    itemMetadata:       itemMetaPda,
    nftMint:            nftMintKp.publicKey,
    metadataAccount:    metaplexMetadataPda,
    masterEdition:      masterEditionPda,
    sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
  })
  .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 })])
  .remainingAccounts(remaining)
  .signers([player, nftMintKp])
  .rpc({ commitment: "confirmed" });
```

### 4. Продаж NFT на Marketplace

```typescript
// Виставити (NFT переходить в escrow PDA)
await marketplaceProgram.methods
  .listItem(new BN(5))   // 5 MagicToken
  .accounts({
    seller:           player.publicKey,
    nftMint:          nftMintKp.publicKey,
    sellerNftAccount: sellerNftAta,
    tokenProgram:     TOKEN_2022_PROGRAM_ID,
  })
  .signers([player])
  .rpc({ commitment: "confirmed" });

// Купити (NFT спалюється, seller отримує MagicToken)
await marketplaceProgram.methods
  .buyItem()
  .accounts({
    buyer:              buyer.publicKey,
    seller:             seller.publicKey,
    nftMint:            nftMintKp.publicKey,
    itemMetadata:       itemMetaPda,
    metadataAccount:    metaplexMetadataPda,
    masterEdition:      masterEditionPda,
    magicMint:          magicMintPda,
    sellerMagicAccount: sellerMagicAta,
    sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
  })
  .signers([buyer])
  .rpc({ commitment: "confirmed" });
```

---

## Безпека

- **mint_authority** ресурсів = GameConfig PDA → прямий мінт через Token Program заблоковано
- **mint_authority** MagicToken = Marketplace Authority PDA → мінт лише через `buy_item` CPI
- **burn** NFT = лише через `buy_item` (listing PDA підписує як authority)
- Всі CPI використовують `new_with_signer` з PDA seeds → зовнішній виклик неможливий
- Owner check на кожному акаунті гравця через constraint або Signer

---

## Структура проєкту

```
programs/
├── resource_manager/   # SPL Token-2022 ресурси
├── magic_token/        # MagicToken
├── item_nft/           # Metaplex NFT
├── crafting/           # Логіка крафту
├── search/             # Пошук ресурсів (таймер)
└── marketplace/        # Купівля/продаж NFT
tests/
└── kozatskyi_biznes.ts # Інтеграційні тести (100% покриття інструкцій)
```
