import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey, SYSVAR_RENT_PUBKEY, ComputeBudgetProgram, SystemProgram } from "@solana/web3.js";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import { expect } from "chai";
import {
  provider,
  rmProgram,
  inProgram,
  craftProgram,
  player1,
  resourceMints,
  craftCallerAuth,
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
} from "./helpers/utils";

describe("04 - Crafting", () => {
  let gameConfigPda: PublicKey;
  let mintAuthorityPda: PublicKey;
  let itemNftConfigPda: PublicKey;
  let nftAuthorityPda: PublicKey;

  before(async () => {
    await initializeAll();
    const setup = require("./helpers/setup");
    gameConfigPda = setup.gameConfigPda;
    mintAuthorityPda = setup.mintAuthorityPda;
    itemNftConfigPda = setup.itemNftConfigPda;
    nftAuthorityPda = setup.nftAuthorityPda;

    await doMultipleSearches(player1, 10, gameConfigPda, mintAuthorityPda);
  });

  it("crafts an item when player has enough resources", async () => {
    const balances: number[] = [];
    for (let i = 0; i < 6; i++) {
      const ata = getAssociatedTokenAddressSync(
        resourceMints[i].publicKey,
        player1.publicKey,
        true,
        TOKEN_2022_PROGRAM_ID
      );
      const info = await provider.connection.getTokenAccountBalance(ata);
      balances.push(parseInt(info.value.amount));
    }
    console.log("    Player1 resource balances:", balances);

    let affordableRecipe = -1;
    for (let r = 0; r < RECIPES.length; r++) {
      let canAfford = true;
      for (let i = 0; i < 6; i++) {
        if (balances[i] < RECIPES[r][i]) { canAfford = false; break; }
      }
      if (canAfford) { affordableRecipe = r; break; }
    }

    if (affordableRecipe === -1) {
      console.log("    Skipping craft test - not enough resources from random searches");
      return;
    }

    console.log("    Crafting item type:", affordableRecipe);

    const nftMintKp = Keypair.generate();
    const neededResourceIds: number[] = [];
    for (let i = 0; i < 6; i++) {
      if (RECIPES[affordableRecipe][i] > 0) neededResourceIds.push(i);
    }

    const resourceRemaining = neededResourceIds.flatMap((id) => {
      const mint = resourceMints[id].publicKey;
      const ata = getAssociatedTokenAddressSync(
        mint, player1.publicKey, true, TOKEN_2022_PROGRAM_ID
      );
      return [
        { pubkey: mint, isSigner: false, isWritable: true },
        { pubkey: ata, isSigner: false, isWritable: true },
      ];
    });

    const [itemMetadataPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("item_metadata"), nftMintKp.publicKey.toBuffer()],
      inProgram.programId
    );

    const playerNftAta = getAssociatedTokenAddressSync(
      nftMintKp.publicKey, player1.publicKey, false, TOKEN_PROGRAM_ID
    );
    const metadataAccount = findMetadataPda(nftMintKp.publicKey);
    const masterEdition = findMasterEditionPda(nftMintKp.publicKey);

    const nftRemaining = [
      { pubkey: nftMintKp.publicKey, isSigner: true, isWritable: true },
      { pubkey: playerNftAta, isSigner: false, isWritable: true },
      { pubkey: metadataAccount, isSigner: false, isWritable: true },
      { pubkey: masterEdition, isSigner: false, isWritable: true },
      { pubkey: METAPLEX_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: itemMetadataPda, isSigner: false, isWritable: true },
      { pubkey: itemNftConfigPda, isSigner: false, isWritable: false },
      { pubkey: nftAuthorityPda, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
    ];

    await craftProgram.methods
      .craftItem(affordableRecipe, Buffer.from(neededResourceIds))
      .accounts({
        player: player1.publicKey,
        callerAuthority: craftCallerAuth,
        gameConfig: gameConfigPda,
        resourceManagerProgram: rmProgram.programId,
        itemNftProgram: inProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        token2022Program: TOKEN_2022_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .remainingAccounts([...resourceRemaining, ...nftRemaining])
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 }),
      ])
      .signers([player1, nftMintKp])
      .rpc();

    const nftBalance = await provider.connection.getTokenAccountBalance(playerNftAta);
    expect(parseInt(nftBalance.value.amount)).to.equal(1);

    const newBalances: number[] = [];
    for (let i = 0; i < 6; i++) {
      const ata = getAssociatedTokenAddressSync(
        resourceMints[i].publicKey,
        player1.publicKey,
        true,
        TOKEN_2022_PROGRAM_ID
      );
      const info = await provider.connection.getTokenAccountBalance(ata);
      newBalances.push(parseInt(info.value.amount));
    }
    for (let i = 0; i < 6; i++) {
      expect(newBalances[i]).to.equal(balances[i] - RECIPES[affordableRecipe][i]);
    }
  });
});
