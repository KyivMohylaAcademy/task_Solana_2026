/** End-to-end demo script that searches, crafts and redeems an item NFT. */
import * as anchor from "@coral-xyz/anchor";
import {
  craftItem,
  createWorkspace,
  ensureBootstrap,
  findCraftableItem,
  formatResourceBalances,
  ITEM_DEFINITIONS,
  performSearch,
  readResourceBalances,
  redeemItem,
  SEARCH_COOLDOWN_MS,
} from "./game";

/** Reads a numeric CLI option passed as `--flag value`. */
const readNumericOption = (flag: string): number | undefined => {
  const index = process.argv.indexOf(flag);
  if (index === -1) {
    return undefined;
  }

  const rawValue = process.argv[index + 1];
  if (!rawValue) {
    throw new Error(`Missing value for ${flag}`);
  }

  return Number(rawValue);
};

/** Reads a public-key CLI option passed as `--flag value`. */
const readPublicKeyOption = (
  flag: string,
): anchor.web3.PublicKey | undefined => {
  const index = process.argv.indexOf(flag);
  if (index === -1) {
    return undefined;
  }

  const rawValue = process.argv[index + 1];
  if (!rawValue) {
    throw new Error(`Missing value for ${flag}`);
  }

  return new anchor.web3.PublicKey(rawValue);
};

/** Runs the interactive demo flow from resource search to marketplace redemption. */
const main = async () => {
  const preferredItemType = readNumericOption("--item-type");
  const maxSearches = readNumericOption("--max-searches") ?? 12;
  const rewardTokenMint = readPublicKeyOption("--reward-mint");
  const workspace = createWorkspace();

  if (
    typeof preferredItemType === "number" &&
    !ITEM_DEFINITIONS.some((item) => item.itemType === preferredItemType)
  ) {
    throw new Error("--item-type must be between 0 and 3");
  }

  await ensureBootstrap(workspace, {
    initializePlayer: true,
    writeAccountsSnapshot: true,
    rewardTokenMint,
  });

  let balances = await readResourceBalances(workspace);
  let selectedItem = findCraftableItem(balances, preferredItemType);

  console.log(`Starting balances: ${formatResourceBalances(balances)}`);

  for (let attempt = 1; !selectedItem && attempt <= maxSearches; attempt += 1) {
    console.log(`Search ${attempt}/${maxSearches}...`);
    const signature = await performSearch(workspace);
    balances = await readResourceBalances(workspace);
    selectedItem = findCraftableItem(balances, preferredItemType);

    console.log(`Search tx: ${signature}`);
    console.log(`Balances: ${formatResourceBalances(balances)}`);

    if (!selectedItem && attempt < maxSearches) {
      console.log(
        `No craftable recipe yet. Waiting about ${Math.round(
          SEARCH_COOLDOWN_MS / 1000,
        )} seconds before the next search.`,
      );
    }
  }

  if (!selectedItem) {
    throw new Error(
      `No craftable recipe after ${maxSearches} searches. Current balances: ${formatResourceBalances(
        balances,
      )}`,
    );
  }

  console.log(`Crafting ${selectedItem.label}...`);
  const craftedItem = await craftItem(workspace, selectedItem);
  console.log(`Craft tx: ${craftedItem.signature}`);
  console.log(`Crafted mint: ${craftedItem.mint.toBase58()}`);

  console.log(`Redeeming ${selectedItem.label} for reward tokens...`);
  const redeemResult = await redeemItem(workspace, craftedItem);
  console.log(`Redeem tx: ${redeemResult.signature}`);
  console.log(`Reward mint: ${workspace.rewardTokenMint.toBase58()}`);
  console.log(`Reward amount: ${redeemResult.reward.toString()}`);
};

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
