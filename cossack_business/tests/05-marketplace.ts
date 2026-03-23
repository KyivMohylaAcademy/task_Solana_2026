import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey, SYSVAR_INSTRUCTIONS_PUBKEY, SystemProgram } from "@solana/web3.js";
import {
  getAssociatedTokenAddressSync,
  createAssociatedTokenAccountInstruction,
} from "@solana/spl-token";
import { expect } from "chai";
import {
  provider,
  rmProgram,
  mtProgram,
  inProgram,
  craftProgram,
  marketProgram,
  player1,
  player2,
  resourceMints,
  marketCallerAuth,
  itemPrices,
  initializeAll,
} from "./helpers/setup";
import {
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  METAPLEX_PROGRAM_ID,
  RECIPES,
  findMetadataPda,
  findMasterEditionPda,
  doMultipleSearches,
  craftNftForPlayer,
} from "./helpers/utils";

describe("05 - Marketplace", () => {
  let gameConfigPda: PublicKey;
  let mintAuthorityPda: PublicKey;
  let magicConfigPda: PublicKey;
  let magicMintAuthPda: PublicKey;
  let itemNftConfigPda: PublicKey;
  let nftAuthorityPda: PublicKey;
  let magicMintKp: Keypair;

  let nftMintForSale: Keypair;
  let craftedItemType: number;

  let listNftMint: Keypair;
  let delistNftMint: Keypair;
  let player2NftMint: Keypair;
  let listingItemType: number;

  before(async () => {
    await initializeAll();
    const setup = require("./helpers/setup");
    gameConfigPda = setup.gameConfigPda;
    mintAuthorityPda = setup.mintAuthorityPda;
    magicConfigPda = setup.magicConfigPda;
    magicMintAuthPda = setup.magicMintAuthPda;
    itemNftConfigPda = setup.itemNftConfigPda;
    nftAuthorityPda = setup.nftAuthorityPda;
    magicMintKp = setup.magicMintKp;
  });

  describe("Sell to Game", () => {
    before(async () => {
      await doMultipleSearches(player1, 15, gameConfigPda, mintAuthorityPda);

      const result = await craftNftForPlayer(player1, gameConfigPda);
      if (!result) {
        console.log("    Skipping sell tests - not enough resources");
        craftedItemType = -1;
        return;
      }
      nftMintForSale = result.mint;
      craftedItemType = result.itemType;
    });

    it("sells item and receives MagicToken at game price", async () => {
      if (craftedItemType === -1) return;

      const sellerMagicAta = getAssociatedTokenAddressSync(
        magicMintKp.publicKey, player1.publicKey, true, TOKEN_2022_PROGRAM_ID
      );
      const ataInfo = await provider.connection.getAccountInfo(sellerMagicAta);
      if (!ataInfo) {
        const ix = createAssociatedTokenAccountInstruction(
          provider.wallet.publicKey, sellerMagicAta, player1.publicKey,
          magicMintKp.publicKey, TOKEN_2022_PROGRAM_ID
        );
        await provider.sendAndConfirm(new anchor.web3.Transaction().add(ix));
      }

      const [itemMetadataPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("item_metadata"), nftMintForSale.publicKey.toBuffer()],
        inProgram.programId
      );
      const sellerNftAta = getAssociatedTokenAddressSync(
        nftMintForSale.publicKey, player1.publicKey, false, TOKEN_PROGRAM_ID
      );
      const metadataAccount = findMetadataPda(nftMintForSale.publicKey);
      const masterEdition = findMasterEditionPda(nftMintForSale.publicKey);

      const sellRemaining = [
        { pubkey: metadataAccount, isSigner: false, isWritable: true },
        { pubkey: masterEdition, isSigner: false, isWritable: true },
        { pubkey: METAPLEX_PROGRAM_ID, isSigner: false, isWritable: false },
        { pubkey: SYSVAR_INSTRUCTIONS_PUBKEY, isSigner: false, isWritable: false },
      ];

      await marketProgram.methods
        .sellItem()
        .accounts({
          seller: player1.publicKey,
          callerAuthority: marketCallerAuth,
          gameConfig: gameConfigPda,
          itemMetadata: itemMetadataPda,
          nftMint: nftMintForSale.publicKey,
          sellerNftAta: sellerNftAta,
          itemNftConfig: itemNftConfigPda,
          magicTokenConfig: magicConfigPda,
          magicTokenMint: magicMintKp.publicKey,
          magicMintAuthority: magicMintAuthPda,
          sellerMagicAta: sellerMagicAta,
          itemNftProgram: inProgram.programId,
          magicTokenProgram: mtProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          token2022Program: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .remainingAccounts(sellRemaining)
        .signers([player1])
        .rpc();

      const magicBalance = await provider.connection.getTokenAccountBalance(sellerMagicAta);
      const expectedPrice = itemPrices[craftedItemType];
      expect(parseInt(magicBalance.value.amount)).to.equal(expectedPrice);
    });
  });

  describe("List, Buy, Delist", () => {
    before(async () => {
      await doMultipleSearches(player1, 20, gameConfigPda, mintAuthorityPda);
      await doMultipleSearches(player2, 15, gameConfigPda, mintAuthorityPda);

      const result1 = await craftNftForPlayer(player1, gameConfigPda);
      if (!result1) { console.log("    Skipping list/buy/delist - not enough resources for player1 NFT #1"); return; }
      listNftMint = result1.mint;
      listingItemType = result1.itemType;

      const result2 = await craftNftForPlayer(player1, gameConfigPda);
      if (!result2) { console.log("    Skipping delist - not enough resources for player1 NFT #2"); return; }
      delistNftMint = result2.mint;

      const result3 = await craftNftForPlayer(player2, gameConfigPda);
      if (!result3) { console.log("    Skipping buy - not enough resources for player2 NFT"); return; }
      player2NftMint = result3.mint;

      // Player2 sells their NFT to the game to get MagicToken for buying
      const seller2MagicAta = getAssociatedTokenAddressSync(
        magicMintKp.publicKey, player2.publicKey, true, TOKEN_2022_PROGRAM_ID
      );
      const ataInfo = await provider.connection.getAccountInfo(seller2MagicAta);
      if (!ataInfo) {
        const ix = createAssociatedTokenAccountInstruction(
          provider.wallet.publicKey, seller2MagicAta, player2.publicKey,
          magicMintKp.publicKey, TOKEN_2022_PROGRAM_ID
        );
        await provider.sendAndConfirm(new anchor.web3.Transaction().add(ix));
      }

      const [p2ItemMetaPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("item_metadata"), player2NftMint.publicKey.toBuffer()],
        inProgram.programId
      );
      const seller2NftAta = getAssociatedTokenAddressSync(
        player2NftMint.publicKey, player2.publicKey, false, TOKEN_PROGRAM_ID
      );
      const p2Metadata = findMetadataPda(player2NftMint.publicKey);
      const p2MasterEd = findMasterEditionPda(player2NftMint.publicKey);

      await marketProgram.methods
        .sellItem()
        .accounts({
          seller: player2.publicKey,
          callerAuthority: marketCallerAuth,
          gameConfig: gameConfigPda,
          itemMetadata: p2ItemMetaPda,
          nftMint: player2NftMint.publicKey,
          sellerNftAta: seller2NftAta,
          itemNftConfig: itemNftConfigPda,
          magicTokenConfig: magicConfigPda,
          magicTokenMint: magicMintKp.publicKey,
          magicMintAuthority: magicMintAuthPda,
          sellerMagicAta: seller2MagicAta,
          itemNftProgram: inProgram.programId,
          magicTokenProgram: mtProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          token2022Program: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .remainingAccounts([
          { pubkey: p2Metadata, isSigner: false, isWritable: true },
          { pubkey: p2MasterEd, isSigner: false, isWritable: true },
          { pubkey: METAPLEX_PROGRAM_ID, isSigner: false, isWritable: false },
          { pubkey: SYSVAR_INSTRUCTIONS_PUBKEY, isSigner: false, isWritable: false },
        ])
        .signers([player2])
        .rpc();
    });

    it("lists an item on the marketplace", async () => {
      if (!listNftMint) return;

      const [itemMetadataPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("item_metadata"), listNftMint.publicKey.toBuffer()],
        inProgram.programId
      );
      const sellerNftAta = getAssociatedTokenAddressSync(
        listNftMint.publicKey, player1.publicKey, false, TOKEN_PROGRAM_ID
      );
      const [escrowAuth] = PublicKey.findProgramAddressSync(
        [Buffer.from("escrow"), listNftMint.publicKey.toBuffer()],
        marketProgram.programId
      );
      const escrowNftAta = getAssociatedTokenAddressSync(
        listNftMint.publicKey, escrowAuth, true, TOKEN_PROGRAM_ID
      );
      const [listingPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("listing"), listNftMint.publicKey.toBuffer()],
        marketProgram.programId
      );

      await marketProgram.methods
        .listItem(new anchor.BN(50))
        .accounts({
          seller: player1.publicKey,
          itemMetadata: itemMetadataPda,
          nftMint: listNftMint.publicKey,
          sellerNftAta: sellerNftAta,
          escrowAuthority: escrowAuth,
          escrowNftAta: escrowNftAta,
          listing: listingPda,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([player1])
        .rpc();

      const listing = await marketProgram.account.listing.fetch(listingPda);
      expect(listing.seller.toBase58()).to.equal(player1.publicKey.toBase58());
      expect(listing.price.toNumber()).to.equal(50);
      expect(listing.itemType).to.equal(listingItemType);

      const escrowBalance = await provider.connection.getTokenAccountBalance(escrowNftAta);
      expect(parseInt(escrowBalance.value.amount)).to.equal(1);
    });

    it("rejects listing with zero price", async () => {
      if (!delistNftMint) return;

      const [itemMetadataPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("item_metadata"), delistNftMint.publicKey.toBuffer()],
        inProgram.programId
      );
      const sellerNftAta = getAssociatedTokenAddressSync(
        delistNftMint.publicKey, player1.publicKey, false, TOKEN_PROGRAM_ID
      );
      const [escrowAuth] = PublicKey.findProgramAddressSync(
        [Buffer.from("escrow"), delistNftMint.publicKey.toBuffer()],
        marketProgram.programId
      );
      const escrowNftAta = getAssociatedTokenAddressSync(
        delistNftMint.publicKey, escrowAuth, true, TOKEN_PROGRAM_ID
      );
      const [listingPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("listing"), delistNftMint.publicKey.toBuffer()],
        marketProgram.programId
      );

      try {
        await marketProgram.methods
          .listItem(new anchor.BN(0))
          .accounts({
            seller: player1.publicKey,
            itemMetadata: itemMetadataPda,
            nftMint: delistNftMint.publicKey,
            sellerNftAta: sellerNftAta,
            escrowAuthority: escrowAuth,
            escrowNftAta: escrowNftAta,
            listing: listingPda,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([player1])
          .rpc();
        expect.fail("Should have thrown");
      } catch (err) {
        expect(err.toString()).to.include("InvalidPrice");
      }
    });

    it("player2 buys listed item from player1", async () => {
      if (!listNftMint || !player2NftMint) return;

      const buyerNftAta = getAssociatedTokenAddressSync(
        listNftMint.publicKey, player2.publicKey, false, TOKEN_PROGRAM_ID
      );
      const buyerNftInfo = await provider.connection.getAccountInfo(buyerNftAta);
      if (!buyerNftInfo) {
        const ix = createAssociatedTokenAccountInstruction(
          provider.wallet.publicKey, buyerNftAta, player2.publicKey,
          listNftMint.publicKey, TOKEN_PROGRAM_ID
        );
        await provider.sendAndConfirm(new anchor.web3.Transaction().add(ix));
      }

      const sellerMagicAta = getAssociatedTokenAddressSync(
        magicMintKp.publicKey, player1.publicKey, true, TOKEN_2022_PROGRAM_ID
      );
      const buyerMagicAta = getAssociatedTokenAddressSync(
        magicMintKp.publicKey, player2.publicKey, true, TOKEN_2022_PROGRAM_ID
      );

      const [listingPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("listing"), listNftMint.publicKey.toBuffer()],
        marketProgram.programId
      );
      const [escrowAuth] = PublicKey.findProgramAddressSync(
        [Buffer.from("escrow"), listNftMint.publicKey.toBuffer()],
        marketProgram.programId
      );
      const escrowNftAta = getAssociatedTokenAddressSync(
        listNftMint.publicKey, escrowAuth, true, TOKEN_PROGRAM_ID
      );
      const [itemMetadataPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("item_metadata"), listNftMint.publicKey.toBuffer()],
        inProgram.programId
      );

      const sellerMagicBefore = await provider.connection.getTokenAccountBalance(sellerMagicAta);
      const buyerMagicBefore = await provider.connection.getTokenAccountBalance(buyerMagicAta);

      await marketProgram.methods
        .buyItem()
        .accounts({
          buyer: player2.publicKey,
          callerAuthority: marketCallerAuth,
          listing: listingPda,
          seller: player1.publicKey,
          itemMetadata: itemMetadataPda,
          itemNftConfig: itemNftConfigPda,
          nftMint: listNftMint.publicKey,
          escrowAuthority: escrowAuth,
          escrowNftAta: escrowNftAta,
          buyerNftAta: buyerNftAta,
          magicTokenConfig: magicConfigPda,
          magicTokenMint: magicMintKp.publicKey,
          buyerMagicAta: buyerMagicAta,
          sellerMagicAta: sellerMagicAta,
          itemNftProgram: inProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          token2022Program: TOKEN_2022_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([player2])
        .rpc();

      const buyerNftBalance = await provider.connection.getTokenAccountBalance(buyerNftAta);
      expect(parseInt(buyerNftBalance.value.amount)).to.equal(1);

      const sellerMagicAfter = await provider.connection.getTokenAccountBalance(sellerMagicAta);
      const buyerMagicAfter = await provider.connection.getTokenAccountBalance(buyerMagicAta);
      expect(parseInt(sellerMagicAfter.value.amount)).to.equal(
        parseInt(sellerMagicBefore.value.amount) + 50
      );
      expect(parseInt(buyerMagicAfter.value.amount)).to.equal(
        parseInt(buyerMagicBefore.value.amount) - 50
      );

      const itemMeta = await inProgram.account.itemMetadata.fetch(itemMetadataPda);
      expect(itemMeta.owner.toBase58()).to.equal(player2.publicKey.toBase58());

      try {
        await marketProgram.account.listing.fetch(listingPda);
        expect.fail("Listing should be closed");
      } catch (err) {
        expect(err.toString()).to.include("Account does not exist");
      }
    });

    it("lists an item for delist testing", async () => {
      if (!delistNftMint) return;

      const [itemMetadataPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("item_metadata"), delistNftMint.publicKey.toBuffer()],
        inProgram.programId
      );
      const sellerNftAta = getAssociatedTokenAddressSync(
        delistNftMint.publicKey, player1.publicKey, false, TOKEN_PROGRAM_ID
      );
      const [escrowAuth] = PublicKey.findProgramAddressSync(
        [Buffer.from("escrow"), delistNftMint.publicKey.toBuffer()],
        marketProgram.programId
      );
      const escrowNftAta = getAssociatedTokenAddressSync(
        delistNftMint.publicKey, escrowAuth, true, TOKEN_PROGRAM_ID
      );
      const [listingPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("listing"), delistNftMint.publicKey.toBuffer()],
        marketProgram.programId
      );

      await marketProgram.methods
        .listItem(new anchor.BN(75))
        .accounts({
          seller: player1.publicKey,
          itemMetadata: itemMetadataPda,
          nftMint: delistNftMint.publicKey,
          sellerNftAta: sellerNftAta,
          escrowAuthority: escrowAuth,
          escrowNftAta: escrowNftAta,
          listing: listingPda,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([player1])
        .rpc();

      const escrowBal = await provider.connection.getTokenAccountBalance(escrowNftAta);
      expect(parseInt(escrowBal.value.amount)).to.equal(1);
    });

    it("rejects delist from non-seller", async () => {
      if (!delistNftMint) return;

      const [listingPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("listing"), delistNftMint.publicKey.toBuffer()],
        marketProgram.programId
      );
      const [escrowAuth] = PublicKey.findProgramAddressSync(
        [Buffer.from("escrow"), delistNftMint.publicKey.toBuffer()],
        marketProgram.programId
      );
      const escrowNftAta = getAssociatedTokenAddressSync(
        delistNftMint.publicKey, escrowAuth, true, TOKEN_PROGRAM_ID
      );
      const player2NftAta = getAssociatedTokenAddressSync(
        delistNftMint.publicKey, player2.publicKey, false, TOKEN_PROGRAM_ID
      );

      try {
        await marketProgram.methods
          .delistItem()
          .accounts({
            seller: player2.publicKey,
            listing: listingPda,
            nftMint: delistNftMint.publicKey,
            escrowAuthority: escrowAuth,
            escrowNftAta: escrowNftAta,
            sellerNftAta: player2NftAta,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([player2])
          .rpc();
        expect.fail("Should have thrown");
      } catch (err) {
        const errStr = err.toString();
        expect(
          errStr.includes("NotOwner") || errStr.includes("ConstraintRaw") || errStr.includes("seller")
        ).to.be.true;
      }
    });

    it("delists the item and returns NFT to seller", async () => {
      if (!delistNftMint) return;

      const sellerNftAta = getAssociatedTokenAddressSync(
        delistNftMint.publicKey, player1.publicKey, false, TOKEN_PROGRAM_ID
      );
      const [escrowAuth] = PublicKey.findProgramAddressSync(
        [Buffer.from("escrow"), delistNftMint.publicKey.toBuffer()],
        marketProgram.programId
      );
      const escrowNftAta = getAssociatedTokenAddressSync(
        delistNftMint.publicKey, escrowAuth, true, TOKEN_PROGRAM_ID
      );
      const [listingPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("listing"), delistNftMint.publicKey.toBuffer()],
        marketProgram.programId
      );

      await marketProgram.methods
        .delistItem()
        .accounts({
          seller: player1.publicKey,
          listing: listingPda,
          nftMint: delistNftMint.publicKey,
          escrowAuthority: escrowAuth,
          escrowNftAta: escrowNftAta,
          sellerNftAta: sellerNftAta,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([player1])
        .rpc();

      const sellerBal = await provider.connection.getTokenAccountBalance(sellerNftAta);
      expect(parseInt(sellerBal.value.amount)).to.equal(1);

      try {
        await marketProgram.account.listing.fetch(listingPda);
        expect.fail("Listing should be closed");
      } catch (err) {
        expect(err.toString()).to.include("Account does not exist");
      }
    });
  });
});
