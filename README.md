<<<<<<< HEAD
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
=======
# Козацький бізнес — Solana Smart Contracts

Гра реалізована як набір смарт-контрактів на Solana з використанням Anchor Framework.

## Program IDs (Devnet)

| Програма | Program ID |
|----------|-----------|
| resource_manager | `C9jeF5eivo4126iDkktjdGk7MEJqNwY9V2pFXMwQYMcy` |
| magic_token | `BQAqENU5HMGNF8Xunzbb859GCTz8v8Tuknqieqqk6ide` |
| search_program | `7qyvBgEsWYpP5UZKhctCA2C6HuVDBFo4DJH6V2P96rPx` |
| item_nft | `HMCgFhEqKWroNqsDNo1RmMsyR7Wky2J7CtfDQf32WHKR` |
| crafting | `EfvmR78Gm6o8dwTpBDMicigDREQFfvPd7nmW8VknbqK3` |
| marketplace | `FBKAbyCSWv1Vm7PVw1NRGWnfH9rpLXqJeP8rNvrRXAkf` |

## Архітектура

- **resource_manager** — керує 6 ресурсами (WOOD, IRON, GOLD, LEATHER, STONE, DIAMOND) як SPL Token-2022
- **magic_token** — SPL токен винагороди, мінтується тільки через marketplace
- **search_program** — гравець шукає ресурси раз на 60 секунд (on-chain таймер)
- **item_nft** — керування NFT предметами (Шабля, Посох, Броня, Браслет)
- **crafting** — крафт предметів: спалює ресурси → створює NFT
- **marketplace** — продаж предметів за MagicToken

## Встановлення

```bash
# Залежності
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest && avm use latest

# Клонування та збірка
git clone <your-repo-url>
cd cossack_business
yarn install
anchor build
```

## Деплой

```bash
solana config set --url devnet
solana airdrop 5
anchor deploy
```

## Тестування

```bash
anchor test
```

## Рецепти крафту
>>>>>>> 02f203a (feat: Козацький бізнес - 6 Anchor programs on Solana Devnet)

| Предмет | Рецепт |
|---------|--------|
| Шабля козака | 3× Залізо + 1× Дерево + 1× Шкіра |
| Посох старійшини | 2× Дерево + 1× Золото + 1× Алмаз |
<<<<<<< HEAD
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

=======
| Броня характерника | 4× Шкіра + 2× Залізо + 1× Золото |
| Бойовий браслет | 4× Залізо + 2× Золото + 2× Алмаз |

## Технічний стек

- Rust + Anchor Framework 1.0.0
- SPL Token-2022
- Solana Devnet
- TypeScript тести
>>>>>>> 02f203a (feat: Козацький бізнес - 6 Anchor programs on Solana Devnet)
