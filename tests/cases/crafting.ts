import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
} from "@solana/spl-token";
import { SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import type { SuiteContext } from "../support/context";

export function registerCraftingTests(ctx: SuiteContext) {
  const {
    itemNft,
    crafting,
    resourceManager,
    gameConfig,
    craftingAuthority,
    resourceAuthority,
    itemAuthority,
    metadataProgramId,
    testItemType,
    deriveItemAddresses,
    recipeRemainingAccounts,
    expectReject,
    fundUser,
  } = ctx;

  it("rejects invalid craft requests before minting anything", async function () {
    this.timeout(60_000);

    const crafter = anchor.web3.Keypair.generate();
    await fundUser(crafter, 0.4);

    const mintSeed = Array.from(anchor.web3.Keypair.generate().secretKey.slice(0, 32));
    const {
      itemMint,
      itemMetadata,
      ownerItemAta: crafterItemAta,
      metadataPda,
      masterEditionPda,
    } = deriveItemAddresses(crafter.publicKey, mintSeed);

    await expectReject(
      crafting.methods
        .craftItem(255, mintSeed, "https://example.com/items/invalid.json")
        .accounts({
          player: crafter.publicKey,
          gameConfig,
          craftingAuthority,
          resourceManagerProgram: resourceManager.programId,
          resourceAuthority,
          itemNftProgram: itemNft.programId,
          itemAuthority,
          itemMint,
          playerItemAccount: crafterItemAta,
          itemMetadata,
          metadata: metadataPda,
          masterEdition: masterEditionPda,
          metadataProgram: metadataProgramId,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          token2022Program: TOKEN_2022_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([crafter])
        .rpc(),
    );

    await expectReject(
      crafting.methods
        .craftItem(
          testItemType,
          mintSeed,
          "https://example.com/items/missing-remaining-accounts.json",
        )
        .accounts({
          player: crafter.publicKey,
          gameConfig,
          craftingAuthority,
          resourceManagerProgram: resourceManager.programId,
          resourceAuthority,
          itemNftProgram: itemNft.programId,
          itemAuthority,
          itemMint,
          playerItemAccount: crafterItemAta,
          itemMetadata,
          metadata: metadataPda,
          masterEdition: masterEditionPda,
          metadataProgram: metadataProgramId,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          token2022Program: TOKEN_2022_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([crafter])
        .rpc(),
    );

    await expectReject(
      crafting.methods
        .craftItem(
          testItemType,
          mintSeed,
          "https://example.com/items/insufficient-resources.json",
        )
        .accounts({
          player: crafter.publicKey,
          gameConfig,
          craftingAuthority,
          resourceManagerProgram: resourceManager.programId,
          resourceAuthority,
          itemNftProgram: itemNft.programId,
          itemAuthority,
          itemMint,
          playerItemAccount: crafterItemAta,
          itemMetadata,
          metadata: metadataPda,
          masterEdition: masterEditionPda,
          metadataProgram: metadataProgramId,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          token2022Program: TOKEN_2022_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .remainingAccounts(recipeRemainingAccounts(crafter.publicKey, ctx.testRecipe))
        .signers([crafter])
        .rpc(),
    );

    expect(await ctx.provider.connection.getAccountInfo(itemMint)).to.equal(null);
    await expectReject(itemNft.account.itemMetadata.fetch(itemMetadata));
  });
}
