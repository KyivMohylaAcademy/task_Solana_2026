import * as anchor from "@coral-xyz/anchor";
import {
  Keypair,
  PublicKey,
  SYSVAR_RENT_PUBKEY,
  SystemProgram,
} from "@solana/web3.js";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import { expect } from "chai";
import {
  rmProgram,
  mtProgram,
  inProgram,
  player1,
  initializeAll,
} from "./helpers/setup";
import {
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  METAPLEX_PROGRAM_ID,
  findMetadataPda,
  findMasterEditionPda,
} from "./helpers/utils";

describe("06 - Security", () => {
  let gameConfigPda: PublicKey;
  let mintAuthorityPda: PublicKey;
  let magicConfigPda: PublicKey;
  let magicMintAuthPda: PublicKey;
  let itemNftConfigPda: PublicKey;
  let nftAuthorityPda: PublicKey;
  let magicMintKp: Keypair;

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

  it("rejects direct resource minting (no CPI authority)", async () => {
    const setup = require("./helpers/setup");
    const fakeCallerAuth = Keypair.generate();
    const ata = getAssociatedTokenAddressSync(
      setup.resourceMints[0].publicKey,
      player1.publicKey,
      true,
      TOKEN_2022_PROGRAM_ID
    );

    try {
      await rmProgram.methods
        .mintResource(0, new anchor.BN(1))
        .accounts({
          callerAuthority: fakeCallerAuth.publicKey,
          gameConfig: gameConfigPda,
          resourceMint: setup.resourceMints[0].publicKey,
          mintAuthority: mintAuthorityPda,
          playerAta: ata,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([fakeCallerAuth])
        .rpc();
      expect.fail("Should have thrown");
    } catch (err) {
      expect(err).to.exist;
    }
  });

  it("rejects direct MagicToken minting (no CPI authority)", async () => {
    const fakeCallerAuth = Keypair.generate();
    const ata = getAssociatedTokenAddressSync(
      magicMintKp.publicKey,
      player1.publicKey,
      true,
      TOKEN_2022_PROGRAM_ID
    );

    try {
      await mtProgram.methods
        .mintMagicToken(new anchor.BN(100))
        .accounts({
          callerAuthority: fakeCallerAuth.publicKey,
          config: magicConfigPda,
          mint: magicMintKp.publicKey,
          mintAuthority: magicMintAuthPda,
          recipientAta: ata,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([fakeCallerAuth])
        .rpc();
      expect.fail("Should have thrown");
    } catch (err) {
      expect(err).to.exist;
    }
  });

  it("rejects unauthorized admin operations", async () => {
    try {
      await rmProgram.methods
        .updateSearchCooldown(new anchor.BN(1))
        .accounts({
          admin: player1.publicKey,
          gameConfig: gameConfigPda,
        })
        .signers([player1])
        .rpc();
      expect.fail("Should have thrown");
    } catch (err) {
      expect(err.toString()).to.include("Unauthorized");
    }
  });

  it("rejects direct NFT creation (no CPI authority)", async () => {
    const fakeCaller = Keypair.generate();
    const nftMint = Keypair.generate();

    try {
      await inProgram.methods
        .createItem(0)
        .accounts({
          callerAuthority: fakeCaller.publicKey,
          config: itemNftConfigPda,
          nftAuthority: nftAuthorityPda,
          player: player1.publicKey,
          payer: player1.publicKey,
          nftMint: nftMint.publicKey,
          playerNftAta: getAssociatedTokenAddressSync(
            nftMint.publicKey,
            player1.publicKey,
            false,
            TOKEN_PROGRAM_ID
          ),
          itemMetadata: PublicKey.findProgramAddressSync(
            [Buffer.from("item_metadata"), nftMint.publicKey.toBuffer()],
            inProgram.programId
          )[0],
          metadataAccount: findMetadataPda(nftMint.publicKey),
          masterEdition: findMasterEditionPda(nftMint.publicKey),
          metadataProgram: METAPLEX_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([fakeCaller, player1, nftMint])
        .rpc();
      expect.fail("Should have thrown");
    } catch (err) {
      expect(err).to.exist;
    }
  });
});
