import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  createAssociatedTokenAccountInstruction,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAccount,
  createAssociatedTokenAccountIdempotentInstruction,
  createTransferCheckedInstruction,
} from "@solana/spl-token";
import { expect } from "chai";
import { BN } from "bn.js";

// Metaplex Token Metadata Program ID
const TOKEN_METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

const ITEM_RECIPES: Array<Array<[number, number]>> = [
  [
    [1, 3],
    [0, 1],
    [3, 1],
  ],
  [
    [0, 2],
    [2, 1],
    [5, 1],
  ],
  [
    [3, 4],
    [1, 2],
    [2, 1],
  ],
  [
    [1, 4],
    [2, 2],
    [5, 2],
  ],
];

const SHORT_ITEM_METADATA = [
  { name: "SAB", symbol: "SAB", uri: "https://e/0" },
  { name: "STF", symbol: "STF", uri: "https://e/1" },
  { name: "ARM", symbol: "ARM", uri: "https://e/2" },
  { name: "BRC", symbol: "BRC", uri: "https://e/3" },
];

function loadWallet(): anchor.Wallet {
  const walletCandidates = [
    process.env.ANCHOR_WALLET,
    path.resolve(".anchor", "test-ledger-8897", "validator-keypair.json"),
    process.env.USERPROFILE
      ? path.join(process.env.USERPROFILE, ".config", "solana", "id.json")
      : undefined,
    process.env.HOME
      ? path.join(process.env.HOME, ".config", "solana", "id.json")
      : undefined,
    path.join(os.homedir(), ".config", "solana", "id.json"),
    "/mnt/c/Users/artem/.config/solana/id.json",
  ].filter((candidate): candidate is string => Boolean(candidate));
  const walletPath = walletCandidates.find((candidate) =>
    fs.existsSync(candidate)
  );

  if (!walletPath) {
    throw new Error("Unable to locate a Solana wallet for Anchor tests");
  }

  const walletBytes = JSON.parse(fs.readFileSync(walletPath, "utf8"));
  return new anchor.Wallet(
    Keypair.fromSecretKey(Uint8Array.from(walletBytes))
  );
}

function buildProvider(): anchor.AnchorProvider {
  if (process.env.ANCHOR_PROVIDER_URL) {
    const providerUrl = process.env.ANCHOR_PROVIDER_URL;
    const wallet = loadWallet();
    const options = {
      ...anchor.AnchorProvider.defaultOptions(),
      commitment: "processed" as const,
      preflightCommitment: "processed" as const,
    };

    return new anchor.AnchorProvider(
      new anchor.web3.Connection(providerUrl, {
        commitment: "processed",
        wsEndpoint: resolveWsUrl(providerUrl),
      }),
      wallet,
      options
    );
  }

  const providerUrl = resolveProviderUrl();
  const wallet = loadWallet();

  const options = {
    ...anchor.AnchorProvider.defaultOptions(),
    commitment: "processed" as const,
    preflightCommitment: "processed" as const,
  };

  return new anchor.AnchorProvider(
    new anchor.web3.Connection(providerUrl, {
      commitment: "processed",
      wsEndpoint: resolveWsUrl(providerUrl),
    }),
    wallet,
    options
  );
}

function resolveProviderUrl(): string {
  const wslGateway = resolveWslGateway();
  if (wslGateway) {
    return `http://${wslGateway}:8897`;
  }

  return "http://127.0.0.1:8897";
}

function resolveWsUrl(providerUrl: string): string {
  const url = new URL(providerUrl);
  const rpcPort = Number(url.port || (url.protocol === "https:" ? 443 : 80));
  const wsProtocol = url.protocol === "https:" ? "wss:" : "ws:";
  return `${wsProtocol}//${url.hostname}:${rpcPort + 1}`;
}

function resolveWslGateway(): string | null {
  const routePath = "/proc/net/route";
  if (!fs.existsSync(routePath)) {
    return null;
  }

  const lines = fs.readFileSync(routePath, "utf8").trim().split("\n").slice(1);
  for (const line of lines) {
    const parts = line.trim().split(/\s+/);
    if (parts[1] !== "00000000" || !parts[2]) {
      continue;
    }

    const octets = parts[2]
      .match(/../g)
      ?.reverse()
      .map((octet) => parseInt(octet, 16));
    if (!octets || octets.length !== 4 || octets.some(Number.isNaN)) {
      continue;
    }

    return octets.join(".");
  }

  return null;
}

async function fundAccount(
  provider: anchor.AnchorProvider,
  recipient: PublicKey,
  lamports: number
): Promise<void> {
  const tx = new Transaction().add(
    SystemProgram.transfer({
      fromPubkey: provider.wallet.publicKey,
      toPubkey: recipient,
      lamports,
    })
  );

  await provider.sendAndConfirm(tx, []);
}

function getErrorText(err: unknown): string {
  const candidate = err as any;
  const parts = [
    candidate?.error?.errorCode?.code,
    candidate?.error?.errorMessage,
    candidate?.message,
    Array.isArray(candidate?.logs) ? candidate.logs.join("\n") : undefined,
    err !== undefined ? String(err) : undefined,
  ].filter((part): part is string => Boolean(part));

  return parts.join("\n");
}

function expectAnchorError(err: unknown, ...patterns: string[]): void {
  const errorText = getErrorText(err).toLowerCase();
  expect(
    patterns.some((pattern) => errorText.includes(pattern.toLowerCase())),
    `Expected one of [${patterns.join(", ")}] in error:\n${getErrorText(err)}`
  ).to.equal(true);
}

async function fetchWithRetry<T>(
  fetcher: () => Promise<T>,
  attempts = 10,
  delayMs = 250
): Promise<T> {
  let lastError: unknown;

  for (let attempt = 0; attempt < attempts; attempt++) {
    try {
      return await fetcher();
    } catch (err) {
      lastError = err;
      if (attempt < attempts - 1) {
        await new Promise((resolve) => setTimeout(resolve, delayMs));
      }
    }
  }

  throw lastError;
}

describe("Козацький бізнес", () => {
  // Налаштування провайдера
  const provider = buildProvider();
  anchor.setProvider(provider);

  // Програми
  const resourceManager = anchor.workspace.ResourceManager as Program;
  const magicToken = anchor.workspace.MagicToken as Program;
  const itemNft = anchor.workspace.ItemNft as Program;
  const search = anchor.workspace.Search as Program;
  const crafting = anchor.workspace.Crafting as Program;
  const marketplace = anchor.workspace.Marketplace as Program;

  // Адміністратор (wallet провайдера)
  const admin = provider.wallet as anchor.Wallet;

  // Гравець (нова keypair)
  const player = Keypair.generate();

  // Keypairs для ресурсних мінтів (Token-2022)
  const resourceMints: Keypair[] = Array.from({ length: 6 }, () =>
    Keypair.generate()
  );

  // Keypair для MagicToken мінт
  const magicMint = Keypair.generate();

  // Назви та символи ресурсів
  const RESOURCES = [
    { name: "Дерево", symbol: "WOOD", uri: "https://example.com/wood.json" },
    { name: "Залізо", symbol: "IRON", uri: "https://example.com/iron.json" },
    { name: "Золото", symbol: "GOLD", uri: "https://example.com/gold.json" },
    { name: "Шкіра", symbol: "LEATHER", uri: "https://example.com/leather.json" },
    { name: "Камінь", symbol: "STONE", uri: "https://example.com/stone.json" },
    { name: "Алмаз", symbol: "DIAMOND", uri: "https://example.com/diamond.json" },
  ];

  // Ціни предметів у MagicTokens
  const ITEM_PRICES = [100, 150, 200, 300];

  // PDA адреси
  let gameConfigPda: PublicKey;
  let gameConfigBump: number;
  let mintAuthorityPda: PublicKey;
  let mintAuthorityBump: number;
  let magicConfigPda: PublicKey;
  let magicMintAuthorityPda: PublicKey;
  let itemCollectionPda: PublicKey;
  let nftAuthorityPda: PublicKey;
  let playerAccountPda: PublicKey;
  let searchCpiAuthorityPda: PublicKey;
  let craftingCpiAuthorityPda: PublicKey;
  let marketplaceCpiAuthorityPda: PublicKey;

  // Token accounts гравця для ресурсів (Token-2022 ATAs)
  let playerResourceAtas: PublicKey[] = [];

  type CraftedItem = {
    nftMint: Keypair;
    nftTokenAccount: PublicKey;
    metadataAccount: PublicKey;
    masterEdition: PublicKey;
    itemMetadataPda: PublicKey;
  };

  function buildResourceRemainingAccounts(atas: PublicKey[]) {
    return [
      ...resourceMints.map((mint) => ({
        pubkey: mint.publicKey,
        isSigner: false,
        isWritable: true,
      })),
      ...atas.map((ata) => ({
        pubkey: ata,
        isSigner: false,
        isWritable: true,
      })),
    ];
  }

  function buildRecipeRemainingAccounts(itemType: number, atas: PublicKey[]) {
    const recipe = ITEM_RECIPES[itemType];
    return recipe.flatMap(([resourceId]) => [
      { pubkey: resourceMints[resourceId].publicKey, isSigner: false, isWritable: true },
      { pubkey: atas[resourceId], isSigner: false, isWritable: true },
    ]);
  }

  async function createResourceAtasForOwner(
    owner: Keypair,
    atas: PublicKey[]
  ): Promise<void> {
    const tx = new Transaction();
    for (let i = 0; i < 6; i++) {
      tx.add(
        createAssociatedTokenAccountIdempotentInstruction(
          owner.publicKey,
          atas[i],
          owner.publicKey,
          resourceMints[i].publicKey,
          TOKEN_2022_PROGRAM_ID
        )
      );
    }
    await sendAndConfirmTransaction(provider.connection, tx, [owner]);
  }

  async function getToken2022Amount(account: PublicKey): Promise<number> {
    try {
      const ata = await getAccount(
        provider.connection,
        account,
        undefined,
        TOKEN_2022_PROGRAM_ID
      );
      return Number(ata.amount);
    } catch {
      return 0;
    }
  }

  async function getPlayerResourceBalances(): Promise<number[]> {
    const balances: number[] = [];
    for (const ata of playerResourceAtas) {
      balances.push(await getToken2022Amount(ata));
    }
    return balances;
  }

  async function registerSearchCollector(): Promise<{
    owner: Keypair;
    playerAccount: PublicKey;
    atas: PublicKey[];
  }> {
    const owner = Keypair.generate();
    await fundAccount(
      provider,
      owner.publicKey,
      anchor.web3.LAMPORTS_PER_SOL
    );

    const [playerAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("player"), owner.publicKey.toBuffer()],
      search.programId
    );

    await search.methods
      .registerPlayer()
      .accounts({
        playerOwner: owner.publicKey,
        playerAccount,
        systemProgram: SystemProgram.programId,
      })
      .signers([owner])
      .rpc();

    const atas = resourceMints.map((mint) =>
      getAssociatedTokenAddressSync(
        mint.publicKey,
        owner.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      )
    );

    await createResourceAtasForOwner(owner, atas);

    return { owner, playerAccount, atas };
  }

  async function transferCollectorResourcesToPlayer(
    owner: Keypair,
    collectorAtas: PublicKey[]
  ): Promise<void> {
    const tx = new Transaction();

    for (let i = 0; i < collectorAtas.length; i++) {
      const amount = await getToken2022Amount(collectorAtas[i]);
      if (amount === 0) {
        continue;
      }

      tx.add(
        createTransferCheckedInstruction(
          collectorAtas[i],
          resourceMints[i].publicKey,
          playerResourceAtas[i],
          owner.publicKey,
          amount,
          0,
          [],
          TOKEN_2022_PROGRAM_ID
        )
      );
    }

    if (tx.instructions.length > 0) {
      await sendAndConfirmTransaction(provider.connection, tx, [owner]);
    }
  }

  async function ensureResourcesForRecipe(
    itemType: number,
    maxCollectors = 48
  ): Promise<number[]> {
    const recipe = ITEM_RECIPES[itemType];
    let balances = await getPlayerResourceBalances();

    const hasEnough = () =>
      recipe.every(([resourceId, amount]) => balances[resourceId] >= amount);

    for (let attempt = 0; attempt < maxCollectors && !hasEnough(); attempt++) {
      const collector = await registerSearchCollector();

      await search.methods
        .searchResources()
        .accounts({
          playerOwner: collector.owner.publicKey,
          playerAccount: collector.playerAccount,
          gameConfig: gameConfigPda,
          mintAuthority: mintAuthorityPda,
          cpiAuthority: searchCpiAuthorityPda,
          resourceManagerProgram: resourceManager.programId,
          token2022Program: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .remainingAccounts(buildResourceRemainingAccounts(collector.atas))
        .signers([collector.owner])
        .rpc();

      await transferCollectorResourcesToPlayer(collector.owner, collector.atas);
      balances = await getPlayerResourceBalances();
    }

    expect(
      recipe.every(([resourceId, amount]) => balances[resourceId] >= amount),
      `Insufficient resources for item_type=${itemType}: ${balances.join(",")}`
    ).to.equal(true);

    return balances;
  }

  async function craftShortItem(itemType: number): Promise<CraftedItem> {
    const nftMint = Keypair.generate();
    const itemMeta = SHORT_ITEM_METADATA[itemType];
    const nftTokenAccount = getAssociatedTokenAddressSync(
      nftMint.publicKey,
      player.publicKey
    );
    const [metadataAccount] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        nftMint.publicKey.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );
    const [masterEdition] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        nftMint.publicKey.toBuffer(),
        Buffer.from("edition"),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );
    const [itemMetadataPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("item_metadata"), nftMint.publicKey.toBuffer()],
      itemNft.programId
    );

    await crafting.methods
      .craftItem(itemType, itemMeta.name, itemMeta.symbol, itemMeta.uri)
      .accounts({
        player: player.publicKey,
        gameConfig: gameConfigPda,
        itemCollection: itemCollectionPda,
        nftAuthority: nftAuthorityPda,
        nftMint: nftMint.publicKey,
        nftTokenAccount,
        metadataAccount,
        masterEdition,
        itemMetadata: itemMetadataPda,
        cpiAuthority: craftingCpiAuthorityPda,
        resourceManagerProgram: resourceManager.programId,
        itemNftProgram: itemNft.programId,
        token2022Program: TOKEN_2022_PROGRAM_ID,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        metadataProgram: TOKEN_METADATA_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .remainingAccounts(buildRecipeRemainingAccounts(itemType, playerResourceAtas))
      .signers([player, nftMint])
      .rpc();

    return {
      nftMint,
      nftTokenAccount,
      metadataAccount,
      masterEdition,
      itemMetadataPda,
    };
  }

  before(async () => {
    // Аірдроп SOL для гравця
    await fundAccount(
      provider,
      player.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL
    );

    // Обчислюємо PDA адреси
    [gameConfigPda, gameConfigBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("game_config")],
      resourceManager.programId
    );

    [mintAuthorityPda, mintAuthorityBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("mint_authority")],
      resourceManager.programId
    );

    [magicConfigPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("magic_config")],
      magicToken.programId
    );

    [magicMintAuthorityPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("magic_mint_authority")],
      magicToken.programId
    );

    [itemCollectionPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("item_collection")],
      itemNft.programId
    );

    [nftAuthorityPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("nft_authority")],
      itemNft.programId
    );

    [playerAccountPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("player"), player.publicKey.toBuffer()],
      search.programId
    );

    [searchCpiAuthorityPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("cpi_authority")],
      search.programId
    );

    [craftingCpiAuthorityPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("cpi_authority")],
      crafting.programId
    );

    [marketplaceCpiAuthorityPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("cpi_authority")],
      marketplace.programId
    );

    // Обчислюємо Token-2022 ATA для ресурсів гравця
    playerResourceAtas = resourceMints.map((mint) =>
      getAssociatedTokenAddressSync(
        mint.publicKey,
        player.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      )
    );
  });

  // ==========================================
  //  1. RESOURCE MANAGER TESTS
  // ==========================================
  describe("Resource Manager", () => {
    it("Ініціалізує гру (initialize_game)", async () => {
      await resourceManager.methods
        .initializeGame(
          ITEM_PRICES.map((p) => new BN(p)),
          search.programId,
          crafting.programId,
          marketplace.programId
        )
        .accounts({
          admin: admin.publicKey,
          gameConfig: gameConfigPda,
          mintAuthority: mintAuthorityPda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      // Перевірка стану GameConfig
      const config = await fetchWithRetry(() =>
        resourceManager.account.gameConfig.fetch(gameConfigPda)
      );
      expect(config.admin.toBase58()).to.equal(admin.publicKey.toBase58());
      expect(config.searchProgram.toBase58()).to.equal(
        search.programId.toBase58()
      );
      expect(config.craftingProgram.toBase58()).to.equal(
        crafting.programId.toBase58()
      );
      expect(config.marketplaceProgram.toBase58()).to.equal(
        marketplace.programId.toBase58()
      );
      expect(config.itemPrices.map((p: BN) => p.toNumber())).to.deep.equal(
        ITEM_PRICES
      );
    });

    it("Створює 6 ресурсних мінтів (create_resource_mint)", async () => {
      for (let i = 0; i < 6; i++) {
        await resourceManager.methods
          .createResourceMint(i, RESOURCES[i].name, RESOURCES[i].symbol, RESOURCES[i].uri)
          .accounts({
            admin: admin.publicKey,
            gameConfig: gameConfigPda,
            mintAuthority: mintAuthorityPda,
            resourceMint: resourceMints[i].publicKey,
            systemProgram: SystemProgram.programId,
            token2022Program: TOKEN_2022_PROGRAM_ID,
          })
          .signers([resourceMints[i]])
          .rpc();
      }

      // Перевірка що мінти збережені в GameConfig
      const config = await fetchWithRetry(() =>
        resourceManager.account.gameConfig.fetch(gameConfigPda)
      );
      for (let i = 0; i < 6; i++) {
        expect(config.resourceMints[i].toBase58()).to.equal(
          resourceMints[i].publicKey.toBase58()
        );
      }
    });
    it("mint_resource is CPI-only at the top level", async () => {
      try {
        await resourceManager.methods
          .mintResource(4, new BN(2))
          .accounts({
            callerAuth: searchCpiAuthorityPda,
            gameConfig: gameConfigPda,
            mintAuthority: mintAuthorityPda,
            resourceMint: resourceMints[4].publicKey,
            playerTokenAccount: playerResourceAtas[4],
            token2022Program: TOKEN_2022_PROGRAM_ID,
          })
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err) {
        expectAnchorError(
          err,
          "signature verification failed",
          "missing signature"
        );
      }
    });
    it("mint_resource rejects invalid resource_id before auth checks", async () => {
      const fakeCallerAuth = Keypair.generate();
      try {
        await resourceManager.methods
          .mintResource(6, new BN(1))
          .accounts({
            callerAuth: fakeCallerAuth.publicKey,
            gameConfig: gameConfigPda,
            mintAuthority: mintAuthorityPda,
            resourceMint: resourceMints[0].publicKey,
            playerTokenAccount: playerResourceAtas[0],
            token2022Program: TOKEN_2022_PROGRAM_ID,
          })
          .signers([fakeCallerAuth])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err) {
        expectAnchorError(err, "InvalidResourceId", "invalid resource id");
      }
    });
    it("burn_resource is CPI-only at the top level", async () => {
      try {
        await resourceManager.methods
          .burnResource(4, new BN(1))
          .accounts({
            player: player.publicKey,
            callerAuth: craftingCpiAuthorityPda,
            gameConfig: gameConfigPda,
            resourceMint: resourceMints[4].publicKey,
            playerTokenAccount: playerResourceAtas[4],
            token2022Program: TOKEN_2022_PROGRAM_ID,
          })
          .signers([player])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err) {
        expectAnchorError(
          err,
          "signature verification failed",
          "missing signature"
        );
      }
    });
    it("burn_resource rejects invalid resource_id before auth checks", async () => {
      const fakeCallerAuth = Keypair.generate();
      try {
        await resourceManager.methods
          .burnResource(6, new BN(1))
          .accounts({
            player: player.publicKey,
            callerAuth: fakeCallerAuth.publicKey,
            gameConfig: gameConfigPda,
            resourceMint: resourceMints[0].publicKey,
            playerTokenAccount: playerResourceAtas[0],
            token2022Program: TOKEN_2022_PROGRAM_ID,
          })
          .signers([player, fakeCallerAuth])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err) {
        expectAnchorError(err, "InvalidResourceId", "invalid resource id");
      }
    });

    it("Не дозволяє створити мінт з невірним resource_id", async () => {
      const extraMint = Keypair.generate();
      try {
        await resourceManager.methods
          .createResourceMint(6, "Test", "TEST", "https://test.com")
          .accounts({
            admin: admin.publicKey,
            gameConfig: gameConfigPda,
            mintAuthority: mintAuthorityPda,
            resourceMint: extraMint.publicKey,
            systemProgram: SystemProgram.programId,
            token2022Program: TOKEN_2022_PROGRAM_ID,
          })
          .signers([extraMint])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err: any) {
        expectAnchorError(err, "InvalidResourceId", "invalid resource id");
      }
    });

    it("Не дозволяє створити мінт повторно", async () => {
      const extraMint = Keypair.generate();
      try {
        await resourceManager.methods
          .createResourceMint(0, "Test", "TEST", "https://test.com")
          .accounts({
            admin: admin.publicKey,
            gameConfig: gameConfigPda,
            mintAuthority: mintAuthorityPda,
            resourceMint: extraMint.publicKey,
            systemProgram: SystemProgram.programId,
            token2022Program: TOKEN_2022_PROGRAM_ID,
          })
          .signers([extraMint])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err: any) {
        expectAnchorError(
          err,
          "MintAlreadyInitialized",
          "resource mint already initialized"
        );
      }
    });

    it("Не дозволяє не-адміну створювати мінти", async () => {
      const fakeAdmin = Keypair.generate();
      await fundAccount(
        provider,
        fakeAdmin.publicKey,
        anchor.web3.LAMPORTS_PER_SOL
      );

      const extraMint = Keypair.generate();
      try {
        await resourceManager.methods
          .createResourceMint(0, "Test", "TEST", "https://test.com")
          .accounts({
            admin: fakeAdmin.publicKey,
            gameConfig: gameConfigPda,
            mintAuthority: mintAuthorityPda,
            resourceMint: extraMint.publicKey,
            systemProgram: SystemProgram.programId,
            token2022Program: TOKEN_2022_PROGRAM_ID,
          })
          .signers([fakeAdmin, extraMint])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err: any) {
        // Має бути помилка авторизації
        expect(err).to.exist;
      }
    });

    it("Встановлює magic_token_mint (set_magic_token_mint)", async () => {
      // Спершу ініціалізуємо MagicToken
      // Цей тест буде в секції MagicToken
    });
  });

  // ==========================================
  //  2. MAGIC TOKEN TESTS
  // ==========================================
  describe("Magic Token", () => {
    it("Ініціалізує MagicToken (initialize_magic_token)", async () => {
      await magicToken.methods
        .initializeMagicToken(
          marketplace.programId,
          "MagicToken",
          "MAGIC",
          "https://example.com/magic.json"
        )
        .accounts({
          admin: admin.publicKey,
          config: magicConfigPda,
          magicMint: magicMint.publicKey,
          mintAuthority: magicMintAuthorityPda,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([magicMint])
        .rpc();

      // Перевірка MagicTokenConfig
      const config = await fetchWithRetry(() =>
        magicToken.account.magicTokenConfig.fetch(magicConfigPda)
      );
      expect(config.admin.toBase58()).to.equal(admin.publicKey.toBase58());
      expect(config.mint.toBase58()).to.equal(magicMint.publicKey.toBase58());
      expect(config.marketplaceProgram.toBase58()).to.equal(
        marketplace.programId.toBase58()
      );
    });

    it("Встановлює magic_token_mint в GameConfig", async () => {
      await resourceManager.methods
        .setMagicTokenMint(magicMint.publicKey)
        .accounts({
          admin: admin.publicKey,
          gameConfig: gameConfigPda,
        })
        .rpc();

      const config = await fetchWithRetry(() =>
        resourceManager.account.gameConfig.fetch(gameConfigPda)
      );
      expect(config.magicTokenMint.toBase58()).to.equal(
        magicMint.publicKey.toBase58()
      );
    });

    it("Не дозволяє мінтити MagicTokens напряму (без marketplace CPI)", async () => {
      // Спроба прямого виклику mint_magic_tokens має провалитись
      const fakeCallerAuth = Keypair.generate();
      try {
        const playerMagicAta = getAssociatedTokenAddressSync(
          magicMint.publicKey,
          player.publicKey,
          false,
          TOKEN_2022_PROGRAM_ID
        );
        await magicToken.methods
          .mintMagicTokens(new BN(100))
          .accounts({
            config: magicConfigPda,
            magicMint: magicMint.publicKey,
            recipientTokenAccount: playerMagicAta,
            mintAuthority: magicMintAuthorityPda,
            callerAuth: fakeCallerAuth.publicKey,
            tokenProgram: TOKEN_2022_PROGRAM_ID,
          })
          .signers([fakeCallerAuth])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err: any) {
        expectAnchorError(err, "UnauthorizedCaller", "unauthorized caller");
      }
    });

    it("Magic Token prevents duplicate initialize", async () => {
      const extraMagicMint = Keypair.generate();
      try {
        await magicToken.methods
          .initializeMagicToken(
            marketplace.programId,
            "MagicToken",
            "MAGIC",
            "https://example.com/magic-2.json"
          )
          .accounts({
            admin: admin.publicKey,
            config: magicConfigPda,
            magicMint: extraMagicMint.publicKey,
            mintAuthority: magicMintAuthorityPda,
            tokenProgram: TOKEN_2022_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([extraMagicMint])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err: any) {
        expect(err).to.exist;
      }
    });

  });

  // ==========================================
  //  3. ITEM NFT TESTS
  // ==========================================
  describe("Item NFT", () => {
    it("Ініціалізує колекцію предметів (initialize_collection)", async () => {
      await itemNft.methods
        .initializeCollection(crafting.programId, marketplace.programId)
        .accounts({
          admin: admin.publicKey,
          collection: itemCollectionPda,
          nftAuthority: nftAuthorityPda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      // Перевірка ItemCollection
      const collection = await fetchWithRetry(() =>
        itemNft.account.itemCollection.fetch(itemCollectionPda)
      );
      expect(collection.admin.toBase58()).to.equal(
        admin.publicKey.toBase58()
      );
      expect(collection.craftingProgram.toBase58()).to.equal(
        crafting.programId.toBase58()
      );
      expect(collection.marketplaceProgram.toBase58()).to.equal(
        marketplace.programId.toBase58()
      );
      expect(collection.itemCount.toNumber()).to.equal(0);
    });

    it("Item NFT prevents duplicate collection initialize", async () => {
      try {
        await itemNft.methods
          .initializeCollection(crafting.programId, marketplace.programId)
          .accounts({
            admin: admin.publicKey,
            collection: itemCollectionPda,
            nftAuthority: nftAuthorityPda,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err: any) {
        expect(err).to.exist;
      }
    });

    it("Не дозволяє створювати NFT напряму (без crafting CPI)", async () => {
      const fakeMint = Keypair.generate();
      const fakeCallerAuth = Keypair.generate();
      try {
        await itemNft.methods
          .createItemNft(0, "Test", "TEST", "https://test.com")
          .accounts({
            collection: itemCollectionPda,
            payer: admin.publicKey,
            player: admin.publicKey,
            nftAuthority: nftAuthorityPda,
            nftMint: fakeMint.publicKey,
            playerAta: PublicKey.default,
            metadataAccount: PublicKey.default,
            masterEdition: PublicKey.default,
            itemMetadata: PublicKey.default,
            callerAuth: fakeCallerAuth.publicKey,
            tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            metadataProgram: TOKEN_METADATA_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([fakeMint, fakeCallerAuth])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err: any) {
        expect(err).to.exist;
      }
    });
    it("Item NFT rejects invalid item type before CPI auth", async () => {
      const fakeMint = Keypair.generate();
      const fakeCallerAuth = Keypair.generate();
      const playerAta = getAssociatedTokenAddressSync(
        fakeMint.publicKey,
        admin.publicKey
      );
      const [metadataAccount] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("metadata"),
          TOKEN_METADATA_PROGRAM_ID.toBuffer(),
          fakeMint.publicKey.toBuffer(),
        ],
        TOKEN_METADATA_PROGRAM_ID
      );
      const [masterEdition] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("metadata"),
          TOKEN_METADATA_PROGRAM_ID.toBuffer(),
          fakeMint.publicKey.toBuffer(),
          Buffer.from("edition"),
        ],
        TOKEN_METADATA_PROGRAM_ID
      );
      const [itemMetadataPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("item_metadata"), fakeMint.publicKey.toBuffer()],
        itemNft.programId
      );
      try {
        await itemNft.methods
          .createItemNft(4, "Bad", "BAD", "u")
          .accounts({
            collection: itemCollectionPda,
            payer: admin.publicKey,
            player: admin.publicKey,
            nftAuthority: nftAuthorityPda,
            nftMint: fakeMint.publicKey,
            playerAta,
            metadataAccount,
            masterEdition,
            itemMetadata: itemMetadataPda,
            callerAuth: fakeCallerAuth.publicKey,
            tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            metadataProgram: TOKEN_METADATA_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([fakeMint, fakeCallerAuth])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err) {
        expectAnchorError(err, "InvalidItemType", "invalid item type");
      }
    });

  });

  // ==========================================
  //  4. SEARCH TESTS
  // ==========================================
  describe("Search", () => {
    it("Реєструє гравця (register_player)", async () => {
      await search.methods
        .registerPlayer()
        .accounts({
          playerOwner: player.publicKey,
          playerAccount: playerAccountPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([player])
        .rpc();

      // Перевірка PlayerAccount
      const account = await fetchWithRetry(() =>
        search.account.playerAccount.fetch(playerAccountPda)
      );
      expect(account.owner.toBase58()).to.equal(player.publicKey.toBase58());
      expect(account.lastSearchTimestamp.toNumber()).to.equal(0);
    });

    it("Не дозволяє повторну реєстрацію", async () => {
      try {
        await search.methods
          .registerPlayer()
          .accounts({
            playerOwner: player.publicKey,
            playerAccount: playerAccountPda,
            systemProgram: SystemProgram.programId,
          })
          .signers([player])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err: any) {
        // PDA вже існує — має бути помилка
        expect(err).to.exist;
      }
    });

    it("Search enforces a full remaining account set", async () => {
      try {
        await search.methods
          .searchResources()
          .accounts({
            playerOwner: player.publicKey,
            playerAccount: playerAccountPda,
            gameConfig: gameConfigPda,
            mintAuthority: mintAuthorityPda,
            cpiAuthority: searchCpiAuthorityPda,
            resourceManagerProgram: resourceManager.programId,
            token2022Program: TOKEN_2022_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .remainingAccounts(buildResourceRemainingAccounts(playerResourceAtas).slice(0, 10))
          .signers([player])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err: any) {
        expectAnchorError(
          err,
          "InsufficientAccounts",
          "insufficient remaining accounts"
        );
      }
    });

    it("Виконує пошук ресурсів (search_resources)", async () => {
      // Спершу створимо Token-2022 ATA для гравця для кожного ресурсу
      for (let i = 0; i < 6; i++) {
        const ataIx = createAssociatedTokenAccountIdempotentInstruction(
          player.publicKey,
          playerResourceAtas[i],
          player.publicKey,
          resourceMints[i].publicKey,
          TOKEN_2022_PROGRAM_ID
        );
        const tx = new Transaction().add(ataIx);
        await sendAndConfirmTransaction(provider.connection, tx, [player]);
      }

      // Збираємо remaining accounts: 6 мінтів + 6 ATA
      const remainingAccounts = [
        ...resourceMints.map((m) => ({
          pubkey: m.publicKey,
          isSigner: false,
          isWritable: true,
        })),
        ...playerResourceAtas.map((ata) => ({
          pubkey: ata,
          isSigner: false,
          isWritable: true,
        })),
      ];

      await search.methods
        .searchResources()
        .accounts({
          playerOwner: player.publicKey,
          playerAccount: playerAccountPda,
          gameConfig: gameConfigPda,
          mintAuthority: mintAuthorityPda,
          cpiAuthority: searchCpiAuthorityPda,
          resourceManagerProgram: resourceManager.programId,
          token2022Program: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .remainingAccounts(remainingAccounts)
        .signers([player])
        .rpc();

      // Перевіряємо що гравець отримав ресурси
      let totalResources = 0;
      for (let i = 0; i < 6; i++) {
        try {
          const ata = await getAccount(
            provider.connection,
            playerResourceAtas[i],
            undefined,
            TOKEN_2022_PROGRAM_ID
          );
          totalResources += Number(ata.amount);
        } catch {
          // ATA може бути порожнім
        }
      }
      expect(totalResources).to.equal(3); // Має бути 3 ресурси
    });

    it("Блокує пошук під час кулдауну (60 сек)", async () => {
      const remainingAccounts = [
        ...resourceMints.map((m) => ({
          pubkey: m.publicKey,
          isSigner: false,
          isWritable: true,
        })),
        ...playerResourceAtas.map((ata) => ({
          pubkey: ata,
          isSigner: false,
          isWritable: true,
        })),
      ];

      try {
        await search.methods
          .searchResources()
          .accounts({
            playerOwner: player.publicKey,
            playerAccount: playerAccountPda,
            gameConfig: gameConfigPda,
            mintAuthority: mintAuthorityPda,
            cpiAuthority: searchCpiAuthorityPda,
            resourceManagerProgram: resourceManager.programId,
            token2022Program: TOKEN_2022_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .remainingAccounts(remainingAccounts)
          .signers([player])
          .rpc();
        expect.fail("Should have thrown error - cooldown");
      } catch (err: any) {
        expectAnchorError(err, "SearchCooldown", "search cooldown");
      }
    });

    it("Перевіряє оновлення таймстампу після пошуку", async () => {
      const account = await fetchWithRetry(() =>
        search.account.playerAccount.fetch(playerAccountPda)
      );
      expect(account.lastSearchTimestamp.toNumber()).to.be.greaterThan(0);
    });

    it("Search validates resource mint ordering", async () => {
      const collector = await registerSearchCollector();
      const remainingAccounts = [
        ...resourceMints.map((_, index) => ({
          pubkey: resourceMints[(index + 1) % resourceMints.length].publicKey,
          isSigner: false,
          isWritable: true,
        })),
        ...collector.atas.map((ata) => ({
          pubkey: ata,
          isSigner: false,
          isWritable: true,
        })),
      ];

      try {
        await search.methods
          .searchResources()
          .accounts({
            playerOwner: collector.owner.publicKey,
            playerAccount: collector.playerAccount,
            gameConfig: gameConfigPda,
            mintAuthority: mintAuthorityPda,
            cpiAuthority: searchCpiAuthorityPda,
            resourceManagerProgram: resourceManager.programId,
            token2022Program: TOKEN_2022_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .remainingAccounts(remainingAccounts)
          .signers([collector.owner])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err: any) {
        expectAnchorError(err, "InvalidResourceId", "invalid resource id");
      }
    });
    it("Search validates player PDA ownership", async () => {
      const stranger = Keypair.generate();
      await fundAccount(
        provider,
        stranger.publicKey,
        anchor.web3.LAMPORTS_PER_SOL
      );

      try {
        await search.methods
          .searchResources()
          .accounts({
            playerOwner: stranger.publicKey,
            playerAccount: playerAccountPda,
            gameConfig: gameConfigPda,
            mintAuthority: mintAuthorityPda,
            cpiAuthority: searchCpiAuthorityPda,
            resourceManagerProgram: resourceManager.programId,
            token2022Program: TOKEN_2022_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .remainingAccounts(buildResourceRemainingAccounts(playerResourceAtas))
          .signers([stranger])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err) {
        expectAnchorError(
          err,
          "ConstraintSeeds",
          "seeds constraint was violated",
          "ConstraintAddress"
        );
      }
    });

  });

  // ==========================================
  //  5. CRAFTING TESTS
  // ==========================================
  describe("Crafting", () => {
    // Для крафту потрібні ресурси — дочекаємось кулдауну та зробимо кілька пошуків
    before(async () => {
      // Чекаємо 60 секунд кулдауну та робимо кілька пошуків для накопичення ресурсів
      // В реальних тестах можна маніпулювати часом через warp
      // Для локальних тестів використовуємо достатню кількість ресурсів

      // Мінтимо ресурси напряму через resource_manager для тестування крафту
      // Використовуємо search CPI authority
      // Але це потребує CPI через search програму...

      // Альтернатива: зробимо кілька search_resources з warp-ом часу
    });

    it("Крафтить Шаблю козака (item_type=0: 3 Iron + 1 Wood + 1 Leather)", async () => {
      // Накопичуємо ресурси через колекторів (обхід кулдауну 60 сек)
      await ensureResourcesForRecipe(0);

      // Крафтимо з короткими метаданими щоб TX вмістився в 1232 байти
      const crafted = await craftShortItem(0);

      // Перевіряємо що NFT створено
      const nftAccount = await getAccount(
        provider.connection,
        crafted.nftTokenAccount
      );
      expect(Number(nftAccount.amount)).to.equal(1);

      // Перевіряємо ItemMetadata
      const itemMeta = await fetchWithRetry(() =>
        itemNft.account.itemMetadata.fetch(crafted.itemMetadataPda)
      );
      expect(itemMeta.itemType).to.equal(0);
      expect(itemMeta.owner.toBase58()).to.equal(
        player.publicKey.toBase58()
      );
      expect(itemMeta.mint.toBase58()).to.equal(
        crafted.nftMint.publicKey.toBase58()
      );
    });
    it("Crafting validates writable NFT accounts before CPI", async () => {
      const nftMint = Keypair.generate();
      try {
        await crafting.methods
          .craftItem(0, "S", "S", "https://e/s")
          .accounts({
            player: player.publicKey,
            gameConfig: gameConfigPda,
            itemCollection: itemCollectionPda,
            nftAuthority: nftAuthorityPda,
            nftMint: nftMint.publicKey,
            nftTokenAccount: PublicKey.default,
            metadataAccount: PublicKey.default,
            masterEdition: PublicKey.default,
            itemMetadata: PublicKey.default,
            cpiAuthority: craftingCpiAuthorityPda,
            resourceManagerProgram: resourceManager.programId,
            itemNftProgram: itemNft.programId,
            token2022Program: TOKEN_2022_PROGRAM_ID,
            tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            metadataProgram: TOKEN_METADATA_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .remainingAccounts(buildRecipeRemainingAccounts(0, playerResourceAtas).slice(0, 5))
          .signers([player, nftMint])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err) {
        expectAnchorError(
          err,
          "InsufficientAccounts",
          "insufficient remaining accounts",
          "ConstraintMut",
          "mut constraint was violated"
        );
      }
    });
    it("Crafting requires nft mint signer", async () => {
      const nftMint = Keypair.generate();
      try {
        await crafting.methods
          .craftItem(0, "S", "S", "u")
          .accounts({
            player: player.publicKey,
            gameConfig: gameConfigPda,
            itemCollection: itemCollectionPda,
            nftAuthority: nftAuthorityPda,
            nftMint: nftMint.publicKey,
            nftTokenAccount: PublicKey.default,
            metadataAccount: PublicKey.default,
            masterEdition: PublicKey.default,
            itemMetadata: PublicKey.default,
            cpiAuthority: craftingCpiAuthorityPda,
            resourceManagerProgram: resourceManager.programId,
            itemNftProgram: itemNft.programId,
            token2022Program: TOKEN_2022_PROGRAM_ID,
            tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            metadataProgram: TOKEN_METADATA_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .remainingAccounts(buildRecipeRemainingAccounts(0, playerResourceAtas).slice(0, 5))
          .signers([player])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err) {
        expectAnchorError(
          err,
          "signature verification failed",
          "unknown signer",
          "missing signature"
        );
      }
    });

    it("Не дозволяє крафт з невірним item_type", async () => {
      const nftMint = Keypair.generate();

      try {
        await crafting.methods
          .craftItem(4, "X", "X", "u")
          .accounts({
            player: player.publicKey,
            gameConfig: gameConfigPda,
            itemCollection: itemCollectionPda,
            nftAuthority: nftAuthorityPda,
            nftMint: nftMint.publicKey,
            nftTokenAccount: PublicKey.default,
            metadataAccount: PublicKey.default,
            masterEdition: PublicKey.default,
            itemMetadata: PublicKey.default,
            cpiAuthority: craftingCpiAuthorityPda,
            resourceManagerProgram: resourceManager.programId,
            itemNftProgram: itemNft.programId,
            token2022Program: TOKEN_2022_PROGRAM_ID,
            tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            metadataProgram: TOKEN_METADATA_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .remainingAccounts(buildRecipeRemainingAccounts(0, playerResourceAtas))
          .signers([player, nftMint])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err: any) {
        expectAnchorError(
          err,
          "InvalidItemType",
          "invalid item type",
          "ConstraintMut",
          "mut constraint was violated"
        );
      }
    });
  });

  // ==========================================
  //  6. MARKETPLACE TESTS
  // ==========================================
  describe("Marketplace", () => {
    let testNftMint: Keypair;
    let testNftTokenAccount: PublicKey;
    let testItemMetadataPda: PublicKey;
    let playerMagicAta: PublicKey;

    before(async () => {
      // Створюємо Token-2022 ATA для MagicToken гравця
      playerMagicAta = getAssociatedTokenAddressSync(
        magicMint.publicKey,
        player.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      try {
        const ataIx = createAssociatedTokenAccountIdempotentInstruction(
          player.publicKey,
          playerMagicAta,
          player.publicKey,
          magicMint.publicKey,
          TOKEN_2022_PROGRAM_ID
        );
        const tx = new Transaction().add(ataIx);
        await sendAndConfirmTransaction(provider.connection, tx, [player]);
      } catch {
        // ATA може вже існувати
      }
    });

    it("Продає предмет на маркетплейсі (sell_item)", async () => {
      // Збираємо ресурси та крафтимо предмет для продажу
      await ensureResourcesForRecipe(1);
      const crafted = await craftShortItem(1);

      // Перевіряємо NFT на руках
      const nftBefore = await getAccount(
        provider.connection,
        crafted.nftTokenAccount
      );
      expect(Number(nftBefore.amount)).to.equal(1);

      const magicBefore = await getToken2022Amount(playerMagicAta);

      // Продаємо предмет
      await marketplace.methods
        .sellItem(1)
        .accounts({
          seller: player.publicKey,
          gameConfig: gameConfigPda,
          itemCollection: itemCollectionPda,
          nftMint: crafted.nftMint.publicKey,
          nftTokenAccount: crafted.nftTokenAccount,
          itemMetadata: crafted.itemMetadataPda,
          magicConfig: magicConfigPda,
          magicMintAuthority: magicMintAuthorityPda,
          magicMint: magicMint.publicKey,
          sellerMagicAta: playerMagicAta,
          cpiAuthority: marketplaceCpiAuthorityPda,
          itemNftProgram: itemNft.programId,
          magicTokenProgram: magicToken.programId,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          token2022Program: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([player])
        .rpc();

      // Перевіряємо що MagicTokens отримано (ціна item_type=1 = 150)
      const magicAfter = await getToken2022Amount(playerMagicAta);
      expect(magicAfter - magicBefore).to.equal(ITEM_PRICES[1]);
    });

    it("Не дозволяє продати з невірним item_type", async () => {
      const fakeMint = Keypair.generate();
      try {
        await marketplace.methods
          .sellItem(5) // Невірний тип
          .accounts({
            seller: player.publicKey,
            gameConfig: gameConfigPda,
            itemCollection: itemCollectionPda,
            nftMint: fakeMint.publicKey,
            nftTokenAccount: PublicKey.default,
            itemMetadata: PublicKey.default,
            magicConfig: magicConfigPda,
            magicMintAuthority: magicMintAuthorityPda,
            magicMint: magicMint.publicKey,
            sellerMagicAta: playerMagicAta,
            cpiAuthority: marketplaceCpiAuthorityPda,
            itemNftProgram: itemNft.programId,
            magicTokenProgram: magicToken.programId,
            tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
            token2022Program: TOKEN_2022_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([player])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err: any) {
        expectAnchorError(
          err,
          "InvalidItemType",
          "invalid item type",
          "AccountNotInitialized",
          "account not initialized"
        );
      }
    });
    it("Marketplace rejects Token-2022 mint as an NFT mint", async () => {
      try {
        await marketplace.methods
          .sellItem(0)
          .accounts({
            seller: player.publicKey,
            gameConfig: gameConfigPda,
            itemCollection: itemCollectionPda,
            nftMint: magicMint.publicKey,
            nftTokenAccount: playerMagicAta,
            itemMetadata: PublicKey.default,
            magicConfig: magicConfigPda,
            magicMintAuthority: magicMintAuthorityPda,
            magicMint: magicMint.publicKey,
            sellerMagicAta: playerMagicAta,
            cpiAuthority: marketplaceCpiAuthorityPda,
            itemNftProgram: itemNft.programId,
            magicTokenProgram: magicToken.programId,
            tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
            token2022Program: TOKEN_2022_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([player])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err) {
        expectAnchorError(
          err,
          "AccountOwnedByWrongProgram",
          "owned by another program",
          "invalid account owner"
        );
      }
    });
    it("Marketplace requires initialized sale accounts", async () => {
      const fakeMint = Keypair.generate();
      try {
        await marketplace.methods
          .sellItem(0)
          .accounts({
            seller: player.publicKey,
            gameConfig: gameConfigPda,
            itemCollection: itemCollectionPda,
            nftMint: fakeMint.publicKey,
            nftTokenAccount: PublicKey.default,
            itemMetadata: PublicKey.default,
            magicConfig: magicConfigPda,
            magicMintAuthority: magicMintAuthorityPda,
            magicMint: magicMint.publicKey,
            sellerMagicAta: playerMagicAta,
            cpiAuthority: marketplaceCpiAuthorityPda,
            itemNftProgram: itemNft.programId,
            magicTokenProgram: magicToken.programId,
            tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
            token2022Program: TOKEN_2022_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([player])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err) {
        expectAnchorError(err, "AccountNotInitialized", "account not initialized");
      }
    });

    it("Перевіряє що MagicTokens мінтяться тільки через marketplace", async () => {
      // Вже перевірено в секції Magic Token
      // Додатковий тест: перевірка що пряме CPI без marketplace PDA не працює
      const fakeAuth = Keypair.generate();
      try {
        await magicToken.methods
          .mintMagicTokens(new BN(1000))
          .accounts({
            config: magicConfigPda,
            magicMint: magicMint.publicKey,
            recipientTokenAccount: playerMagicAta,
            mintAuthority: magicMintAuthorityPda,
            callerAuth: fakeAuth.publicKey,
            tokenProgram: TOKEN_2022_PROGRAM_ID,
          })
          .signers([fakeAuth])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err: any) {
        expectAnchorError(err, "UnauthorizedCaller", "unauthorized caller");
      }
    });
  });

  // ==========================================
  //  7. PDA AUTHORITY & SECURITY TESTS
  // ==========================================
  describe("Безпека (PDA Authority)", () => {
    it("Перевіряє що mint_resource вимагає правильний caller_auth", async () => {
      const fakeAuth = Keypair.generate();
      try {
        await resourceManager.methods
          .mintResource(0, new BN(100))
          .accounts({
            callerAuth: fakeAuth.publicKey,
            gameConfig: gameConfigPda,
            mintAuthority: mintAuthorityPda,
            resourceMint: resourceMints[0].publicKey,
            playerTokenAccount: playerResourceAtas[0],
            token2022Program: TOKEN_2022_PROGRAM_ID,
          })
          .signers([fakeAuth])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err: any) {
        expectAnchorError(err, "UnauthorizedCaller", "unauthorized caller");
      }
    });

    it("Перевіряє що burn_resource вимагає правильний caller_auth", async () => {
      const fakeAuth = Keypair.generate();
      try {
        await resourceManager.methods
          .burnResource(0, new BN(1))
          .accounts({
            player: player.publicKey,
            callerAuth: fakeAuth.publicKey,
            gameConfig: gameConfigPda,
            resourceMint: resourceMints[0].publicKey,
            playerTokenAccount: playerResourceAtas[0],
            token2022Program: TOKEN_2022_PROGRAM_ID,
          })
          .signers([player, fakeAuth])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err: any) {
        expectAnchorError(err, "UnauthorizedCaller", "unauthorized caller");
      }
    });

    it("Перевіряє що тільки адмін може set_magic_token_mint", async () => {
      const fakeAdmin = Keypair.generate();
      await fundAccount(
        provider,
        fakeAdmin.publicKey,
        anchor.web3.LAMPORTS_PER_SOL
      );

      try {
        await resourceManager.methods
          .setMagicTokenMint(Keypair.generate().publicKey)
          .accounts({
            admin: fakeAdmin.publicKey,
            gameConfig: gameConfigPda,
          })
          .signers([fakeAdmin])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (err: any) {
        expect(err).to.exist;
      }
    });
  });

  // ==========================================
  //  8. INTEGRATION / END-TO-END TESTS
  // ==========================================
  describe("Інтеграційний тест (End-to-End)", () => {
    it("Повний цикл: search → craft → sell", async () => {
      // 1. Збираємо ресурси для Броні характерника (type 2)
      //    Рецепт: 4× Шкіра + 2× Залізо + 1× Золото
      await ensureResourcesForRecipe(2);

      const balancesBefore = await getPlayerResourceBalances();

      // 2. Крафтимо предмет — ресурси спалюються, NFT створюється
      const crafted = await craftShortItem(2);

      const balancesAfter = await getPlayerResourceBalances();
      // Перевіряємо що ресурси спалились за рецептом
      const recipe = ITEM_RECIPES[2]; // [[3,4],[1,2],[2,1]]
      for (const [resourceId, amount] of recipe) {
        expect(balancesBefore[resourceId] - balancesAfter[resourceId]).to.equal(
          amount,
          `Resource ${resourceId} should decrease by ${amount}`
        );
      }

      // Перевіряємо NFT на руках
      const nftAccount = await getAccount(
        provider.connection,
        crafted.nftTokenAccount
      );
      expect(Number(nftAccount.amount)).to.equal(1);

      // 3. Продаємо на маркетплейсі — NFT спалюється, MagicToken мінтиться
      const playerMagicAta = getAssociatedTokenAddressSync(
        magicMint.publicKey,
        player.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );
      const magicBefore = await getToken2022Amount(playerMagicAta);

      await marketplace.methods
        .sellItem(2)
        .accounts({
          seller: player.publicKey,
          gameConfig: gameConfigPda,
          itemCollection: itemCollectionPda,
          nftMint: crafted.nftMint.publicKey,
          nftTokenAccount: crafted.nftTokenAccount,
          itemMetadata: crafted.itemMetadataPda,
          magicConfig: magicConfigPda,
          magicMintAuthority: magicMintAuthorityPda,
          magicMint: magicMint.publicKey,
          sellerMagicAta: playerMagicAta,
          cpiAuthority: marketplaceCpiAuthorityPda,
          itemNftProgram: itemNft.programId,
          magicTokenProgram: magicToken.programId,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          token2022Program: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([player])
        .rpc();

      // Перевіряємо MagicToken (ціна type=2 = 200)
      const magicAfter = await getToken2022Amount(playerMagicAta);
      expect(magicAfter - magicBefore).to.equal(ITEM_PRICES[2]);
    });
  });
});
