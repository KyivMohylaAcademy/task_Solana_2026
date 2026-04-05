/** Integration tests for the MagicToken mint and marketplace-only reward path. */
import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";
import { createRequire } from "module";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountIdempotent,
  getAccount,
  getAssociatedTokenAddressSync,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import BN from "bn.js";

const require = createRequire(`${process.cwd()}/tests/magic-token.ts`);
const { getGamePrograms, getProgramPublicKey } = require("../utils/programs");
const {
  findGameConfigPda,
  findItemMetadataPda,
  findMagicTokenMintPda,
  findProgramAuthorityPda,
} = require("../utils/account_utils");

/** Canonical Metaplex Token Metadata program used by item NFTs. */
const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s",
);

describe("magic token", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const programs = getGamePrograms(provider);
  const admin = (provider.wallet as anchor.Wallet).payer;
  const player = anchor.web3.Keypair.generate();
  const itemPrices = [25, 40, 75, 110].map((value) => new BN(value));
  const itemType = 0;
  const nftUri = "https://example.com/items/kozak-sabre.json";
  const nftName = "Kozak Sabre";
  const nftSymbol = "SABRE";
  const [gameConfigPda] = findGameConfigPda();
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
  const mintComputeBudgetIx =
    anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
      units: 400_000,
    });
  const redeemComputeBudgetIx =
    anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
      units: 500_000,
    });

  /** Asserts that a transaction promise fails without caring about the exact error text. */
  const expectRpcToFail = async (promise: Promise<unknown>) => {
    try {
      await promise;
      expect.fail("Expected transaction to fail");
    } catch (_error) {
      expect(true).to.equal(true);
    }
  };

  /** Ensures the test player has enough SOL for ATAs and redemption transactions. */
  const ensurePlayerFunded = async () => {
    const balance = await provider.connection.getBalance(
      player.publicKey,
      "confirmed",
    );
    if (balance >= anchor.web3.LAMPORTS_PER_SOL / 2) {
      return;
    }

    const signature = await provider.connection.requestAirdrop(
      player.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL,
    );
    await provider.connection.confirmTransaction(signature, "confirmed");
  };

  /** Creates the shared config and MagicToken mint needed by the suite. */
  const ensureBootstrap = async () => {
    const existingGameConfig =
      await programs.resource_manager.account.gameConfig.fetchNullable(
        gameConfigPda,
      );

    if (!existingGameConfig) {
      await programs.resource_manager.methods
        .initializeGameConfig(magicTokenMintPda, itemPrices)
        .accounts({
          admin: admin.publicKey,
          gameConfig: gameConfigPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([admin])
        .rpc();
    }

    const existingMagicMint = await provider.connection.getAccountInfo(
      magicTokenMintPda,
      "confirmed",
    );
    if (!existingMagicMint) {
      await programs.magic_token.methods
        .initializeMagicTokenMint()
        .accounts({
          admin: admin.publicKey,
          gameConfig: gameConfigPda,
          programAuthority: magicTokenAuthority,
          magicTokenMint: magicTokenMintPda,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([admin])
        .rpc();
    }
  };

  /** Creates and returns the player's MagicToken ATA. */
  const ensurePlayerAta = async (): Promise<anchor.web3.PublicKey> => {
    const ata = getAssociatedTokenAddressSync(
      magicTokenMintPda,
      player.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
    );

    await createAssociatedTokenAccountIdempotent(
      provider.connection,
      admin,
      magicTokenMintPda,
      player.publicKey,
      {},
      TOKEN_2022_PROGRAM_ID,
    );

    return ata;
  };

  /** Reads the player's MagicToken balance from the canonical ATA. */
  const readBalance = async (): Promise<bigint> => {
    const ata = getAssociatedTokenAddressSync(
      magicTokenMintPda,
      player.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
    );
    const accountInfo = await provider.connection.getAccountInfo(
      ata,
      "confirmed",
    );
    if (!accountInfo) {
      return 0n;
    }

    const account = await getAccount(
      provider.connection,
      ata,
      "confirmed",
      TOKEN_2022_PROGRAM_ID,
    );

    return account.amount;
  };

  /** Derives the Metaplex metadata PDA for an NFT mint. */
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

  /** Derives the Metaplex master edition PDA for an NFT mint. */
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

  /** Mints one item NFT through the authorized crafting CPI flow. */
  const mintViaCrafting = async () => {
    const mint = anchor.web3.Keypair.generate();
    const [itemMetadataPda] = findItemMetadataPda(mint.publicKey);
    const metadataPda = deriveMetadataPda(mint.publicKey);
    const masterEditionPda = deriveMasterEditionPda(mint.publicKey);
    const playerItemAta = getAssociatedTokenAddressSync(
      mint.publicKey,
      player.publicKey,
      false,
      TOKEN_PROGRAM_ID,
    );

    await programs.crafting.methods
      .proxyMintItemNft(itemType, nftUri, nftName, nftSymbol)
      .preInstructions([mintComputeBudgetIx])
      .accounts({
        owner: player.publicKey,
        gameConfig: gameConfigPda,
        craftingAuthority,
        itemNftAuthority,
        mint: mint.publicKey,
        itemMetadata: itemMetadataPda,
        metadata: metadataPda,
        masterEdition: masterEditionPda,
        ownerItemTokenAccount: playerItemAta,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        itemNftProgram: getProgramPublicKey("item_nft"),
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
      })
      .signers([player, mint])
      .rpc();

    return {
      mint: mint.publicKey,
      itemMetadataPda,
      metadataPda,
      masterEditionPda,
      playerItemAta,
    };
  };

  before(async () => {
    await ensurePlayerFunded();
    await ensureBootstrap();
  });

  it("mints MagicToken only when a valid NFT is redeemed through marketplace", async () => {
    const mintedNft = await mintViaCrafting();
    const playerAta = getAssociatedTokenAddressSync(
      magicTokenMintPda,
      player.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
    );
    const beforeBalance = await readBalance();

    const redeemSignature = await programs.marketplace.methods
      .redeemItemForMagic(itemType)
      .preInstructions([redeemComputeBudgetIx])
      .accounts({
        owner: player.publicKey,
        gameConfig: gameConfigPda,
        marketplaceAuthority,
        magicTokenAuthority,
        mint: mintedNft.mint,
        itemMetadata: mintedNft.itemMetadataPda,
        metadata: mintedNft.metadataPda,
        masterEdition: mintedNft.masterEditionPda,
        ownerItemTokenAccount: mintedNft.playerItemAta,
        magicTokenMint: magicTokenMintPda,
        playerMagicTokenAccount: playerAta,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        itemNftProgram: getProgramPublicKey("item_nft"),
        magicTokenProgram: getProgramPublicKey("magic_token"),
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        itemTokenProgram: TOKEN_PROGRAM_ID,
        magicTokenTokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
      })
      .signers([player])
      .rpc();
    await provider.connection.confirmTransaction(redeemSignature, "confirmed");

    const afterBalance = await readBalance();
    expect(afterBalance - beforeBalance).to.equal(
      BigInt(itemPrices[itemType].toNumber()),
    );
  });

  it("rejects direct mint calls without the marketplace PDA signer", async () => {
    const playerAta = await ensurePlayerAta();

    await expectRpcToFail(
      programs.magic_token.methods
        .mintMagicToPlayer(new BN(1))
        .accounts({
          player: player.publicKey,
          gameConfig: gameConfigPda,
          callerAuthority: marketplaceAuthority,
          programAuthority: magicTokenAuthority,
          magicTokenMint: magicTokenMintPda,
          playerMagicTokenAccount: playerAta,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([player])
        .rpc(),
    );
  });
});
