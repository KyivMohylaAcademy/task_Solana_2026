import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  createBurnInstruction,
  getAssociatedTokenAddressSync,
  getMint,
} from "@solana/spl-token";
import {
  ComputeBudgetProgram,
  SYSVAR_RENT_PUBKEY,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import type { SuiteContext } from "../support/context";

export function registerE2ETests(ctx: SuiteContext) {
  const {
    provider,
    itemNft,
    resourceManager,
    magicToken,
    searchProgram,
    crafting,
    marketplace,
    gameConfig,
    resourceAuthority,
    craftingAuthority,
    marketplaceAuthority,
    itemAuthority,
    magicMint,
    magicAuthority,
    metadataProgramId,
    itemPrices,
    testItemType,
    testRecipe,
    playerAddress,
    resourceTokenAccounts,
    searchAccounts,
    recipeRemainingAccounts,
    deriveItemAddresses,
    createLegacyMintForTests,
    createLegacyTokenAccountForOwner,
    waitForBalances,
    waitForStableBalances,
    waitForTokenBalance,
    expectZeroOrClosedTokenAccount,
    sweepCollectorToCrafter,
    hasRecipe,
    expectReject,
    ensureAllResourceAtas,
    fundUser,
  } = ctx;

  it("crafts, transfers, and sells a saber NFT for MagicToken", async function () {
    this.timeout(240_000);

    const crafter = anchor.web3.Keypair.generate();
    await fundUser(crafter, 0.8);
    const buyer = anchor.web3.Keypair.generate();
    await fundUser(buyer, 0.6);
    const outsider = anchor.web3.Keypair.generate();
    await fundUser(outsider, 0.2);

    const crafterResourceAccounts = resourceTokenAccounts(crafter.publicKey);
    await ensureAllResourceAtas(crafter.publicKey);

    let attempts = 0;
    while (!(await hasRecipe(crafterResourceAccounts, testRecipe)) && attempts < 24) {
      const collector = anchor.web3.Keypair.generate();
      attempts += 1;
      await fundUser(collector, 0.4);
      await ensureAllResourceAtas(collector.publicKey);
      const [collectorPlayer] = playerAddress(collector.publicKey);
      await searchProgram.methods
        .initializePlayer()
        .accounts({
          owner: collector.publicKey,
          player: collectorPlayer,
          systemProgram: SystemProgram.programId,
        })
        .signers([collector])
        .rpc();
      await searchProgram.methods
        .searchResources()
        .preInstructions([
          ComputeBudgetProgram.setComputeUnitLimit({ units: 500_000 }),
        ])
        .accounts(searchAccounts(collector.publicKey, collectorPlayer))
        .signers([collector])
        .rpc();
      await sweepCollectorToCrafter(collector, crafter.publicKey);
    }

    const beforeCraftBalances = await waitForStableBalances(
      crafterResourceAccounts,
      (balances) => testRecipe.every((needed, index) => balances[index] >= needed),
    );
    for (const [index, needed] of testRecipe.entries()) {
      expect(beforeCraftBalances[index]).to.be.at.least(
        needed,
        `resource ${index} balance before craft`,
      );
    }

    const mintSeed = Array.from(anchor.web3.Keypair.generate().secretKey.slice(0, 32));
    const {
      itemMint,
      itemMetadata,
      ownerItemAta: crafterItemAta,
      metadataPda,
      masterEditionPda,
    } = deriveItemAddresses(crafter.publicKey, mintSeed);

    await crafting.methods
      .craftItem(
        testItemType,
        mintSeed,
        "https://example.com/items/saber.json",
      )
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({ units: 500_000 }),
      ])
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
      .remainingAccounts(recipeRemainingAccounts(crafter.publicKey, testRecipe))
      .signers([crafter])
      .rpc();

    const nftAccount = await waitForTokenBalance(crafterItemAta, 1);
    expect(Number(nftAccount.amount)).to.equal(1);
    await expectReject(
      provider.sendAndConfirm(
        new Transaction().add(
          createBurnInstruction(crafterItemAta, itemMint, crafter.publicKey, 1),
        ),
        [crafter],
      ),
    );

    const buyerItemAta = getAssociatedTokenAddressSync(itemMint, buyer.publicKey);
    const buyerSameMintAccount = await createLegacyTokenAccountForOwner(
      itemMint,
      buyer.publicKey,
    );
    const crafterEmptySameMintAccount = await createLegacyTokenAccountForOwner(
      itemMint,
      crafter.publicKey,
    );
    const wrongMint = await createLegacyMintForTests();
    const crafterWrongMintAccount = await createLegacyTokenAccountForOwner(
      wrongMint,
      crafter.publicKey,
    );
    const buyerWrongMintAccount = await createLegacyTokenAccountForOwner(
      wrongMint,
      buyer.publicKey,
    );
    const staleSellerMagicAta = getAssociatedTokenAddressSync(
      magicMint,
      crafter.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
    );

    await expectReject(
      itemNft.methods
        .transferItem()
        .accounts({
          owner: crafter.publicKey,
          recipient: buyer.publicKey,
          gameConfig,
          itemAuthority,
          itemMint,
          ownerItemAccount: buyerSameMintAccount,
          recipientItemAccount: buyerSameMintAccount,
          itemMetadata,
          metadata: metadataPda,
          masterEdition: masterEditionPda,
          metadataProgram: metadataProgramId,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([crafter])
        .rpc(),
    );

    await expectReject(
      itemNft.methods
        .transferItem()
        .accounts({
          owner: crafter.publicKey,
          recipient: buyer.publicKey,
          gameConfig,
          itemAuthority,
          itemMint,
          ownerItemAccount: crafterWrongMintAccount,
          recipientItemAccount: buyerSameMintAccount,
          itemMetadata,
          metadata: metadataPda,
          masterEdition: masterEditionPda,
          metadataProgram: metadataProgramId,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([crafter, buyer])
        .rpc(),
    );

    await expectReject(
      itemNft.methods
        .transferItem()
        .accounts({
          owner: crafter.publicKey,
          recipient: buyer.publicKey,
          gameConfig,
          itemAuthority,
          itemMint,
          ownerItemAccount: crafterEmptySameMintAccount,
          recipientItemAccount: buyerSameMintAccount,
          itemMetadata,
          metadata: metadataPda,
          masterEdition: masterEditionPda,
          metadataProgram: metadataProgramId,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([crafter, buyer])
        .rpc(),
    );

    await expectReject(
      marketplace.methods
        .sellItem()
        .preInstructions([
          ComputeBudgetProgram.setComputeUnitLimit({ units: 500_000 }),
        ])
        .accounts({
          seller: crafter.publicKey,
          gameConfig,
          marketplaceAuthority,
          itemNftProgram: itemNft.programId,
          itemAuthority,
          itemMint,
          sellerItemAccount: crafterEmptySameMintAccount,
          itemMetadata,
          metadata: metadataPda,
          masterEdition: masterEditionPda,
          metadataProgram: metadataProgramId,
          nftTokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          magicTokenProgram: magicToken.programId,
          magicMint,
          magicAuthority,
          sellerMagicAccount: staleSellerMagicAta,
          token2022Program: TOKEN_2022_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([crafter])
        .rpc(),
    );

    await expectReject(
      itemNft.methods
        .transferItem()
        .accounts({
          owner: crafter.publicKey,
          recipient: crafter.publicKey,
          gameConfig,
          itemAuthority,
          itemMint,
          ownerItemAccount: crafterItemAta,
          recipientItemAccount: crafterItemAta,
          itemMetadata,
          metadata: metadataPda,
          masterEdition: masterEditionPda,
          metadataProgram: metadataProgramId,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([crafter])
        .rpc(),
    );

    await itemNft.methods
      .prepareItemReceive()
      .accounts({
        recipient: buyer.publicKey,
        payer: crafter.publicKey,
        gameConfig,
        itemAuthority,
        itemMint,
        recipientItemAccount: buyerItemAta,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([buyer, crafter])
      .rpc();

    await itemNft.methods
      .transferItem()
      .accounts({
        owner: crafter.publicKey,
        recipient: buyer.publicKey,
        gameConfig,
        itemAuthority,
        itemMint,
        ownerItemAccount: crafterItemAta,
        recipientItemAccount: buyerItemAta,
        itemMetadata,
        metadata: metadataPda,
        masterEdition: masterEditionPda,
        metadataProgram: metadataProgramId,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([crafter])
      .rpc();

    const buyerNftAccount = await waitForTokenBalance(buyerItemAta, 1);
    expect(Number(buyerNftAccount.amount)).to.equal(1);
    const crafterNftAccount = await waitForTokenBalance(crafterItemAta, 0);
    expect(Number(crafterNftAccount.amount)).to.equal(0);

    const transferredItemMetadata = await itemNft.account.itemMetadata.fetch(itemMetadata);
    expect(transferredItemMetadata.owner.toBase58()).to.equal(buyer.publicKey.toBase58());

    await expectReject(
      provider.sendAndConfirm(
        new Transaction().add(
          createBurnInstruction(buyerItemAta, itemMint, buyer.publicKey, 1),
        ),
        [buyer],
      ),
    );

    await expectReject(
      marketplace.methods
        .sellItem()
        .preInstructions([
          ComputeBudgetProgram.setComputeUnitLimit({ units: 500_000 }),
        ])
        .accounts({
          seller: crafter.publicKey,
          gameConfig,
          marketplaceAuthority,
          itemNftProgram: itemNft.programId,
          itemAuthority,
          itemMint,
          sellerItemAccount: crafterItemAta,
          itemMetadata,
          metadata: metadataPda,
          masterEdition: masterEditionPda,
          metadataProgram: metadataProgramId,
          nftTokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          magicTokenProgram: magicToken.programId,
          magicMint,
          magicAuthority,
          sellerMagicAccount: staleSellerMagicAta,
          token2022Program: TOKEN_2022_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([crafter])
        .rpc(),
    );

    const afterCraftBalances = await waitForStableBalances(
      crafterResourceAccounts,
      (balances) =>
        testRecipe.every(
          (needed, index) => balances[index] === beforeCraftBalances[index] - needed,
        ),
    );
    for (const [index, needed] of testRecipe.entries()) {
      expect(
        afterCraftBalances[index],
        `resource ${index}: before=${beforeCraftBalances[index]}, need=${needed}`,
      ).to.equal(beforeCraftBalances[index] - needed);
    }

    const sellerMagicAta = getAssociatedTokenAddressSync(
      magicMint,
      buyer.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
    );

    await marketplace.methods
      .sellItem()
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({ units: 500_000 }),
      ])
      .accounts({
        seller: buyer.publicKey,
        gameConfig,
        marketplaceAuthority,
        itemNftProgram: itemNft.programId,
        itemAuthority,
        itemMint,
        sellerItemAccount: buyerItemAta,
        itemMetadata,
        metadata: metadataPda,
        masterEdition: masterEditionPda,
        metadataProgram: metadataProgramId,
        nftTokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        magicTokenProgram: magicToken.programId,
        magicMint,
        magicAuthority,
        sellerMagicAccount: sellerMagicAta,
        token2022Program: TOKEN_2022_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([buyer])
      .rpc();

    const sellerMagic = await waitForTokenBalance(
      sellerMagicAta,
      itemPrices[testItemType],
      TOKEN_2022_PROGRAM_ID,
    );
    expect(Number(sellerMagic.amount)).to.equal(itemPrices[testItemType]);

    await expectReject(itemNft.account.itemMetadata.fetch(itemMetadata));
    await expectZeroOrClosedTokenAccount(buyerItemAta);
    const magicMintState = await getMint(
      provider.connection,
      magicMint,
      undefined,
      TOKEN_2022_PROGRAM_ID,
    );
    expect(Number(magicMintState.supply)).to.equal(itemPrices[testItemType]);
  });
}
