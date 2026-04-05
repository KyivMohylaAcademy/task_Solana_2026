/** Shared off-chain game helpers used by scripts, tests and the deploy flow. */
import * as anchor from "@coral-xyz/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAccount,
  getAssociatedTokenAddressSync,
  getMint,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import BN from "bn.js";
import { mkdirSync, readFileSync, writeFileSync } from "fs";
import { homedir } from "os";
import path from "path";
import {
  findGameConfigPda,
  findItemMetadataPda,
  findMagicTokenMintPda,
  findPlayerPda,
  findProgramAuthorityPda,
  findResourceMintPda,
} from "../utils/account_utils";
import {
  getGamePrograms,
  getProgramPublicKey,
  PROGRAM_IDS,
  type GamePrograms,
} from "../utils/programs";

/** Canonical Metaplex Token Metadata program ID used by item mints. */
export const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s",
);

/** Search delay with a small buffer so scripts avoid racing the on-chain cooldown. */
export const SEARCH_COOLDOWN_MS = 61_000;
/** Reward table ordered by item type. */
export const ITEM_PRICES = [25, 40, 75, 110] as const;

/** Static metadata for every searchable resource mint. */
export const RESOURCE_DEFINITIONS = [
  {
    resourceType: 0,
    slug: "wood",
    name: "Wood",
    symbol: "WOOD",
    uri: "https://example.com/resources/wood.json",
  },
  {
    resourceType: 1,
    slug: "iron",
    name: "Iron",
    symbol: "IRON",
    uri: "https://example.com/resources/iron.json",
  },
  {
    resourceType: 2,
    slug: "gold",
    name: "Gold",
    symbol: "GOLD",
    uri: "https://example.com/resources/gold.json",
  },
  {
    resourceType: 3,
    slug: "leather",
    name: "Leather",
    symbol: "LETHR",
    uri: "https://example.com/resources/leather.json",
  },
  {
    resourceType: 4,
    slug: "stone",
    name: "Stone",
    symbol: "STONE",
    uri: "https://example.com/resources/stone.json",
  },
  {
    resourceType: 5,
    slug: "diamond",
    name: "Diamond",
    symbol: "DIAM",
    uri: "https://example.com/resources/diamond.json",
  },
] as const;

/** Static metadata, crafting costs and marketplace prices for each item type. */
export const ITEM_DEFINITIONS = [
  {
    itemType: 0,
    slug: "kozak-sabre",
    label: "Kozak Sabre",
    symbol: "SABRE",
    uri: "https://example.com/items/kozak-sabre.json",
    price: ITEM_PRICES[0],
    costs: [1, 3, 0, 1, 0, 0],
  },
  {
    itemType: 1,
    slug: "elder-staff",
    label: "Elder Staff",
    symbol: "STAFF",
    uri: "https://example.com/items/elder-staff.json",
    price: ITEM_PRICES[1],
    costs: [2, 0, 1, 0, 0, 1],
  },
  {
    itemType: 2,
    slug: "characteristic-armor",
    label: "Characteristic Armor",
    symbol: "ARMOR",
    uri: "https://example.com/items/characteristic-armor.json",
    price: ITEM_PRICES[2],
    costs: [0, 2, 1, 4, 0, 0],
  },
  {
    itemType: 3,
    slug: "battle-bracelet",
    label: "Battle Bracelet",
    symbol: "BRACE",
    uri: "https://example.com/items/battle-bracelet.json",
    price: ITEM_PRICES[3],
    costs: [0, 4, 2, 0, 0, 2],
  },
] as const;

/** One resource descriptor entry from `RESOURCE_DEFINITIONS`. */
export type ResourceDefinition = (typeof RESOURCE_DEFINITIONS)[number];
/** One item descriptor entry from `ITEM_DEFINITIONS`. */
export type ItemDefinition = (typeof ITEM_DEFINITIONS)[number];

/** Precomputed off-chain handles required to drive the full game flow. */
export type Workspace = {
  provider: anchor.AnchorProvider;
  programs: GamePrograms & Record<string, any>;
  walletPublicKey: anchor.web3.PublicKey;
  gameConfigPda: anchor.web3.PublicKey;
  resourceManagerAuthority: anchor.web3.PublicKey;
  searchAuthority: anchor.web3.PublicKey;
  craftingAuthority: anchor.web3.PublicKey;
  itemNftAuthority: anchor.web3.PublicKey;
  marketplaceAuthority: anchor.web3.PublicKey;
  magicTokenAuthority: anchor.web3.PublicKey;
  magicTokenMintPda: anchor.web3.PublicKey;
  rewardTokenMint: anchor.web3.PublicKey;
  rewardTokenProgramId: anchor.web3.PublicKey;
  resourceMints: anchor.web3.PublicKey[];
};

/** Optional bootstrap toggles for local setup scripts. */
export type BootstrapOptions = {
  initializePlayer?: boolean;
  writeAccountsSnapshot?: boolean;
  rewardTokenMint?: anchor.web3.PublicKey;
};

/** Result summary returned after ensuring bootstrap state exists. */
export type BootstrapResult = {
  playerPda: anchor.web3.PublicKey | null;
  initializedGameConfig: boolean;
  initializedResourceMints: number[];
  initializedDefaultRewardMint: boolean;
};

/** Details about one item crafted through the off-chain helper flow. */
export type CraftedItem = {
  definition: ItemDefinition;
  mint: anchor.web3.PublicKey;
  itemMetadataPda: anchor.web3.PublicKey;
  metadataPda: anchor.web3.PublicKey;
  masterEditionPda: anchor.web3.PublicKey;
  ownerItemTokenAccount: anchor.web3.PublicKey;
  signature: string;
};

const ACCOUNTS_OUTPUT_PATH = path.resolve(__dirname, "../utils/accounts.json");

/** Creates the default Anchor provider from `ANCHOR_PROVIDER_URL` and `ANCHOR_WALLET`. */
const createDefaultProvider = (): anchor.AnchorProvider => {
  const url = process.env.ANCHOR_PROVIDER_URL ?? "http://127.0.0.1:8899";
  const walletPath =
    process.env.ANCHOR_WALLET ?? path.join(homedir(), ".config/solana/id.json");
  const secretKey = Uint8Array.from(
    JSON.parse(readFileSync(walletPath, "utf8")) as number[],
  );
  const payer = anchor.web3.Keypair.fromSecretKey(secretKey);
  const NodeWallet =
    require("@coral-xyz/anchor/dist/cjs/nodewallet.js").default;
  const wallet = new NodeWallet(payer);
  const options = anchor.AnchorProvider.defaultOptions();
  const connection = new anchor.web3.Connection(url, options.commitment);

  return new anchor.AnchorProvider(connection, wallet, options);
};

/** Creates a reusable workspace with program clients, PDAs and derived mint addresses. */
export const createWorkspace = (
  provider: anchor.AnchorProvider = createDefaultProvider(),
): Workspace => {
  anchor.setProvider(provider);

  const programs = getGamePrograms(provider) as GamePrograms &
    Record<string, any>;
  const [gameConfigPda] = findGameConfigPda();
  const [resourceManagerAuthority] = findProgramAuthorityPda(
    getProgramPublicKey("resource_manager"),
  );
  const [searchAuthority] = findProgramAuthorityPda(
    getProgramPublicKey("search"),
  );
  const [craftingAuthority] = findProgramAuthorityPda(
    getProgramPublicKey("crafting"),
  );
  const [itemNftAuthority] = findProgramAuthorityPda(
    getProgramPublicKey("item_nft"),
  );
  const [marketplaceAuthority] = findProgramAuthorityPda(
    getProgramPublicKey("marketplace"),
  );
  const [magicTokenAuthority] = findProgramAuthorityPda(
    getProgramPublicKey("magic_token"),
  );
  const [magicTokenMintPda] = findMagicTokenMintPda();
  const resourceMints = RESOURCE_DEFINITIONS.map(
    ({ resourceType }) => findResourceMintPda(resourceType)[0],
  );

  return {
    provider,
    programs,
    walletPublicKey: provider.wallet.publicKey,
    gameConfigPda,
    resourceManagerAuthority,
    searchAuthority,
    craftingAuthority,
    itemNftAuthority,
    marketplaceAuthority,
    magicTokenAuthority,
    magicTokenMintPda,
    rewardTokenMint: magicTokenMintPda,
    rewardTokenProgramId: TOKEN_2022_PROGRAM_ID,
    resourceMints,
  };
};

/** Normalizes Anchor numeric wrappers into a plain JavaScript number. */
export const toNumber = (
  value: number | bigint | BN | { toNumber?: () => number; toString(): string },
): number => {
  if (typeof value === "number") {
    return value;
  }

  if (typeof value === "bigint") {
    return Number(value);
  }

  if ("toNumber" in value && typeof value.toNumber === "function") {
    return value.toNumber();
  }

  return Number(value.toString());
};

/** Returns whether the RPC endpoint appears to be a local validator. */
const isLocalEndpoint = (endpoint: string): boolean => {
  return endpoint.includes("127.0.0.1") || endpoint.includes("localhost");
};

/** Sleeps for the provided number of milliseconds. */
const sleep = async (ms: number) =>
  new Promise((resolve) => {
    setTimeout(resolve, ms);
  });

/** Airdrops SOL to the active wallet when running against a local validator. */
export const ensureWalletFunded = async (
  workspace: Workspace,
  minimumLamports = 2 * anchor.web3.LAMPORTS_PER_SOL,
): Promise<void> => {
  const { provider, walletPublicKey } = workspace;
  const balance = await provider.connection.getBalance(
    walletPublicKey,
    "confirmed",
  );
  if (
    balance >= minimumLamports ||
    !isLocalEndpoint(provider.connection.rpcEndpoint)
  ) {
    return;
  }

  const signature = await provider.connection.requestAirdrop(
    walletPublicKey,
    5 * anchor.web3.LAMPORTS_PER_SOL,
  );
  await provider.connection.confirmTransaction(signature, "confirmed");
};

/** Initializes the canonical on-chain bootstrap state required by the game. */
export const ensureBootstrap = async (
  workspace: Workspace,
  options: BootstrapOptions = {},
): Promise<BootstrapResult> => {
  const {
    initializePlayer = true,
    writeAccountsSnapshot = true,
    rewardTokenMint: requestedRewardTokenMint,
  } = options;
  const { provider, programs, walletPublicKey } = workspace;
  const resourceManagerProgram = programs.resource_manager as any;
  const magicTokenProgram = programs.magic_token as any;
  const craftingProgram = programs.crafting as any;
  const marketplaceProgram = programs.marketplace as any;
  const searchProgram = programs.search as any;

  await ensureWalletFunded(workspace);

  let initializedGameConfig = false;
  const existingGameConfig =
    await resourceManagerProgram.account.gameConfig.fetchNullable(
      workspace.gameConfigPda,
    );
  const configuredRewardMint =
    requestedRewardTokenMint ?? workspace.rewardTokenMint;

  if (!existingGameConfig) {
    await resourceManagerProgram.methods
      .initializeGameConfig(
        configuredRewardMint,
        ITEM_PRICES.map((value) => new BN(value)),
      )
      .accounts({
        admin: walletPublicKey,
        gameConfig: workspace.gameConfigPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    initializedGameConfig = true;
  }

  const gameConfig = initializedGameConfig
    ? await resourceManagerProgram.account.gameConfig.fetch(
        workspace.gameConfigPda,
      )
    : existingGameConfig;
  const activeRewardTokenMint = new anchor.web3.PublicKey(
    gameConfig.rewardTokenMint,
  );

  if (
    requestedRewardTokenMint &&
    !requestedRewardTokenMint.equals(activeRewardTokenMint)
  ) {
    throw new Error(
      `GameConfig already uses reward mint ${activeRewardTokenMint.toBase58()}, requested ${requestedRewardTokenMint.toBase58()}.`,
    );
  }

  workspace.rewardTokenMint = activeRewardTokenMint;

  const initializedResourceMints: number[] = [];
  for (const resource of RESOURCE_DEFINITIONS) {
    const resourceMint = workspace.resourceMints[resource.resourceType];
    const existingMint = await provider.connection.getAccountInfo(
      resourceMint,
      "confirmed",
    );

    if (existingMint) {
      continue;
    }

    await resourceManagerProgram.methods
      .initializeResourceMint(
        resource.resourceType,
        resource.name,
        resource.symbol,
        resource.uri,
      )
      .accounts({
        admin: walletPublicKey,
        gameConfig: workspace.gameConfigPda,
        programAuthority: workspace.resourceManagerAuthority,
        resourceMint,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    initializedResourceMints.push(resource.resourceType);
  }

  let initializedDefaultRewardMint = false;
  const usingDefaultRewardMint = workspace.rewardTokenMint.equals(
    workspace.magicTokenMintPda,
  );
  const existingRewardMint = await provider.connection.getAccountInfo(
    workspace.rewardTokenMint,
    "confirmed",
  );

  if (!existingRewardMint) {
    if (!usingDefaultRewardMint) {
      throw new Error(
        `Configured reward mint ${workspace.rewardTokenMint.toBase58()} does not exist.`,
      );
    }

    await magicTokenProgram.methods
      .initializeMagicTokenMint()
      .accounts({
        admin: walletPublicKey,
        gameConfig: workspace.gameConfigPda,
        programAuthority: workspace.magicTokenAuthority,
        magicTokenMint: workspace.magicTokenMintPda,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    initializedDefaultRewardMint = true;
  }

  const rewardMintAccountInfo = await provider.connection.getAccountInfo(
    workspace.rewardTokenMint,
    "confirmed",
  );
  if (!rewardMintAccountInfo) {
    throw new Error(
      `Configured reward mint ${workspace.rewardTokenMint.toBase58()} could not be loaded after bootstrap.`,
    );
  }

  workspace.rewardTokenProgramId = rewardMintAccountInfo.owner;
  if (
    !workspace.rewardTokenProgramId.equals(TOKEN_PROGRAM_ID) &&
    !workspace.rewardTokenProgramId.equals(TOKEN_2022_PROGRAM_ID)
  ) {
    throw new Error(
      `Reward mint ${workspace.rewardTokenMint.toBase58()} is not owned by SPL Token or Token-2022.`,
    );
  }
  const rewardMintInfo = await getMint(
    provider.connection,
    workspace.rewardTokenMint,
    "confirmed",
    workspace.rewardTokenProgramId,
  );

  if (!rewardMintInfo.mintAuthority?.equals(workspace.magicTokenAuthority)) {
    throw new Error(
      `Reward mint ${workspace.rewardTokenMint.toBase58()} must use ${workspace.magicTokenAuthority.toBase58()} as mint authority.`,
    );
  }

  await craftingProgram.methods.initialize().rpc();
  await marketplaceProgram.methods.initialize().rpc();

  let playerPda: anchor.web3.PublicKey | null = null;
  if (initializePlayer) {
    playerPda = findPlayerPda(walletPublicKey)[0];
    const existingPlayer =
      await searchProgram.account.player.fetchNullable(playerPda);

    if (!existingPlayer) {
      await searchProgram.methods
        .initPlayer()
        .accounts({
          owner: walletPublicKey,
          player: playerPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
    }
  }

  if (writeAccountsSnapshot) {
    writeAccountsFile(workspace, playerPda);
  }

  return {
    playerPda,
    initializedGameConfig,
    initializedResourceMints,
    initializedDefaultRewardMint,
  };
};

/** Persists a JSON snapshot of program IDs and derived PDAs for scripts/tests. */
const writeAccountsFile = (
  workspace: Workspace,
  playerPda: anchor.web3.PublicKey | null,
): void => {
  mkdirSync(path.dirname(ACCOUNTS_OUTPUT_PATH), { recursive: true });

  const payload = {
    clusterUrl: workspace.provider.connection.rpcEndpoint,
    generatedAt: new Date().toISOString(),
    wallet: workspace.walletPublicKey.toBase58(),
    programIds: PROGRAM_IDS,
    pdas: {
      gameConfig: workspace.gameConfigPda.toBase58(),
      resourceManagerAuthority: workspace.resourceManagerAuthority.toBase58(),
      searchAuthority: workspace.searchAuthority.toBase58(),
      craftingAuthority: workspace.craftingAuthority.toBase58(),
      itemNftAuthority: workspace.itemNftAuthority.toBase58(),
      marketplaceAuthority: workspace.marketplaceAuthority.toBase58(),
      magicTokenAuthority: workspace.magicTokenAuthority.toBase58(),
      player: playerPda?.toBase58() ?? null,
    },
    resourceMints: Object.fromEntries(
      RESOURCE_DEFINITIONS.map((resource) => [
        resource.slug,
        workspace.resourceMints[resource.resourceType].toBase58(),
      ]),
    ),
    defaultRewardTokenMint: workspace.magicTokenMintPda.toBase58(),
    rewardTokenMint: workspace.rewardTokenMint.toBase58(),
    rewardTokenProgramId: workspace.rewardTokenProgramId.toBase58(),
    magicTokenMint: workspace.rewardTokenMint.toBase58(),
  };

  writeFileSync(
    ACCOUNTS_OUTPUT_PATH,
    `${JSON.stringify(payload, null, 2)}\n`,
    "utf8",
  );
};

/** Reads all player resource balances in the canonical resource order. */
export const readResourceBalances = async (
  workspace: Workspace,
  owner: anchor.web3.PublicKey = workspace.walletPublicKey,
): Promise<bigint[]> => {
  return Promise.all(
    workspace.resourceMints.map(async (mint) => {
      const tokenAccount = getAssociatedTokenAddressSync(
        mint,
        owner,
        false,
        TOKEN_2022_PROGRAM_ID,
      );
      const accountInfo = await workspace.provider.connection.getAccountInfo(
        tokenAccount,
        "confirmed",
      );

      if (!accountInfo) {
        return 0n;
      }

      const account = await getAccount(
        workspace.provider.connection,
        tokenAccount,
        "confirmed",
        TOKEN_2022_PROGRAM_ID,
      );
      return account.amount;
    }),
  );
};

/** Formats resource balances into a compact `WOOD=1, IRON=2` string. */
export const formatResourceBalances = (balances: readonly bigint[]): string => {
  return RESOURCE_DEFINITIONS.map(
    (resource, index) => `${resource.symbol}=${balances[index].toString()}`,
  ).join(", ");
};

/** Returns the first craftable item for the given balances and optional preference. */
export const findCraftableItem = (
  balances: readonly bigint[],
  preferredItemType?: number,
): ItemDefinition | null => {
  const candidates =
    typeof preferredItemType === "number"
      ? ITEM_DEFINITIONS.filter((item) => item.itemType === preferredItemType)
      : ITEM_DEFINITIONS;

  return (
    candidates.find((item) =>
      item.costs.every((requiredAmount, resourceType) => {
        return balances[resourceType] >= BigInt(requiredAmount);
      }),
    ) ?? null
  );
};

/** Builds the account map required by the `searchResources` instruction. */
const buildSearchAccounts = (workspace: Workspace) => {
  const [playerPda] = findPlayerPda(workspace.walletPublicKey);
  const resourceTokenAccounts = workspace.resourceMints.map((mint) =>
    getAssociatedTokenAddressSync(
      mint,
      workspace.walletPublicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
    ),
  );

  return {
    owner: workspace.walletPublicKey,
    player: playerPda,
    gameConfig: workspace.gameConfigPda,
    searchAuthority: workspace.searchAuthority,
    resourceManagerAuthority: workspace.resourceManagerAuthority,
    woodMint: workspace.resourceMints[0],
    ironMint: workspace.resourceMints[1],
    goldMint: workspace.resourceMints[2],
    leatherMint: workspace.resourceMints[3],
    stoneMint: workspace.resourceMints[4],
    diamondMint: workspace.resourceMints[5],
    woodTokenAccount: resourceTokenAccounts[0],
    ironTokenAccount: resourceTokenAccounts[1],
    goldTokenAccount: resourceTokenAccounts[2],
    leatherTokenAccount: resourceTokenAccounts[3],
    stoneTokenAccount: resourceTokenAccounts[4],
    diamondTokenAccount: resourceTokenAccounts[5],
    resourceManagerProgram: getProgramPublicKey("resource_manager"),
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    tokenProgram: TOKEN_2022_PROGRAM_ID,
    systemProgram: anchor.web3.SystemProgram.programId,
  };
};

/** Waits until the on-chain search cooldown has definitely expired for the wallet. */
export const waitForSearchCooldown = async (
  workspace: Workspace,
): Promise<void> => {
  const [playerPda] = findPlayerPda(workspace.walletPublicKey);
  const playerAccount = await (
    workspace.programs.search as any
  ).account.player.fetchNullable(playerPda);

  if (!playerAccount) {
    return;
  }

  const nextSearchAt = toNumber(playerAccount.lastSearchTimestamp) + 60;
  const now = Math.floor(Date.now() / 1000);
  const secondsRemaining = nextSearchAt - now;

  if (secondsRemaining > 0) {
    await sleep((secondsRemaining + 1) * 1000);
  }
};

/** Executes one search action and waits for the transaction to confirm. */
export const performSearch = async (workspace: Workspace): Promise<string> => {
  await waitForSearchCooldown(workspace);

  const signature = await workspace.programs.search.methods
    .searchResources()
    .preInstructions([
      anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
        units: 400_000,
      }),
    ])
    .accounts(buildSearchAccounts(workspace))
    .rpc();
  await workspace.provider.connection.confirmTransaction(
    signature,
    "confirmed",
  );
  return signature;
};

/** Derives the Metaplex metadata PDA for a mint. */
const deriveMetadataPda = (
  mint: anchor.web3.PublicKey,
): anchor.web3.PublicKey => {
  return anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
    ],
    TOKEN_METADATA_PROGRAM_ID,
  )[0];
};

/** Derives the Metaplex master edition PDA for a mint. */
const deriveMasterEditionPda = (
  mint: anchor.web3.PublicKey,
): anchor.web3.PublicKey => {
  return anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
      Buffer.from("edition"),
    ],
    TOKEN_METADATA_PROGRAM_ID,
  )[0];
};

/** Crafts one item NFT by burning the recipe resources from the current wallet. */
export const craftItem = async (
  workspace: Workspace,
  definition: ItemDefinition,
): Promise<CraftedItem> => {
  const mint = anchor.web3.Keypair.generate();
  const [itemMetadataPda] = findItemMetadataPda(mint.publicKey);
  const metadataPda = deriveMetadataPda(mint.publicKey);
  const masterEditionPda = deriveMasterEditionPda(mint.publicKey);
  const ownerItemTokenAccount = getAssociatedTokenAddressSync(
    mint.publicKey,
    workspace.walletPublicKey,
    false,
    TOKEN_PROGRAM_ID,
  );
  const remainingAccounts = definition.costs.flatMap(
    (requiredAmount, resourceType) => {
      if (requiredAmount === 0) {
        return [];
      }

      return [
        {
          pubkey: workspace.resourceMints[resourceType],
          isWritable: true,
          isSigner: false,
        },
        {
          pubkey: getAssociatedTokenAddressSync(
            workspace.resourceMints[resourceType],
            workspace.walletPublicKey,
            false,
            TOKEN_2022_PROGRAM_ID,
          ),
          isWritable: true,
          isSigner: false,
        },
      ];
    },
  );

  const signature = await workspace.programs.crafting.methods
    .craftItem(definition.itemType)
    .preInstructions([
      anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
        units: 500_000,
      }),
    ])
    .accounts({
      owner: workspace.walletPublicKey,
      gameConfig: workspace.gameConfigPda,
      craftingAuthority: workspace.craftingAuthority,
      itemNftAuthority: workspace.itemNftAuthority,
      mint: mint.publicKey,
      itemMetadata: itemMetadataPda,
      metadata: metadataPda,
      masterEdition: masterEditionPda,
      ownerItemTokenAccount,
      tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
      resourceManagerProgram: getProgramPublicKey("resource_manager"),
      itemNftProgram: getProgramPublicKey("item_nft"),
      resourceTokenProgram: TOKEN_2022_PROGRAM_ID,
      itemTokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
      sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
    })
    .remainingAccounts(remainingAccounts)
    .signers([mint])
    .rpc();
  await workspace.provider.connection.confirmTransaction(
    signature,
    "confirmed",
  );

  return {
    definition,
    mint: mint.publicKey,
    itemMetadataPda,
    metadataPda,
    masterEditionPda,
    ownerItemTokenAccount,
    signature,
  };
};

/** Reads the player's reward-token balance from the configured ATA. */
export const readRewardBalance = async (
  workspace: Workspace,
  owner: anchor.web3.PublicKey = workspace.walletPublicKey,
): Promise<bigint> => {
  const ata = getAssociatedTokenAddressSync(
    workspace.rewardTokenMint,
    owner,
    false,
    workspace.rewardTokenProgramId,
  );
  const accountInfo = await workspace.provider.connection.getAccountInfo(
    ata,
    "confirmed",
  );
  if (!accountInfo) {
    return 0n;
  }

  const account = await getAccount(
    workspace.provider.connection,
    ata,
    "confirmed",
    workspace.rewardTokenProgramId,
  );
  return account.amount;
};

/** Backward-compatible alias for callers that still use the old helper name. */
export const readMagicBalance = readRewardBalance;

/** Redeems a crafted item NFT for reward tokens and returns the observed reward delta. */
export const redeemItem = async (
  workspace: Workspace,
  craftedItem: CraftedItem,
): Promise<{ signature: string; reward: bigint }> => {
  const playerMagicTokenAccount = getAssociatedTokenAddressSync(
    workspace.rewardTokenMint,
    workspace.walletPublicKey,
    false,
    workspace.rewardTokenProgramId,
  );
  const beforeBalance = await readRewardBalance(workspace);

  const signature = await workspace.programs.marketplace.methods
    .redeemItemForMagic(craftedItem.definition.itemType)
    .preInstructions([
      anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
        units: 500_000,
      }),
    ])
    .accounts({
      owner: workspace.walletPublicKey,
      gameConfig: workspace.gameConfigPda,
      marketplaceAuthority: workspace.marketplaceAuthority,
      magicTokenAuthority: workspace.magicTokenAuthority,
      mint: craftedItem.mint,
      itemMetadata: craftedItem.itemMetadataPda,
      metadata: craftedItem.metadataPda,
      masterEdition: craftedItem.masterEditionPda,
      ownerItemTokenAccount: craftedItem.ownerItemTokenAccount,
      magicTokenMint: workspace.rewardTokenMint,
      playerMagicTokenAccount,
      tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
      itemNftProgram: getProgramPublicKey("item_nft"),
      magicTokenProgram: getProgramPublicKey("magic_token"),
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      itemTokenProgram: TOKEN_PROGRAM_ID,
      magicTokenTokenProgram: workspace.rewardTokenProgramId,
      systemProgram: anchor.web3.SystemProgram.programId,
      sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
    })
    .rpc();
  await workspace.provider.connection.confirmTransaction(
    signature,
    "confirmed",
  );

  const afterBalance = await readRewardBalance(workspace);

  return {
    signature,
    reward: afterBalance - beforeBalance,
  };
};
