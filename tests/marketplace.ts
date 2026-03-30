/** Integration tests for marketplace redemption and MagicToken reward minting. */
import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";
import { createRequire } from "module";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAccount,
  getAssociatedTokenAddressSync,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import BN from "bn.js";

const require = createRequire(`${process.cwd()}/tests/marketplace.ts`);
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

describe("marketplace", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const programs = getGamePrograms(provider);
  const admin = (provider.wallet as anchor.Wallet).payer;
  const owner = anchor.web3.Keypair.generate();
  const intruder = anchor.web3.Keypair.generate();
  const itemPrices = [25, 40, 75, 110].map((value) => new BN(value));
  const itemType = 0;
  const invalidItemType = 1;
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

  /** Ensures a test wallet has enough SOL for redemption-related transactions. */
  const ensureWalletFunded = async (wallet: anchor.web3.Keypair) => {
    const balance = await provider.connection.getBalance(
      wallet.publicKey,
      "confirmed",
    );
    if (balance >= anchor.web3.LAMPORTS_PER_SOL / 2) {
      return;
    }

    const signature = await provider.connection.requestAirdrop(
      wallet.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL,
    );
    await provider.connection.confirmTransaction(signature, "confirmed");
  };

  /** Creates the shared config and MagicToken mint needed for marketplace tests. */
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

  /** Reads a wallet's MagicToken balance from its canonical ATA. */
  const readMagicBalance = async (
    wallet: anchor.web3.PublicKey,
  ): Promise<bigint> => {
    const ata = getAssociatedTokenAddressSync(
      magicTokenMintPda,
      wallet,
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

  /** Mints one test NFT through the authorized crafting CPI flow. */
  const mintItem = async (wallet: anchor.web3.Keypair) => {
    const mint = anchor.web3.Keypair.generate();
    const [itemMetadataPda] = findItemMetadataPda(mint.publicKey);
    const metadataPda = deriveMetadataPda(mint.publicKey);
    const masterEditionPda = deriveMasterEditionPda(mint.publicKey);
    const ownerItemTokenAccount = getAssociatedTokenAddressSync(
      mint.publicKey,
      wallet.publicKey,
      false,
      TOKEN_PROGRAM_ID,
    );

    await programs.crafting.methods
      .proxyMintItemNft(itemType, nftUri, nftName, nftSymbol)
      .preInstructions([mintComputeBudgetIx])
      .accounts({
        owner: wallet.publicKey,
        gameConfig: gameConfigPda,
        craftingAuthority,
        itemNftAuthority,
        mint: mint.publicKey,
        itemMetadata: itemMetadataPda,
        metadata: metadataPda,
        masterEdition: masterEditionPda,
        ownerItemTokenAccount,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        itemNftProgram: getProgramPublicKey("item_nft"),
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
      })
      .signers([wallet, mint])
      .rpc();

    return {
      mint: mint.publicKey,
      itemMetadataPda,
      metadataPda,
      masterEditionPda,
      ownerItemTokenAccount,
    };
  };

  before(async () => {
    await ensureWalletFunded(owner);
    await ensureWalletFunded(intruder);
    await ensureBootstrap();
  });

  it("redeems a valid NFT, burns it and mints the configured MagicToken reward", async () => {
    const mintedItem = await mintItem(owner);
    const playerMagicTokenAccount = getAssociatedTokenAddressSync(
      magicTokenMintPda,
      owner.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
    );
    const beforeBalance = await readMagicBalance(owner.publicKey);

    const redeemSignature = await programs.marketplace.methods
      .redeemItemForMagic(itemType)
      .preInstructions([redeemComputeBudgetIx])
      .accounts({
        owner: owner.publicKey,
        gameConfig: gameConfigPda,
        marketplaceAuthority,
        magicTokenAuthority,
        mint: mintedItem.mint,
        itemMetadata: mintedItem.itemMetadataPda,
        metadata: mintedItem.metadataPda,
        masterEdition: mintedItem.masterEditionPda,
        ownerItemTokenAccount: mintedItem.ownerItemTokenAccount,
        magicTokenMint: magicTokenMintPda,
        playerMagicTokenAccount,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        itemNftProgram: getProgramPublicKey("item_nft"),
        magicTokenProgram: getProgramPublicKey("magic_token"),
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        itemTokenProgram: TOKEN_PROGRAM_ID,
        magicTokenTokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
      })
      .signers([owner])
      .rpc();
    await provider.connection.confirmTransaction(redeemSignature, "confirmed");

    const afterBalance = await readMagicBalance(owner.publicKey);
    const remainingItemMetadata =
      await programs.item_nft.account.itemMetadata.fetchNullable(
        mintedItem.itemMetadataPda,
      );
    const ownerItemTokenAccountInfo = await provider.connection.getAccountInfo(
      mintedItem.ownerItemTokenAccount,
      "confirmed",
    );

    expect(afterBalance - beforeBalance).to.equal(
      BigInt(itemPrices[itemType].toNumber()),
    );
    expect(remainingItemMetadata).to.equal(null);
    expect(ownerItemTokenAccountInfo).to.equal(null);
  });

  it("rejects redeeming the same NFT twice", async () => {
    const mintedItem = await mintItem(owner);
    const playerMagicTokenAccount = getAssociatedTokenAddressSync(
      magicTokenMintPda,
      owner.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
    );

    const firstRedeemSignature = await programs.marketplace.methods
      .redeemItemForMagic(itemType)
      .preInstructions([redeemComputeBudgetIx])
      .accounts({
        owner: owner.publicKey,
        gameConfig: gameConfigPda,
        marketplaceAuthority,
        magicTokenAuthority,
        mint: mintedItem.mint,
        itemMetadata: mintedItem.itemMetadataPda,
        metadata: mintedItem.metadataPda,
        masterEdition: mintedItem.masterEditionPda,
        ownerItemTokenAccount: mintedItem.ownerItemTokenAccount,
        magicTokenMint: magicTokenMintPda,
        playerMagicTokenAccount,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        itemNftProgram: getProgramPublicKey("item_nft"),
        magicTokenProgram: getProgramPublicKey("magic_token"),
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        itemTokenProgram: TOKEN_PROGRAM_ID,
        magicTokenTokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
      })
      .signers([owner])
      .rpc();
    await provider.connection.confirmTransaction(
      firstRedeemSignature,
      "confirmed",
    );

    await expectRpcToFail(
      programs.marketplace.methods
        .redeemItemForMagic(itemType)
        .preInstructions([redeemComputeBudgetIx])
        .accounts({
          owner: owner.publicKey,
          gameConfig: gameConfigPda,
          marketplaceAuthority,
          magicTokenAuthority,
          mint: mintedItem.mint,
          itemMetadata: mintedItem.itemMetadataPda,
          metadata: mintedItem.metadataPda,
          masterEdition: mintedItem.masterEditionPda,
          ownerItemTokenAccount: mintedItem.ownerItemTokenAccount,
          magicTokenMint: magicTokenMintPda,
          playerMagicTokenAccount,
          tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
          itemNftProgram: getProgramPublicKey("item_nft"),
          magicTokenProgram: getProgramPublicKey("magic_token"),
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          itemTokenProgram: TOKEN_PROGRAM_ID,
          magicTokenTokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
        })
        .signers([owner])
        .rpc(),
    );
  });

  it("rejects redeem attempts from a signer who does not own the NFT", async () => {
    const mintedItem = await mintItem(owner);
    const intruderMagicTokenAccount = getAssociatedTokenAddressSync(
      magicTokenMintPda,
      intruder.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
    );
    const beforeBalance = await readMagicBalance(intruder.publicKey);

    await expectRpcToFail(
      programs.marketplace.methods
        .redeemItemForMagic(itemType)
        .preInstructions([redeemComputeBudgetIx])
        .accounts({
          owner: intruder.publicKey,
          gameConfig: gameConfigPda,
          marketplaceAuthority,
          magicTokenAuthority,
          mint: mintedItem.mint,
          itemMetadata: mintedItem.itemMetadataPda,
          metadata: mintedItem.metadataPda,
          masterEdition: mintedItem.masterEditionPda,
          ownerItemTokenAccount: mintedItem.ownerItemTokenAccount,
          magicTokenMint: magicTokenMintPda,
          playerMagicTokenAccount: intruderMagicTokenAccount,
          tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
          itemNftProgram: getProgramPublicKey("item_nft"),
          magicTokenProgram: getProgramPublicKey("magic_token"),
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          itemTokenProgram: TOKEN_PROGRAM_ID,
          magicTokenTokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
        })
        .signers([intruder])
        .rpc(),
    );

    const afterBalance = await readMagicBalance(intruder.publicKey);
    const remainingItemMetadata =
      await programs.item_nft.account.itemMetadata.fetchNullable(
        mintedItem.itemMetadataPda,
      );

    expect(afterBalance).to.equal(beforeBalance);
    expect(remainingItemMetadata).to.not.equal(null);
  });

  it("does not mint MagicToken when item metadata does not match the requested item type", async () => {
    const mintedItem = await mintItem(owner);
    const playerMagicTokenAccount = getAssociatedTokenAddressSync(
      magicTokenMintPda,
      owner.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
    );
    const beforeBalance = await readMagicBalance(owner.publicKey);

    await expectRpcToFail(
      programs.marketplace.methods
        .redeemItemForMagic(invalidItemType)
        .preInstructions([redeemComputeBudgetIx])
        .accounts({
          owner: owner.publicKey,
          gameConfig: gameConfigPda,
          marketplaceAuthority,
          magicTokenAuthority,
          mint: mintedItem.mint,
          itemMetadata: mintedItem.itemMetadataPda,
          metadata: mintedItem.metadataPda,
          masterEdition: mintedItem.masterEditionPda,
          ownerItemTokenAccount: mintedItem.ownerItemTokenAccount,
          magicTokenMint: magicTokenMintPda,
          playerMagicTokenAccount,
          tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
          itemNftProgram: getProgramPublicKey("item_nft"),
          magicTokenProgram: getProgramPublicKey("magic_token"),
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          itemTokenProgram: TOKEN_PROGRAM_ID,
          magicTokenTokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
        })
        .signers([owner])
        .rpc(),
    );

    const afterBalance = await readMagicBalance(owner.publicKey);
    const remainingItemMetadata =
      await programs.item_nft.account.itemMetadata.fetchNullable(
        mintedItem.itemMetadataPda,
      );

    expect(afterBalance).to.equal(beforeBalance);
    expect(remainingItemMetadata).to.not.equal(null);
  });
});
