# Швидкий старт - 5 хвилин

## 1. Клонування та налаштування

```bash
# Клонувати проект
git clone https://github.com/YOUR_USERNAME/task_Solana_2026.git
cd task_Solana_2026

# Встановити npm залежності
npm install

# (Опціонально) Установити утиліти локально
chmod +x scripts/*.sh
```

## 2. Тестування на Localhost (найшвидше)

```bash
# Терміну 1: Запустити локальний Solana validator
solana-test-validator --reset

# Терміну 2: Налаштувати CLI
solana config set --url localhost

# Терміну 3: Отримати тестові SOL та запустити тести
solana airdrop 2 ~/.solana/id.json
anchor test --skip-local-validator
```

✅ **Готово!** Всі тести повинні пройти за ~30 секунд.

## 3. Деплой на Devnet (для сабміту)

```bash
# Переключитися на Devnet
solana config set --url devnet

# Отримати Devnet SOL
solana airdrop 3 ~/.solana/id.json

# Зібрати та задеплоїти
anchor build
anchor deploy

# Запустити тести на Devnet
anchor test
```

Це займе ~5 хвилин. Після завершення отримаєте Program IDs.

## 4. Оновити README

```bash
# Скопіюйте Program IDs з деплою
# Оновіть README.md:

# Program IDs on Devnet:
# resource_manager: <ID1>
# item_nft: <ID2>
# crafting: <ID3>
# search: <ID4>
# marketplace: <ID5>
# magic_token: <ID6>

# Зробіть commit та push
git add README.md
git commit -m "Add Devnet program IDs"
git push origin main
```

## 5. Зробіть Pull Request

```bash
1. Перейдіть на GitHub
2. Нажміть "Pull Request"
3. Заповніть описання з:
   - Посилання на Devnet перевірку або фото
   - Список реалізованих функцій
   - Результати тестів
4. Нажміть "Create Pull Request"
```

## 📋 Контрольний список перед сабміту

- [ ] Усі програми компілюються без помилок
- [ ] 100% тестів проходять на Devnet
- [ ] README оновлений з Program IDs
- [ ] DEPLOYMENT_GUIDE.md готовий
- [ ] Код задокументований (Rust doc comments)
- [ ] .gitignore настроєн правильно
- [ ] Нема приватних ключів у репозиторії

## 🆘 Швидкі рішення

### Помилка: "Program ID was not as expected"
```bash
anchor clean
anchor build
# Оновіть Anchor.toml з новими ID
```

### Помилка: "Insufficient funds"
```bash
solana airdrop 5 ~/.solana/id.json
```

### Тест зависає
```bash
# Ctrl+C щоб вийти
solana-test-validator --reset  # Перезапустіть validator
```

### Не компілюється
```bash
cargo clean
cargo update
anchor build
```

## 📚 Корисні команди

```bash
# Перевірити баланс гамания
solana balance

# Дивитись логи програми
solana logs <PROGRAM_ID> --url devnet

# Перевірити статус програми
solana program show <PROGRAM_ID> --url devnet

# Отримати IDL програми
anchor idl fetch <PROGRAM_ID> -o idl/program.json

# Запустити конкретний тест
anchor test -- --grep "should craft item"
```

## 🎯 Следующие кроки після реалізації

1. **Оптимізаціяzoom:**
   - Зменшите розмір аккаунтів
   - Оптимізуйте CPI чіпи

2. **Додаткові функції:**
   - Торгівля ресурсами
   - Рейтинги гравців
   - Таймер для виліковування предметів

3. **Фронтенд:**
   - React + Anchor.js UI
   - Wallet integration (Phantom/Solflare)
   - Real-time event listening

---

**Успіху! 🚀**
