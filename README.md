# Гра "Козацький бізнес" для Solana

Виконав студент Джос Олексій
Результат деплою можна знайти в utils/account.json

Навчальний проєкт на актуальній Anchor 0.32.1 з шістьма програмами:
resource_manager, item_nft, crafting, search, marketplace, magic_token

Поточний flow: search -> craft -> redeem

Гравець шукає ресурси, крафтить NFT-предмет і обмінює його на reward token

## Program IDs

| Program            | Program ID                                     |
| ------------------ | ---------------------------------------------- |
| resource_manager | CwwxNgkg1s8rjRAAN9zcvLgBCBhXTvCu4L1oAupBqiTe |
| item_nft         | 31YqF1ymwThcZTyGCmx6Uqnvjev15JRkWvMSJoxc3wve |
| crafting         | A14WMVRTuuS4JtVcg22BuiWHvhJx1ZhxJS5CrWfy2tHh |
| search           | 5vrMHniMhyCnZBK5PWTMMF2w886LDc1Kd3GdN17cbPGh |
| marketplace      | 3cPgZBSjpvcuD5FmhGQfCSBFXnz3ZMs573u8UDszgpeW |
| magic_token      | Bvw1CY1ZBu7jE2zmmKkWKe75LfoQvudwT11YxGYaLGW  |

## Game Data

### Resources

Усі базові ресурси використовують Token-2022 mint з decimals = 0

| ID  | Name    | Symbol  |
| --- | ------- | ------- |
| 0 | Wood    | WOOD  |
| 1 | Iron    | IRON  |
| 2 | Gold    | GOLD  |
| 3 | Leather | LETHR |
| 4 | Stone   | STONE |
| 5 | Diamond | DIAM  |

### Items and rewards

| ID  | Item                 | Symbol  | Recipe                        | Reward |
| --- | -------------------- | ------- | ----------------------------- | ------ |
| 0 | Kozak Sabre          | SABRE | 1 WOOD + 3 IRON + 1 LEATHER | 25   |
| 1 | Elder Staff          | STAFF | 2 WOOD + 1 GOLD + 1 DIAMOND | 40   |
| 2 | Characteristic Armor | ARMOR | 2 IRON + 1 GOLD + 4 LEATHER | 75   |
| 3 | Battle Bracelet      | BRACE | 4 IRON + 2 GOLD + 2 DIAMOND | 110  |

### PDA seeds

| PDA                | Seeds                              | Owner program      |
| ------------------ | ---------------------------------- | ------------------ |
| GameConfig       | ["game_config"]                  | resource_manager |
| Player           | ["player", owner]                | search           |
| ProgramAuthority | ["program_authority"]            | per-program        |
| ResourceMint     | ["resource_mint", resource_type] | resource_manager |
| MagicTokenMint   | ["magic_token_mint"]             | magic_token      |
| ItemMetadata     | ["item_metadata", mint]          | item_nft         |

MagicTokenMint тепер є default reward mint

Фактичний reward mint зберігається в GameConfig.reward_token_mint і може бути замінений для тестування

## Reward Token

Підтримуються два режими:

1. Default reward mint - bootstrap створює canonical mint через програму magic_token
2. External reward mint - у bootstrap можна передати власний mint через --reward-mint

## Scripts

| Command                                    | Purpose                                                      |
| ------------------------------------------ | ------------------------------------------------------------ |
| yarn create:mints                        | створює GameConfig, resource mint-и та default reward mint |
| yarn create:mints --reward-mint <pubkey> | використовує зовнішній reward mint замість default mint      |
| yarn bootstrap                           | bootstrap state + Player PDA для поточного wallet          |
| yarn bootstrap --skip-player             | bootstrap без ініціалізації Player                         |
| yarn bootstrap --reward-mint <pubkey>    | bootstrap з уже створеним reward mint                        |
| yarn demo:flow                           | проходить flow search -> craft -> redeem                   |
| yarn demo:flow --reward-mint <pubkey>    | запускає demo flow з external reward mint                    |
| anchor deploy                            | деплоїть програми і запускає migrations/deploy.ts          |

## Localnet

### 1. Install dependencies

yarn install
anchor build

### 2. Start validator

anchor localnet

### 3. Deploy and bootstrap

Default reward mint:

anchor deploy
yarn bootstrap

External reward mint:

anchor deploy
yarn bootstrap --reward-mint <MINT_PUBKEY>

### 4. Run demo flow

yarn demo:flow

## Devnet

### 1. Switch cluster and fund wallet

Поповнити коштами на https://faucet.solana.com/

solana config set --url https://api.devnet.solana.com

### 2. Build and deploy

anchor build
anchor deploy --provider.cluster devnet

### 3. Bootstrap state on devnet

Default reward mint:

ANCHOR_PROVIDER_URL=https://api.devnet.solana.com yarn create:mints

ANCHOR_PROVIDER_URL=https://api.devnet.solana.com yarn bootstrap

External reward mint:

ANCHOR_PROVIDER_URL=https://api.devnet.solana.com yarn bootstrap --reward-mint <MINT_PUBKEY>

## Tests

anchor build
yarn typecheck
yarn test --skip-build

## Repository map

| Path                                   | Purpose                                                               |
| -------------------------------------- | --------------------------------------------------------------------- |
| programs/resource_manager/src/lib.rs | GameConfig, resource mint initialization, authorized mint/burn      |
| programs/search/src/lib.rs           | Player, cooldown, search rewards                                    |
| programs/crafting/src/lib.rs         | recipe validation, resource burn, NFT mint CPI                        |
| programs/item_nft/src/lib.rs         | NFT mint/burn and ItemMetadata                                      |
| programs/marketplace/src/lib.rs      | redeem crafted NFT for configured reward token                        |
| programs/magic_token/src/lib.rs      | default reward mint initialization and marketplace-authorized minting |
| shared/src/lib.rs                    | shared constants, seeds, recipes, errors                              |
| scripts/game.ts                      | reusable bootstrap/search/craft/redeem helpers                        |
| tests/                               | інтеграційні тести для всіх основних flow                             |
