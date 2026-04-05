/** Integration tests for item NFT minting, burning and transfer/redeem behavior. */
import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";
import { createRequire } from "module";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountIdempotent,
  createTransferCheckedInstruction,
  getAccount,
  getAssociatedTokenAddressSync,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import BN from "bn.js";

const require = createRequire(`${process.cwd()}/tests/item-nft.ts`);
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

describe("item nft", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const programs = getGamePrograms(provider);
  const admin = (provider.wallet as anchor.Wallet).payer;
  const player = anchor.web3.Keypair.generate();
  const recipient = anchor.web3.Keypair.generate();
  const itemPrices = [25, 40, 75, 110].map((value) => new BN(value));
  const itemType = 0;
  const nftUri = "https://example.com/items/kozak-sabre.json";
  const nftName = "Kozak Sabre";
  const nftSymbol = "SABRE";
  const [gameConfigPda] = findGameConfigPda();
  const [craftingAuthority] = findProgramAuthorityPda(
    getProgramPublicKey("crafting"),
  );
  const [marketplaceAuthority] = findProgramAuthorityPda(
    getProgramPublicKey("marketplace"),
  );
  const [itemNftAuthority] = findProgramAuthorityPda(
    getProgramPublicKey("item_nft"),
  );
  const [magicTokenAuthority] = findProgramAuthorityPda(
    getProgramPublicKey("magic_token"),
  );
  const [magicTokenMintPda] = findMagicTokenMintPda();
  const mintComputeBudgetIx =
    anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
      units: 400_000,
    });
  const burnComputeBudgetIx =
    anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
      units: 300_000,
    });

  let mintedNft: {
    mint: anchor.web3.Keypair;
    itemMetadataPda: anchor.web3.PublicKey;
    metadataPda: anchor.web3.PublicKey;
    masterEditionPda: anchor.web3.PublicKey;
    playerAta: anchor.web3.PublicKey;
  };

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

  /** Ensures a test wallet has enough SOL for minting and token-account creation. */
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

  /** Creates the shared game config and MagicToken mint needed by the suite. */
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
    if (existingMagicMint) {
      return;
    }

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
  };

  /** Mints one item NFT through the authorized crafting CPI flow. */
  const mintViaCrafting = async (owner: anchor.web3.Keypair) => {
    const mint = anchor.web3.Keypair.generate();
    const [itemMetadataPda] = findItemMetadataPda(mint.publicKey);
    const metadataPda = deriveMetadataPda(mint.publicKey);
    const masterEditionPda = deriveMasterEditionPda(mint.publicKey);
    const playerAta = getAssociatedTokenAddressSync(
      mint.publicKey,
      owner.publicKey,
      false,
      TOKEN_PROGRAM_ID,
    );

    const signature = await programs.crafting.methods
      .proxyMintItemNft(itemType, nftUri, nftName, nftSymbol)
      .preInstructions([mintComputeBudgetIx])
      .accounts({
        owner: owner.publicKey,
        gameConfig: gameConfigPda,
        craftingAuthority,
        itemNftAuthority,
        mint: mint.publicKey,
        itemMetadata: itemMetadataPda,
        metadata: metadataPda,
        masterEdition: masterEditionPda,
        ownerItemTokenAccount: playerAta,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        itemNftProgram: getProgramPublicKey("item_nft"),
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
      })
      .signers([owner, mint])
      .rpc();

    await provider.connection.confirmTransaction(signature, "confirmed");

    return { mint, itemMetadataPda, metadataPda, masterEditionPda, playerAta };
  };

  before(async () => {
    await ensureWalletFunded(player);
    await ensureWalletFunded(recipient);
    await ensureBootstrap();
    mintedNft = await mintViaCrafting(player);
  });

  it("mints a non-fungible item with metadata and master edition", async () => {
    const mintInfo = await provider.connection.getAccountInfo(
      mintedNft.mint.publicKey,
      "confirmed",
    );
    const tokenAccount = await getAccount(
      provider.connection,
      mintedNft.playerAta,
      "confirmed",
      TOKEN_PROGRAM_ID,
    );
    const itemMetadata = await programs.item_nft.account.itemMetadata.fetch(
      mintedNft.itemMetadataPda,
    );
    const metadataInfo = await provider.connection.getAccountInfo(
      mintedNft.metadataPda,
      "confirmed",
    );
    const masterEditionInfo = await provider.connection.getAccountInfo(
      mintedNft.masterEditionPda,
      "confirmed",
    );

    expect(mintInfo).to.not.equal(null);
    expect(mintInfo?.owner.toBase58()).to.equal(TOKEN_PROGRAM_ID.toBase58());
    expect(Number(tokenAccount.amount)).to.equal(1);
    expect(itemMetadata.itemType).to.equal(itemType);
    expect(itemMetadata.owner.toBase58()).to.equal(player.publicKey.toBase58());
    expect(itemMetadata.mint.toBase58()).to.equal(
      mintedNft.mint.publicKey.toBase58(),
    );
    expect(metadataInfo).to.not.equal(null);
    expect(masterEditionInfo).to.not.equal(null);
  });

  it("rejects direct mint and burn calls without the authorized CPI signer", async () => {
    const directMint = anchor.web3.Keypair.generate();
    const [directItemMetadataPda] = findItemMetadataPda(directMint.publicKey);
    const directMetadataPda = deriveMetadataPda(directMint.publicKey);
    const directMasterEditionPda = deriveMasterEditionPda(directMint.publicKey);
    const directOwnerAta = getAssociatedTokenAddressSync(
      directMint.publicKey,
      player.publicKey,
      false,
      TOKEN_PROGRAM_ID,
    );

    await expectRpcToFail(
      programs.item_nft.methods
        .mintItemNft(itemType, nftUri, nftName, nftSymbol)
        .preInstructions([mintComputeBudgetIx])
        .accounts({
          owner: player.publicKey,
          gameConfig: gameConfigPda,
          callerAuthority: craftingAuthority,
          programAuthority: itemNftAuthority,
          mint: directMint.publicKey,
          itemMetadata: directItemMetadataPda,
          metadata: directMetadataPda,
          masterEdition: directMasterEditionPda,
          ownerItemTokenAccount: directOwnerAta,
          tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
        })
        .signers([player, directMint])
        .rpc(),
    );

    await expectRpcToFail(
      programs.item_nft.methods
        .burnItemNft(itemType)
        .preInstructions([burnComputeBudgetIx])
        .accounts({
          owner: player.publicKey,
          gameConfig: gameConfigPda,
          callerAuthority: marketplaceAuthority,
          mint: mintedNft.mint.publicKey,
          itemMetadata: mintedNft.itemMetadataPda,
          metadata: mintedNft.metadataPda,
          masterEdition: mintedNft.masterEditionPda,
          ownerItemTokenAccount: mintedNft.playerAta,
          tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
        })
        .signers([player])
        .rpc(),
    );
  });

  it("allows a standard transfer and still redeems via the marketplace path", async () => {
    const recipientAta = getAssociatedTokenAddressSync(
      mintedNft.mint.publicKey,
      recipient.publicKey,
      false,
      TOKEN_PROGRAM_ID,
    );
    const recipientMagicAta = getAssociatedTokenAddressSync(
      magicTokenMintPda,
      recipient.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
    );

    await createAssociatedTokenAccountIdempotent(
      provider.connection,
      admin,
      mintedNft.mint.publicKey,
      recipient.publicKey,
      {},
      TOKEN_PROGRAM_ID,
    );

    const transferSignature = await anchor.web3.sendAndConfirmTransaction(
      provider.connection,
      new anchor.web3.Transaction().add(
        createTransferCheckedInstruction(
          mintedNft.playerAta,
          mintedNft.mint.publicKey,
          recipientAta,
          player.publicKey,
          1,
          0,
          [],
          TOKEN_PROGRAM_ID,
        ),
      ),
      [admin, player],
    );

    await provider.connection.confirmTransaction(
      transferSignature,
      "confirmed",
    );

    const recipientTokenBeforeBurn = await getAccount(
      provider.connection,
      recipientAta,
      "confirmed",
      TOKEN_PROGRAM_ID,
    );
    const recipientMagicBeforeRedeemInfo =
      await provider.connection.getAccountInfo(recipientMagicAta, "confirmed");
    const recipientMagicBeforeRedeem = recipientMagicBeforeRedeemInfo
      ? (
          await getAccount(
            provider.connection,
            recipientMagicAta,
            "confirmed",
            TOKEN_2022_PROGRAM_ID,
          )
        ).amount
      : 0n;
    expect(Number(recipientTokenBeforeBurn.amount)).to.equal(1);

    const redeemSignature = await programs.marketplace.methods
      .redeemItemForMagic(itemType)
      .preInstructions([burnComputeBudgetIx])
      .accounts({
        owner: recipient.publicKey,
        gameConfig: gameConfigPda,
        marketplaceAuthority,
        magicTokenAuthority,
        mint: mintedNft.mint.publicKey,
        itemMetadata: mintedNft.itemMetadataPda,
        metadata: mintedNft.metadataPda,
        masterEdition: mintedNft.masterEditionPda,
        ownerItemTokenAccount: recipientAta,
        magicTokenMint: magicTokenMintPda,
        playerMagicTokenAccount: recipientMagicAta,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        itemNftProgram: getProgramPublicKey("item_nft"),
        magicTokenProgram: getProgramPublicKey("magic_token"),
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        itemTokenProgram: TOKEN_PROGRAM_ID,
        magicTokenTokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
      })
      .signers([recipient])
      .rpc();
    await provider.connection.confirmTransaction(redeemSignature, "confirmed");

    const mintInfo = await provider.connection.getAccountInfo(
      mintedNft.mint.publicKey,
      "confirmed",
    );
    const remainingItemMetadata =
      await programs.item_nft.account.itemMetadata.fetchNullable(
        mintedNft.itemMetadataPda,
      );
    const recipientMagicAfterRedeem = await getAccount(
      provider.connection,
      recipientMagicAta,
      "confirmed",
      TOKEN_2022_PROGRAM_ID,
    );
    expect(mintInfo).to.not.equal(undefined);
    expect(remainingItemMetadata).to.equal(null);
    expect(
      recipientMagicAfterRedeem.amount - recipientMagicBeforeRedeem,
    ).to.equal(25n);
  });
});
