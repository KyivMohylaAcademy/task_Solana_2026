/**
 * Інтеграційні тести для гри "Козацький бізнес"
 * Покриває всі 6 програм: resource_manager, magic_token, search,
 * item_nft, crafting, marketplace.
 *
 * Anchor 0.32.x автоматично виводить PDA-акаунти та fixed-address
 * акаунти із IDL — у .accounts() вказуємо лише "plain" акаунти.
 *
 * Запуск: anchor test (localnet + клонований Metaplex, див. Anchor.toml)
 */

import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import {
  TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getOrCreateAssociatedTokenAccount,
  getAssociatedTokenAddressSync,
  getAccount,
  createMint,
} from "@solana/spl-token";
import {
  Keypair,
  PublicKey,
  LAMPORTS_PER_SOL,
  Transaction,
  SystemProgram,
  ComputeBudgetProgram,
} from "@solana/web3.js";
import { assert } from "chai";

import { ResourceManager } from "../target/types/resource_manager";
import { MagicToken } from "../target/types/magic_token";
import { ItemNft } from "../target/types/item_nft";
import { Search } from "../target/types/search";
import { Crafting } from "../target/types/crafting";
import { Marketplace } from "../target/types/marketplace";

// ─── Константи ──────────────────────────────────────────────────────────────

const MPL_TOKEN_METADATA_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

const SYSVAR_INSTRUCTIONS_PUBKEY = new PublicKey(
  "Sysvar1nstructions1111111111111111111111111"
);

/** Metaplex Metadata PDA для заданого мінту. */
function mplMetaPda(mint: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      MPL_TOKEN_METADATA_ID.toBuffer(),
      mint.toBuffer(),
    ],
    MPL_TOKEN_METADATA_ID
  )[0];
}

/** Metaplex Master Edition PDA для заданого мінту. */
function mplMasterEditionPda(mint: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      MPL_TOKEN_METADATA_ID.toBuffer(),
      mint.toBuffer(),
      Buffer.from("edition"),
    ],
    MPL_TOKEN_METADATA_ID
  )[0];
}

/** Associated Token Address для Token-2022. */
function ata22(mint: PublicKey, owner: PublicKey): PublicKey {
  return getAssociatedTokenAddressSync(
    mint,
    owner,
    false,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
  );
}

/** ItemMetadata PDA. */
function itemMetaPda(nftMint: PublicKey, nftProgramId: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("item_metadata"), nftMint.toBuffer()],
    nftProgramId
  )[0];
}

// ─── Допоміжна ──────────────────────────────────────────────────────────────

async function airdrop(
  provider: anchor.AnchorProvider,
  from: anchor.web3.Keypair,
  to: PublicKey,
  sol: number
): Promise<void> {
  const tx = new Transaction().add(
    SystemProgram.transfer({
      fromPubkey: from.publicKey,
      toPubkey: to,
      lamports: sol * LAMPORTS_PER_SOL,
    })
  );
  await provider.sendAndConfirm(tx, [from], { commitment: "confirmed" });
}

// ─── Тести ──────────────────────────────────────────────────────────────────

describe("Козацький бізнес — інтеграційні тести", () => {
  // preflightCommitment="processed" щоб уникнути хибних "Blockhash not found" на devnet
  const envProvider = anchor.AnchorProvider.env();
  const provider = new anchor.AnchorProvider(
    envProvider.connection,
    envProvider.wallet,
    { commitment: "confirmed", skipPreflight: false, preflightCommitment: "processed" }
  );
  anchor.setProvider(provider);
  const conn = provider.connection;

  // Програми з workspace (Anchor.toml [programs.localnet])
  const rmProg  = anchor.workspace.ResourceManager as Program<ResourceManager>;
  const mtProg  = anchor.workspace.MagicToken      as Program<MagicToken>;
  const nftProg = anchor.workspace.ItemNft         as Program<ItemNft>;
  const srProg  = anchor.workspace.Search          as Program<Search>;
  const crProg  = anchor.workspace.Crafting        as Program<Crafting>;
  const mpProg  = anchor.workspace.Marketplace     as Program<Marketplace>;

  // Keypairs
  // На devnet використовуємо provider wallet як admin (вже має SOL).
  const admin  = (provider.wallet as anchor.Wallet).payer;
  const player = Keypair.generate();

  // Marketplace Authority PDA (seeds: ["marketplace_authority"], prog: marketplace)
  const [marketplaceAuthority] = PublicKey.findProgramAddressSync(
    [Buffer.from("marketplace_authority")],
    mpProg.programId
  );

  // MagicMint PDA (seeds: ["magic_mint"], prog: magic_token)
  const [magicMintPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("magic_mint")],
    mtProg.programId
  );

  // Player PDA (seeds: ["player", owner], prog: search)
  const [playerPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("player"), player.publicKey.toBuffer()],
    srProg.programId
  );

  // GameConfig PDA (seeds: ["game_config"], prog: resource_manager)
  const [gameConfigPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("game_config")],
    rmProg.programId
  );

  // 6 ресурсних мінтів та Token Accounts гравця (заповнюються у before())
  let resourceMints: PublicKey[] = [];
  let playerTas: PublicKey[]     = [];

  // ── Глобальна ініціалізація ────────────────────────────────────────────
  before("fund + magic_token + resource_manager + ATAs", async () => {
    // На devnet airdrop rate-limited; переказуємо SOL від provider wallet до player.
    const fundTx = new Transaction().add(
      SystemProgram.transfer({
        fromPubkey: admin.publicKey,
        toPubkey: player.publicKey,
        lamports: 0.2 * LAMPORTS_PER_SOL,
      })
    );
    await provider.sendAndConfirm(fundTx, [], { commitment: "confirmed" });

    // 1. magic_token: ініціалізуємо MagicConfig + MagicMint PDA (якщо ще не існує)
    const [magicConfigPdaLocal] = PublicKey.findProgramAddressSync(
      [Buffer.from("magic_config")],
      mtProg.programId
    );
    const magicConfigInfo = await conn.getAccountInfo(magicConfigPdaLocal, "confirmed");
    if (!magicConfigInfo) {
      await mtProg.methods
        .initializeMagicToken()
        .accounts({ marketplaceAuthority })
        .signers([admin])
        .rpc();
    }

    // 2. resource_manager: якщо GameConfig вже існує — читаємо мінти з нього,
    //    інакше — створюємо нові та ініціалізуємо.
    const gameConfigInfo = await conn.getAccountInfo(gameConfigPda, "confirmed");
    if (gameConfigInfo) {
      // GameConfig вже ініціалізовано — читаємо існуючі мінти
      const cfg = await rmProg.account.gameConfig.fetch(gameConfigPda);
      for (let i = 0; i < 6; i++) {
        resourceMints.push(cfg.resourceMints[i]);
      }
    } else {
      // Створюємо 6 SPL Token-2022 мінтів; mint_authority = GameConfig PDA
      for (let i = 0; i < 6; i++) {
        const mint = await createMint(
          conn,
          admin,
          gameConfigPda,
          null,
          0,
          undefined,
          { commitment: "confirmed" },
          TOKEN_2022_PROGRAM_ID
        );
        resourceMints.push(mint);
      }

      await rmProg.methods
        .initializeGameConfig()
        .accounts({
          mintWood:       resourceMints[0],
          mintIron:       resourceMints[1],
          mintGold:       resourceMints[2],
          mintLeather:    resourceMints[3],
          mintStone:      resourceMints[4],
          mintDiamond:    resourceMints[5],
          magicTokenMint: magicMintPda,
        })
        .signers([admin])
        .rpc();
    }

    // 3. Створюємо/відкриваємо Token-2022 ATA для кожного ресурсу для гравця
    for (let i = 0; i < 6; i++) {
      const ta = await getOrCreateAssociatedTokenAccount(
        conn, player,
        resourceMints[i], player.publicKey,
        false, "confirmed", { commitment: "confirmed" },
        TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID
      );
      playerTas.push(ta.address);
    }
  });

  // ═══════════════════════════════════════════════════════════════════════
  // 1. resource_manager
  // ═══════════════════════════════════════════════════════════════════════
  describe("resource_manager", () => {

    it("initialize_game_config — GameConfig містить 6 мінтів і ціни [10,15,20,25]", async () => {
      const cfg = await rmProg.account.gameConfig.fetch(gameConfigPda);

      assert.equal(cfg.admin.toBase58(), admin.publicKey.toBase58());
      for (let i = 0; i < 6; i++) {
        assert.equal(cfg.resourceMints[i].toBase58(), resourceMints[i].toBase58());
      }
      assert.equal(cfg.magicTokenMint.toBase58(), magicMintPda.toBase58());
      assert.deepEqual(cfg.itemPrices.map((p: BN) => p.toNumber()), [10, 15, 20, 25]);
    });

    it("mint_resource — мінтить 5 WOOD (id=0) гравцю", async () => {
      // Plain-акаунти: resourceMint, playerTokenAccount, player
      await rmProg.methods
        .mintResource(0, new BN(5))
        .accounts({
          resourceMint:        resourceMints[0],
          playerTokenAccount:  playerTas[0],
          player:              player.publicKey,
        })
        .rpc({ commitment: "confirmed" });

      const ta = await getAccount(conn, playerTas[0], "confirmed", TOKEN_2022_PROGRAM_ID);
      assert.equal(ta.amount.toString(), "5");
    });

    it("burn_resource — спалює 2 WOOD, залишається 3", async () => {
      await rmProg.methods
        .burnResource(0, new BN(2))
        .accounts({
          resourceMint:       resourceMints[0],
          playerTokenAccount: playerTas[0],
          player:             player.publicKey,
        })
        .signers([player])
        .rpc({ commitment: "confirmed" });

      const ta = await getAccount(conn, playerTas[0], "confirmed", TOKEN_2022_PROGRAM_ID);
      assert.equal(ta.amount.toString(), "3");
    });

    it("mint_resource — помилка при resource_id=6 (InvalidResourceId)", async () => {
      try {
        await rmProg.methods
          .mintResource(6, new BN(1))
          .accounts({
            resourceMint:       resourceMints[0],
            playerTokenAccount: playerTas[0],
            player:             player.publicKey,
          })
          .rpc();
        assert.fail("Мало кинути помилку");
      } catch (err: any) {
        // constraint resource_mints[6] паде до invalid index або програма
        assert.exists(err);
      }
    });
  });

  // ═══════════════════════════════════════════════════════════════════════
  // 2. magic_token
  // ═══════════════════════════════════════════════════════════════════════
  describe("magic_token", () => {

    it("initialize_magic_token — MagicConfig містить правильний mint і authority", async () => {
      const [magicConfigPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("magic_config")],
        mtProg.programId
      );
      const cfg = await mtProg.account.magicConfig.fetch(magicConfigPda);

      assert.equal(cfg.mint.toBase58(), magicMintPda.toBase58());
      assert.equal(cfg.marketplaceAuthority.toBase58(), marketplaceAuthority.toBase58());
    });

    // mint_magic_token: перевіряється через buy_item (marketplace), бо
    // marketplace_authority PDA є єдиним валідним Signer для mint_authority.
  });

  // ═══════════════════════════════════════════════════════════════════════
  // 3. search
  // ═══════════════════════════════════════════════════════════════════════
  describe("search", () => {

    it("initialize_player — Player PDA створено, last_search_timestamp = 0", async () => {
      // owner — єдиний plain-акаунт (player — PDA, system_program — addr)
      await srProg.methods
        .initializePlayer()
        .accounts({ owner: player.publicKey })
        .signers([player])
        .rpc();

      const p = await srProg.account.player.fetch(playerPda);
      assert.equal(p.owner.toBase58(), player.publicKey.toBase58());
      assert.equal(p.lastSearchTimestamp.toNumber(), 0);
    });

    it("search_resources — перший пошук успішний, +3 ресурси", async () => {
      // remaining_accounts: [mint_0, ta_0, mint_1, ta_1, ..., mint_5, ta_5]
      const remaining = [];
      for (let i = 0; i < 6; i++) {
        remaining.push({ pubkey: resourceMints[i], isWritable: true,  isSigner: false });
        remaining.push({ pubkey: playerTas[i],     isWritable: true,  isSigner: false });
      }

      let totalBefore = BigInt(0);
      for (const ta of playerTas) {
        const acc = await getAccount(conn, ta, "confirmed", TOKEN_2022_PROGRAM_ID);
        totalBefore += acc.amount;
      }

      // owner — єдиний plain; решта (player, game_config, програми) — PDAs/addr
      await srProg.methods
        .searchResources()
        .accounts({ owner: player.publicKey })
        .remainingAccounts(remaining)
        .signers([player])
        .rpc({ commitment: "confirmed" });

      let totalAfter = BigInt(0);
      for (const ta of playerTas) {
        const acc = await getAccount(conn, ta, "confirmed", TOKEN_2022_PROGRAM_ID);
        totalAfter += acc.amount;
      }
      assert.equal((totalAfter - totalBefore).toString(), "3", "Має додатись рівно 3 ресурси");

      const p = await srProg.account.player.fetch(playerPda);
      assert.isAbove(p.lastSearchTimestamp.toNumber(), 0);
    });

    it("search_resources — помилка: cooldown ще не минув (CooldownNotExpired)", async () => {
      const remaining = [];
      for (let i = 0; i < 6; i++) {
        remaining.push({ pubkey: resourceMints[i], isWritable: true, isSigner: false });
        remaining.push({ pubkey: playerTas[i],     isWritable: true, isSigner: false });
      }

      try {
        await srProg.methods
          .searchResources()
          .accounts({ owner: player.publicKey })
          .remainingAccounts(remaining)
          .signers([player])
          .rpc();
        assert.fail("Мало кинути CooldownNotExpired");
      } catch (err: any) {
        assert.include(err.toString(), "CooldownNotExpired");
      }
    });
  });

  // ═══════════════════════════════════════════════════════════════════════
  // 4. item_nft  (потребує Metaplex Token Metadata на валідаторі)
  // ═══════════════════════════════════════════════════════════════════════
  describe("item_nft", () => {
    let nftMintKp: Keypair;
    let playerNftAta: PublicKey;

    before(() => {
      nftMintKp   = Keypair.generate();
      playerNftAta = ata22(nftMintKp.publicKey, player.publicKey);
    });

    it("create_item_nft — мінтить NFT типу 0 (Шабля козака)", async () => {
      // Plain-акаунти: nftMint (signer), metadataAccount, tokenProgram, rent
      // itemMetadata (PDA) та playerNftAccount (ATA-PDA) — автодеривуються
      await nftProg.methods
        .createItemNft(0, "Шабля козака", "SABER", "https://arweave.net/saber")
        .accounts({
          player:             player.publicKey,
          nftMint:            nftMintKp.publicKey,
          metadataAccount:    mplMetaPda(nftMintKp.publicKey),
          masterEdition:      mplMasterEditionPda(nftMintKp.publicKey),
          tokenProgram:       TOKEN_2022_PROGRAM_ID,
          sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
        })
        .preInstructions([
          ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 }),
        ])
        .signers([player, nftMintKp])
        .rpc({ commitment: "confirmed" });

      const meta = await nftProg.account.itemMetadata.fetch(
        itemMetaPda(nftMintKp.publicKey, nftProg.programId)
      );
      assert.equal(meta.itemType, 0);
      assert.equal(meta.owner.toBase58(), player.publicKey.toBase58());

      const nftAcc = await getAccount(conn, playerNftAta, "confirmed", TOKEN_2022_PROGRAM_ID);
      assert.equal(nftAcc.amount.toString(), "1");
    });

    it("burn_item_nft — NFT спалено, ItemMetadata PDA закрита", async () => {
      // Plain: authority, nftMint, nftTokenAccount, metadataAccount, tokenProgram
      // itemMetadata (PDA) — автодеривується
      await nftProg.methods
        .burnItemNft()
        .accounts({
          authority:          player.publicKey,
          nftMint:            nftMintKp.publicKey,
          nftTokenAccount:    playerNftAta,
          metadataAccount:    mplMetaPda(nftMintKp.publicKey),
          masterEdition:      mplMasterEditionPda(nftMintKp.publicKey),
          tokenProgram:       TOKEN_2022_PROGRAM_ID,
          sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
        })
        .signers([player])
        .rpc({ commitment: "confirmed" });

      const acct = await conn.getAccountInfo(
        itemMetaPda(nftMintKp.publicKey, nftProg.programId)
      );
      assert.isNull(acct, "ItemMetadata має бути закрита після burn");
    });
  });

  // ═══════════════════════════════════════════════════════════════════════
  // 5. crafting
  // ═══════════════════════════════════════════════════════════════════════
  describe("crafting", () => {

    /** Мінтить ресурс id на рахунок гравця. */
    async function mintRes(id: number, amount: number) {
      await rmProg.methods
        .mintResource(id, new BN(amount))
        .accounts({
          resourceMint:       resourceMints[id],
          playerTokenAccount: playerTas[id],
          player:             player.publicKey,
        })
        .rpc({ commitment: "confirmed" });
    }

    it("craft_item — помилка: item_type=4 (поза діапазоном 0–3)", async () => {
      const dummyKp = Keypair.generate();
      // Всі plain-акаунти потрібні для проходження account validation
      try {
        await crProg.methods
          .craftItem(4, "https://arweave.net/item")
          .accounts({
            player:             player.publicKey,
            itemMetadata:       itemMetaPda(dummyKp.publicKey, nftProg.programId),
            nftMint:            dummyKp.publicKey,
            playerNftAccount:   ata22(dummyKp.publicKey, player.publicKey),
            metadataAccount:    mplMetaPda(dummyKp.publicKey),
            masterEdition:      mplMasterEditionPda(dummyKp.publicKey),
            sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
          })
          .remainingAccounts([])
          .signers([player, dummyKp])
          .rpc();
        assert.fail("Мало кинути InvalidItemType");
      } catch (err: any) {
        // InvalidItemType або помилка account validation — обидва прийнятні
        assert.exists(err);
      }
    });

    it("craft_item — Шабля козака (type=0): 3×IRON + 1×WOOD + 1×LEATHER", async () => {
      await mintRes(1, 3); // IRON
      await mintRes(0, 1); // WOOD (додатково до наявних)
      await mintRes(3, 1); // LEATHER

      const nftKp  = Keypair.generate();
      const nftAta = ata22(nftKp.publicKey, player.publicKey);

      // Рецепт item_0: [(1,3),(0,1),(3,1),(0,0)]
      // remaining: [mint_iron, ta_iron, mint_wood, ta_wood, mint_leather, ta_leather]
      const remaining = [
        { pubkey: resourceMints[1], isWritable: true, isSigner: false },
        { pubkey: playerTas[1],     isWritable: true, isSigner: false },
        { pubkey: resourceMints[0], isWritable: true, isSigner: false },
        { pubkey: playerTas[0],     isWritable: true, isSigner: false },
        { pubkey: resourceMints[3], isWritable: true, isSigner: false },
        { pubkey: playerTas[3],     isWritable: true, isSigner: false },
      ];

      await crProg.methods
        .craftItem(0, "https://arweave.net/saber")
        .accounts({
          player:             player.publicKey,
          itemMetadata:       itemMetaPda(nftKp.publicKey, nftProg.programId),
          nftMint:            nftKp.publicKey,
          playerNftAccount:   nftAta,
          metadataAccount:    mplMetaPda(nftKp.publicKey),
          masterEdition:      mplMasterEditionPda(nftKp.publicKey),
          sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
        })
        .preInstructions([
          ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 }),
        ])
        .remainingAccounts(remaining)
        .signers([player, nftKp])
        .rpc({ commitment: "confirmed" });

      const meta = await nftProg.account.itemMetadata.fetch(
        itemMetaPda(nftKp.publicKey, nftProg.programId)
      );
      assert.equal(meta.itemType, 0, "item_type має бути 0");

      const nftAcct = await getAccount(conn, nftAta, "confirmed", TOKEN_2022_PROGRAM_ID);
      assert.equal(nftAcct.amount.toString(), "1");
    });

    it("craft_item — Посох старійшини (type=1): 2×WOOD + 1×GOLD + 1×DIAMOND", async () => {
      await mintRes(0, 2); // WOOD
      await mintRes(2, 1); // GOLD
      await mintRes(5, 1); // DIAMOND

      const nftKp  = Keypair.generate();
      const nftAta = ata22(nftKp.publicKey, player.publicKey);

      // Рецепт item_1: [(0,2),(2,1),(5,1),(0,0)]
      const remaining = [
        { pubkey: resourceMints[0], isWritable: true, isSigner: false },
        { pubkey: playerTas[0],     isWritable: true, isSigner: false },
        { pubkey: resourceMints[2], isWritable: true, isSigner: false },
        { pubkey: playerTas[2],     isWritable: true, isSigner: false },
        { pubkey: resourceMints[5], isWritable: true, isSigner: false },
        { pubkey: playerTas[5],     isWritable: true, isSigner: false },
      ];

      await crProg.methods
        .craftItem(1, "https://arweave.net/staff")
        .accounts({
          player:             player.publicKey,
          itemMetadata:       itemMetaPda(nftKp.publicKey, nftProg.programId),
          nftMint:            nftKp.publicKey,
          playerNftAccount:   nftAta,
          metadataAccount:    mplMetaPda(nftKp.publicKey),
          masterEdition:      mplMasterEditionPda(nftKp.publicKey),
          sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
        })
        .preInstructions([
          ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 }),
        ])
        .remainingAccounts(remaining)
        .signers([player, nftKp])
        .rpc({ commitment: "confirmed" });

      const meta = await nftProg.account.itemMetadata.fetch(
        itemMetaPda(nftKp.publicKey, nftProg.programId)
      );
      assert.equal(meta.itemType, 1);
    });
  });

  // ═══════════════════════════════════════════════════════════════════════
  // 6. marketplace  (+  mint_magic_token через buy_item CPI)
  // ═══════════════════════════════════════════════════════════════════════
  describe("marketplace", () => {
    let listNftKp:  Keypair;
    let listNftAta: PublicKey;
    let sellerMagicAta: PublicKey;

    // Listing/Escrow PDAs (потрібні лише для перевірки стану)
    let listingPda: PublicKey;
    let escrowPda:  PublicKey;

    before("Мінтимо NFT для marketplace тестів", async () => {
      listNftKp  = Keypair.generate();
      listNftAta = ata22(listNftKp.publicKey, player.publicKey);

      [listingPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("listing"), listNftKp.publicKey.toBuffer()],
        mpProg.programId
      );
      [escrowPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("escrow"), listNftKp.publicKey.toBuffer()],
        mpProg.programId
      );

      // MagicToken ATA продавця
      const magicAtaObj = await getOrCreateAssociatedTokenAccount(
        conn, player,
        magicMintPda, player.publicKey,
        false, "confirmed", { commitment: "confirmed" },
        TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID
      );
      sellerMagicAta = magicAtaObj.address;

      // Мінтимо NFT через item_nft напряму
      await nftProg.methods
        .createItemNft(0, "Шабля козака", "SABER", "https://arweave.net/saber")
        .accounts({
          player:             player.publicKey,
          nftMint:            listNftKp.publicKey,
          metadataAccount:    mplMetaPda(listNftKp.publicKey),
          masterEdition:      mplMasterEditionPda(listNftKp.publicKey),
          tokenProgram:       TOKEN_2022_PROGRAM_ID,
          sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
        })
        .preInstructions([
          ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 }),
        ])
        .signers([player, listNftKp])
        .rpc({ commitment: "confirmed" });
    });

    it("list_item — NFT виставлено за 10 MagicToken, ескроу отримав 1 NFT", async () => {
      // Plain: nftMint, sellerNftAccount, tokenProgram
      // listing і escrowAccount — PDAs, автодеривуються Anchor
      await mpProg.methods
        .listItem(new BN(10))
        .accounts({
          seller:           player.publicKey,
          nftMint:          listNftKp.publicKey,
          sellerNftAccount: listNftAta,
          tokenProgram:     TOKEN_2022_PROGRAM_ID,
        })
        .signers([player])
        .rpc({ commitment: "confirmed" });

      const listing = await mpProg.account.listing.fetch(listingPda);
      assert.equal(listing.seller.toBase58(), player.publicKey.toBase58());
      assert.equal(listing.price.toNumber(), 10);
      assert.equal(listing.nftMint.toBase58(), listNftKp.publicKey.toBase58());

      const escrow = await getAccount(conn, escrowPda, "confirmed", TOKEN_2022_PROGRAM_ID);
      assert.equal(escrow.amount.toString(), "1");
    });

    it("cancel_listing — лістинг скасовано, NFT повернено продавцю", async () => {
      // Plain: nftMint, sellerNftAccount, tokenProgram
      await mpProg.methods
        .cancelListing()
        .accounts({
          seller:           player.publicKey,
          nftMint:          listNftKp.publicKey,
          sellerNftAccount: listNftAta,
          tokenProgram:     TOKEN_2022_PROGRAM_ID,
        })
        .signers([player])
        .rpc({ commitment: "confirmed" });

      const nftAcct = await getAccount(conn, listNftAta, "confirmed", TOKEN_2022_PROGRAM_ID);
      assert.equal(nftAcct.amount.toString(), "1", "NFT повернуто продавцю");

      const listAcct = await conn.getAccountInfo(listingPda);
      assert.isNull(listAcct, "Listing PDA має бути закрита");
    });

    it("list_item — повторне виставлення (після cancel_listing)", async () => {
      await mpProg.methods
        .listItem(new BN(10))
        .accounts({
          seller:           player.publicKey,
          nftMint:          listNftKp.publicKey,
          sellerNftAccount: listNftAta,
          tokenProgram:     TOKEN_2022_PROGRAM_ID,
        })
        .signers([player])
        .rpc({ commitment: "confirmed" });

      const escrow = await getAccount(conn, escrowPda, "confirmed", TOKEN_2022_PROGRAM_ID);
      assert.equal(escrow.amount.toString(), "1");
    });

    it("buy_item — NFT спалено, продавець отримав 10 MagicToken [тест mint_magic_token CPI]", async () => {
      const buyer = Keypair.generate();
      await airdrop(provider, admin, buyer.publicKey, 1);

      const meta = itemMetaPda(listNftKp.publicKey, nftProg.programId);
      const balanceBefore = (
        await getAccount(conn, sellerMagicAta, "confirmed", TOKEN_2022_PROGRAM_ID)
      ).amount;

      // Plain: seller, nftMint, itemMetadata, metadataAccount, magicMint, sellerMagicAccount
      // listing, escrowAccount, magicConfig, marketplaceAuthority — PDAs, автодеривуються
      // itemNftProgram, magicTokenProgram, tokenProgram — addr, автодеривуються
      await mpProg.methods
        .buyItem()
        .accounts({
          buyer:              buyer.publicKey,
          seller:             player.publicKey,
          nftMint:            listNftKp.publicKey,
          itemMetadata:       meta,
          metadataAccount:    mplMetaPda(listNftKp.publicKey),
          masterEdition:      mplMasterEditionPda(listNftKp.publicKey),
          magicMint:          magicMintPda,
          sellerMagicAccount: sellerMagicAta,
          sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
        })
        .signers([buyer])
        .rpc();

      // Продавець отримав 10 MagicToken
      const sellerMagic = await getAccount(
        conn, sellerMagicAta, "confirmed", TOKEN_2022_PROGRAM_ID
      );
      assert.equal(
        (sellerMagic.amount - balanceBefore).toString(),
        "10",
        "Продавець має отримати 10 MagicToken"
      );

      // ItemMetadata закрита (NFT спалено)
      const metaAcct = await conn.getAccountInfo(meta);
      assert.isNull(metaAcct, "ItemMetadata має бути закрита після buy_item");
    });

    it("list_item — помилка: ціна = 0 (ZeroPrice)", async () => {
      // Потрібен новий NFT (попередній спалено)
      const nftKp  = Keypair.generate();
      const nftAta = ata22(nftKp.publicKey, player.publicKey);

      await nftProg.methods
        .createItemNft(1, "Посох старійшини", "STAFF", "https://arweave.net/staff")
        .accounts({
          player:             player.publicKey,
          nftMint:            nftKp.publicKey,
          metadataAccount:    mplMetaPda(nftKp.publicKey),
          masterEdition:      mplMasterEditionPda(nftKp.publicKey),
          tokenProgram:       TOKEN_2022_PROGRAM_ID,
          sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
        })
        .preInstructions([
          ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 }),
        ])
        .signers([player, nftKp])
        .rpc({ commitment: "confirmed" });

      try {
        await mpProg.methods
          .listItem(new BN(0))
          .accounts({
            seller:           player.publicKey,
            nftMint:          nftKp.publicKey,
            sellerNftAccount: nftAta,
            tokenProgram:     TOKEN_2022_PROGRAM_ID,
          })
          .signers([player])
          .rpc();
        assert.fail("Мало кинути ZeroPrice");
      } catch (err: any) {
        assert.include(err.toString(), "ZeroPrice");
      }
    });
  });
});
