# Deployment Guide - Гра "Козацький бізнес"

## ✅ Статус виконання (Devnet)

Деплой успішно виконано (April 3, 2026).

- deploy wallet: `97HgTrutpW6gAN3ZdqS1TP8zQPPdxvUzoHhBXLpY8sgr`
- upgrade authority: `97HgTrutpW6gAN3ZdqS1TP8zQPPdxvUzoHhBXLpY8sgr`

Program IDs:
- resource_manager: `2Y2tAWf4DGPhk9kTDHyyProMw4wrNJf6R6U61WL8D4Vv`
- item_nft: `9GU3Nb13w1YaA8vwfLo2MqWmakbVLF9G6xZiNqCXn8ns`
- crafting: `CTHKMpMxaV89e4g7a4uwmvPmSYygWvtFn4vv9qRQ5m2t`
- search: `HDtdF8EjnBeRuVFVA3TUQFi3oM8qA8iGCcfrCJbRar1e`
- marketplace: `5EyYkXzfHkH278x25q42csiR8FLeGvujpqCYdhncfcUd`
- magic_token: `4NvPT6ob4cPTGpXDq9TEp5ByuW5HgxAYYCUWW5xDS6dE`

## Перед деплоєм

### 1. Перевірте встановлення інструментів

```bash
# Перевірити Rust
rustc --version

# Перевірити Solana CLI
solana --version
# Має бути ≥ 1.18.0

# Перевірити Anchor
anchor --version
# Має бути 0.29.0

# Перевірити Node.js
node --version
npm --version
```

### 2. Налаштуйте гаманець

```bash
# Перевірити, який гаманець налаштований
solana config get

# Якщо потрібно, встановити новий гаманець
solana config set --keypair ~/.config/solana/id.json
```

## Деплой на Devnet

### Крок 1: Налаштуйте Solana для Devnet

```bash
# Встановити Devnet як активний кластер
solana config set --url devnet

# Перевірити
solana config get
# Має показати: RPC URL: https://api.devnet.solana.com
```

### Крок 2: Отримайте тестові SOL

```bash
# Рекомендовано мати 10+ SOL для деплою всіх 6 програм
solana airdrop 2

# Перевірити баланс
solana balance
```

### Крок 3: Збудуйте програми

```bash
# Перейти в директорію проекту
cd /path/to/HW3

# Встановити залежності
npm install

# Зібрати усі програми
anchor build

# Оберіть виходячи файли у target/deploy/
ls target/deploy/*.so
```

### Крок 4: Отримайте адреси програм

Після компіляції, кожна програма має унікальну адресу (Program ID). Отримайте їх:

```bash
# Знайти всі keypair файли
find target/deploy -name "*.json" -type f

# Для кожного файлу отримаєте ID:
solana address -k target/deploy/resource_manager-keypair.json
solana address -k target/deploy/item_nft-keypair.json
solana address -k target/deploy/crafting-keypair.json
solana address -k target/deploy/search-keypair.json
solana address -k target/deploy/marketplace-keypair.json
solana address -k target/deploy/magic_token-keypair.json
```

### Крок 5: Оновіть Anchor.toml

Замініть placeholder адреси реальними:

```toml
[programs.devnet]
resource_manager = "ВАШ_СПРАВЖНІЙ_ADDRESS"
item_nft = "ВАШ_СПРАВЖНІЙ_ADDRESS"
crafting = "ВАШ_СПРАВЖНІЙ_ADDRESS"
search = "ВАШ_СПРАВЖНІЙ_ADDRESS"
marketplace = "ВАШ_СПРАВЖНІЙ_ADDRESS"
magic_token = "ВАШ_СПРАВЖНІЙ_ADDRESS"
```

### Крок 6: Зробіть деплой

```bash
# Деплой на Devnet
anchor deploy

# Це займе кілька хвилин...
# На завершення ви побачите транзакштейші
```

### Крок 7: Перевірте деплой

```bash
# Перевірити статус програм
solana program show RESOURCE_MANAGER_PROGRAM_ID

# Або за допомогою Solana Explorer
# https://explorer.solana.com/?cluster=devnet
# Шукайте вашу адресу програми
```

## Деплой на Localhost (для тестування)

### Крок 1: Запустіть Solana Validator

```bash
# У одному терміналі, запустіть локальний валідатор
solana-test-validator --reset

# Залишіть його запущеним!
```

### Крок 2: Налаштуйте Solana CLI

```bash
# У новому терміналі налаштуйте localhost
solana config set --url localhost

# Перевірити
solana config get
```

### Крок 3: Отримайте локальні SOL

```bash
# Отримати 1000 SOL (локально)
solana airdrop 1000

# Перевірити баланс
solana balance
```

### Крок 4: Зібудьте програми

```bash
# Перейти в директорію проекту
cd /path/to/HW3

# Встановити залежності (якщо не встановлені)
npm install

# Зібрати
anchor build
```

### Крок 5: Оновіте Anchor.toml для локального деплою

```toml
[provider]
cluster = "localnet"
wallet = "~/.config/solana/id.json"
```

### Крок 6: Зробіть деплой

```bash
# Деплой на localhost
anchor deploy --provider.cluster localnet
```

### Крок 7: Запустіть тести

```bash
# Тести проти локального валідатора
anchor test --skip-local-validator

# Або, якщо локальний валідатор вже запущений:
anchor test
```

## Запуск тестів

### Повне тестування на Devnet

```bash
# Перевірити, що налаштовані на Devnet
solana config get

# Запустити тести
anchor test
```

### Покриття тестами

```bash
# Тести повинні покривати:
# ✅ resource_manager:
#   - Ініціалізація GameConfig
#   - Мінтинг ресурсів
#   - Спалення ресурсів
#   - Обробка помилок

# ✅ search:
#   - Пошук ресурсів
#   - Таймер 60 секунд
#   - Недопущення скорочення часу

# ✅ crafting:
#   - Крафт предметів
#   - Валідація рецептів
#   - Спалення ресурсів при крафті

# ✅ item_nft:
#   - Створення NFT
#   - Спалення NFT
#   - Валідація типів предметів

# ✅ marketplace:
#   - Виставлення предметів
#   - Купівля предметів
#   - Мінтинг MagicToken

# ✅ magic_token:
#   - Мінтинг тільки з Marketplace
#   - Контроль доступу
```

## Верифікація на Solana Explorer

### 1. Перейдіть на Solana Explorer

- **Devnet:** https://explorer.solana.com/?cluster=devnet
- **Localhost:** https://explorer.solana.com/?cluster=custom&customUrl=http://localhost:8899

### 2. Шукайте вашу адресу

Введіть адресу однієї з ваших програм у строку пошуку.

### 3. Перевірте:

- ✅ Програма існує
- ✅ Статус: "Active"
- ✅ Дані: Ваш source code або BPF виконуваний файл
- ✅ Upgrade authority відповідає вашому deploy wallet

## Рішення проблем

### "Program ID was not as expected"

```bash
# Переконайтеся, що Anchor.toml має правильні адреси
cat Anchor.toml | grep -E "resource_manager|item_nft|crafting|search|marketplace|magic_token"

# Перезберіть і передеплойте:
anchor build
anchor deploy
```

### "Insufficient funds"

```bash
# Отримайте більше SOL
solana airdrop 3

# Перевірити баланс
solana balance
```

### "Program not found"

```bash
# Переконайтеся, що програма задеплоєна
solana program show YOUR_PROGRAM_ID

# Якщо не знайдена, зробіть деплой заново
anchor deploy
```

### Помилки компіляції

```bash
# Очистіть артефакти
anchor clean

# Переберіть залежності
cargo update

# Спробуйте снова
anchor build
```

## Наступні кроки

1. **Оновіть README.md** з actual адресами програм
2. **Задокументуйте** структуру своїх IDL файлів
3. **Напишіть скрипти** для ініціалізації ресурсів та тестування
4. **Скачайте IDL** та додайте у проект:
   ```bash
   anchor idl fetch YOUR_PROGRAM_ID -o idl/program.json
   ```

## Корисні команди

```bash
# Показати програми на поточному кластері
anchor info

# Показати лог програм
solana logs YOUR_PROGRAM_ID

# Показати транзакцію
solana confirm YOUR_TRANSACTION_SIGNATURE

# Показати акаунт
solana account YOUR_ACCOUNT_ADDRESS
```

---

**Успіхів з деплоєм! 🚀**
