import * as anchor from "@coral-xyz/anchor";
import { BN } from "@coral-xyz/anchor";
import { expect } from "chai";
import {
  getMetadataPointerState,
  getMint,
  TOKEN_2022_PROGRAM_ID,
} from "@solana/spl-token";
import { Keypair, SystemProgram } from "@solana/web3.js";
import type { SuiteContext } from "../support/context";

export function registerSetupTests(ctx: SuiteContext) {
  const {
    provider,
    payer,
    resourceManager,
    magicToken,
    itemNft,
    gameConfig,
    resourceAuthority,
    itemAuthority,
    magicMint,
    magicAuthority,
    resourceMints,
    resourceDefs,
    invalidResourceId,
    magicTokenDef,
    testItemType,
    expectReject,
    ensureToken2022Ata,
    directMintItemContext,
    fundUser,
  } = ctx;

  it("initializes Token-2022 mints with MetadataPointer", async function () {
    this.timeout(60_000);

    for (const [index] of resourceDefs.entries()) {
      const mintState = await getMint(
        provider.connection,
        resourceMints[index],
        "confirmed",
        TOKEN_2022_PROGRAM_ID,
      );
      expect(mintState.decimals).to.equal(0);

      const pointerState = getMetadataPointerState(mintState);
      expect(pointerState).to.not.equal(null);
      expect(pointerState?.authority?.toBase58()).to.equal(resourceAuthority.toBase58());
      expect(pointerState?.metadataAddress?.toBase58()).to.equal(
        resourceMints[index].toBase58(),
      );
    }

    const magicMintState = await getMint(
      provider.connection,
      magicMint,
      "confirmed",
      TOKEN_2022_PROGRAM_ID,
    );
    expect(magicMintState.decimals).to.equal(0);

    const magicPointer = getMetadataPointerState(magicMintState);
    expect(magicPointer).to.not.equal(null);
    expect(magicPointer?.authority?.toBase58()).to.equal(magicAuthority.toBase58());
    expect(magicPointer?.metadataAddress?.toBase58()).to.equal(magicMint.toBase58());
  });

  it("locks direct resource and MagicToken minting", async function () {
    this.timeout(60_000);

    const walletWoodAta = await ensureToken2022Ata(resourceMints[0], payer.publicKey);
    const walletMagicAta = await ensureToken2022Ata(magicMint, payer.publicKey);

    await expectReject(
      resourceManager.methods
        .mintResource(0, new BN(1))
        .accounts({
          authority: payer.publicKey,
          gameConfig,
          mint: resourceMints[0],
          destination: walletWoodAta,
          mintAuthority: resourceAuthority,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .rpc(),
    );

    await expectReject(
      magicToken.methods
        .mintReward(new BN(1))
        .accounts({
          marketplaceAuthority: payer.publicKey,
          gameConfig,
          magicMint,
          destination: walletMagicAta,
          mintAuthority: magicAuthority,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .rpc(),
    );
  });

  it("rejects setup guard branches for resource and MagicToken mints", async function () {
    this.timeout(60_000);

    const outsider = Keypair.generate();
    await fundUser(outsider, 0.2);

    const invalidResourceMint = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("resource-mint"),
        gameConfig.toBuffer(),
        Buffer.from([invalidResourceId]),
      ],
      resourceManager.programId,
    )[0];
    const payerWoodAta = await ensureToken2022Ata(resourceMints[0], payer.publicKey);

    await expectReject(
      resourceManager.methods
        .initializeResourceMint(
          invalidResourceId,
          "Bogus",
          "BOGUS",
          "https://example.com/resources/bogus.json",
        )
        .accounts({
          admin: payer.publicKey,
          gameConfig,
          mint: invalidResourceMint,
          mintAuthority: resourceAuthority,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc(),
    );

    await expectReject(
      resourceManager.methods
        .initializeResourceMint(
          0,
          resourceDefs[0].name,
          resourceDefs[0].symbol,
          resourceDefs[0].uri,
        )
        .accounts({
          admin: payer.publicKey,
          gameConfig,
          mint: resourceMints[0],
          mintAuthority: resourceAuthority,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc(),
    );

    await expectReject(
      magicToken.methods
        .initializeMagicToken(
          magicTokenDef.name,
          magicTokenDef.symbol,
          magicTokenDef.uri,
        )
        .accounts({
          admin: outsider.publicKey,
          gameConfig,
          magicMint,
          mintAuthority: magicAuthority,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([outsider])
        .rpc(),
    );

    await expectReject(
      magicToken.methods
        .initializeMagicToken(
          magicTokenDef.name,
          magicTokenDef.symbol,
          magicTokenDef.uri,
        )
        .accounts({
          admin: payer.publicKey,
          gameConfig,
          magicMint,
          mintAuthority: magicAuthority,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc(),
    );

    await expectReject(
      resourceManager.methods
        .registerMagicTokenMint()
        .accounts({
          admin: payer.publicKey,
          gameConfig,
          magicTokenMint: magicMint,
        })
        .rpc(),
    );

    await expectReject(
      resourceManager.methods
        .mintResource(invalidResourceId, new BN(1))
        .accounts({
          authority: payer.publicKey,
          gameConfig,
          mint: resourceMints[0],
          destination: payerWoodAta,
          mintAuthority: resourceAuthority,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .rpc(),
    );

    await expectReject(
      resourceManager.methods
        .mintResource(0, new BN(1))
        .accounts({
          authority: payer.publicKey,
          gameConfig,
          mint: resourceMints[1],
          destination: payerWoodAta,
          mintAuthority: resourceAuthority,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .rpc(),
    );

    await expectReject(
      magicToken.methods
        .mintReward(new BN(1))
        .accounts({
          marketplaceAuthority: payer.publicKey,
          gameConfig,
          magicMint: resourceMints[0],
          destination: payerWoodAta,
          mintAuthority: magicAuthority,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .rpc(),
    );
  });

  it("rejects direct item minting outside crafting", async function () {
    this.timeout(60_000);

    const player = Keypair.generate();
    const fakeCraftingAuthority = Keypair.generate();
    await fundUser(player, 0.5);
    await fundUser(fakeCraftingAuthority, 0.1);

    const invalidMintSeed = Array.from(
      anchor.web3.Keypair.generate().secretKey.slice(0, 32),
    );
    const invalidMintContext = directMintItemContext(
      player.publicKey,
      invalidMintSeed,
      fakeCraftingAuthority.publicKey,
    );

    await expectReject(
      itemNft.methods
        .mintItem(255, invalidMintSeed, "https://example.com/items/invalid.json")
        .accounts(invalidMintContext.accounts)
        .signers([player, fakeCraftingAuthority])
        .rpc(),
    );

    const unauthorizedMintSeed = Array.from(
      anchor.web3.Keypair.generate().secretKey.slice(0, 32),
    );
    const unauthorizedMintContext = directMintItemContext(
      player.publicKey,
      unauthorizedMintSeed,
      fakeCraftingAuthority.publicKey,
    );

    await expectReject(
      itemNft.methods
        .mintItem(
          testItemType,
          unauthorizedMintSeed,
          "https://example.com/items/saber.json",
        )
        .accounts(unauthorizedMintContext.accounts)
        .signers([player, fakeCraftingAuthority])
        .rpc(),
    );
  });
}
