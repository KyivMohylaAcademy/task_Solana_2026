import { execSync } from "child_process";

const tests = [
  "tests/cases/resource_manager/init_mints.test.ts",
  "tests/cases/resource_manager/initialize.test.ts",
  "tests/cases/magic_token/magic_token.test.ts",
  "tests/cases/item_nft/item_nft.test.ts",
  "tests/cases/search/search.test.ts",
  "tests/cases/crafting/crafting.test.ts",
  "tests/cases/marketplace/marketplace.test.ts",
];

// Propagate Anchor environment variables to child processes.
// anchor test sets ANCHOR_PROVIDER_URL and ANCHOR_WALLET before running this script.
const env = {
  ...process.env,
  ANCHOR_PROVIDER_URL: process.env.ANCHOR_PROVIDER_URL ?? "http://localhost:8899",
  ANCHOR_WALLET: process.env.ANCHOR_WALLET ?? `${process.env.HOME}/.config/solana/id.json`,
};

for (const t of tests) {
  console.log(`\n=== ${t} ===`);
  execSync(`npx ts-mocha -p ./tsconfig.json -t 1000000 ${t}`, { stdio: "inherit", env });
}
