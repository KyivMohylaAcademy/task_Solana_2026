import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import { assert } from "chai";
import { ResourceManager } from "../../../target/types/resource_manager";
import { MagicToken } from "../../../target/types/magic_token";
import { saveAccounts, loadAccounts } from "../../../utils/account_utils";

describe("resource_manager: initialize + magic_token mint", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const rmProgram = anchor.workspace.ResourceManager as Program<ResourceManager>;
  const mtProgram = anchor.workspace.MagicToken as Program<MagicToken>;

  let gameConfigPda: PublicKey;
  let magicTokenMint: Keypair;

  const [magicMintAuth] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("magic_mint_auth")],
    mtProgram.programId
  );

  before(async () => {
    [gameConfigPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("game_config")],
      rmProgram.programId
    );
    magicTokenMint = Keypair.generate();
  });

  it("initializes MagicToken mint", async () => {
    await mtProgram.methods
      .initMagicTokenMint("MagicToken", "MGT", "https://REPLACE_ME/magic-token.json")
      .accounts({
        payer: provider.wallet.publicKey,
        mint: magicTokenMint.publicKey,
        magicMintAuth,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([magicTokenMint])
      .rpc();

    const info = await provider.connection.getAccountInfo(magicTokenMint.publicKey);
    assert.isNotNull(info, "MagicToken mint should exist");
  });

  it("initializes GameConfig with all mint addresses and item prices", async () => {
    const saved = loadAccounts();
    const resourceMints = Array.from({ length: 6 }, (_, i) =>
      new PublicKey(saved[`resourceMint${i}`])
    );
    const itemPrices = [
      new anchor.BN(10),
      new anchor.BN(15),
      new anchor.BN(20),
      new anchor.BN(25),
    ];

    await rmProgram.methods
      .initialize(resourceMints, magicTokenMint.publicKey, itemPrices)
      .accounts({
        admin: provider.wallet.publicKey,
        gameConfig: gameConfigPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const config = await rmProgram.account.gameConfig.fetch(gameConfigPda);
    assert.equal(config.admin.toBase58(), provider.wallet.publicKey.toBase58());
    assert.deepEqual(
      config.itemPrices.map((p) => p.toNumber()),
      [10, 15, 20, 25]
    );
    assert.equal(config.resourceMints[0].toBase58(), resourceMints[0].toBase58());
  });

  it("rejects mint_resource without valid cpi_auth", async () => {
    const saved = loadAccounts();
    const mint0 = new PublicKey(saved["resourceMint0"]);
    const [gameConfig] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("game_config")],
      rmProgram.programId
    );
    const [resourceMintAuth] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("resource_mint_auth")],
      rmProgram.programId
    );
    // Use a fake cpi_auth (just provider wallet) — not a valid PDA from search/crafting.
    const fakeAuth = provider.wallet.publicKey;
    const recipientAta = getAssociatedTokenAddressSync(
      mint0,
      provider.wallet.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID
    );
    try {
      await rmProgram.methods
        .mintResource(0, new anchor.BN(1))
        .accounts({
          cpiAuth: fakeAuth,
          gameConfig,
          mint: mint0,
          recipientAta,
          recipient: provider.wallet.publicKey,
          resourceMintAuth,
          payer: provider.wallet.publicKey,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      assert.fail("Should have rejected unauthorized mint");
    } catch (e: any) {
      assert.include(e.message, "Unauthorized");
    }
  });

  it("mints Stone (resource_id=4) via admin_mint_resource", async () => {
    const saved = loadAccounts();
    const mint4 = new PublicKey(saved["resourceMint4"]);
    const [gameConfig] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("game_config")],
      rmProgram.programId
    );
    const [resourceMintAuth] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("resource_mint_auth")],
      rmProgram.programId
    );
    const recipientAta = getAssociatedTokenAddressSync(
      mint4,
      provider.wallet.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID
    );

    await rmProgram.methods
      .adminMintResource(4, new anchor.BN(5))
      .accounts({
        admin: provider.wallet.publicKey,
        gameConfig,
        mint: mint4,
        recipientAta,
        recipient: provider.wallet.publicKey,
        resourceMintAuth,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const ataInfo = await provider.connection.getTokenAccountBalance(recipientAta);
    assert.equal(ataInfo.value.uiAmount, 5);
  });

  it("saves accounts to accounts.json", async () => {
    saveAccounts({
      gameConfig: gameConfigPda.toBase58(),
      magicTokenMint: magicTokenMint.publicKey.toBase58(),
    });
  });
});
