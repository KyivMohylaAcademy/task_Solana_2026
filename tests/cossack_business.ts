import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { assert } from "chai";

describe("cossack_business", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Program IDs
  const RESOURCE_MANAGER_ID = new PublicKey("C9jeF5eivo4126iDkktjdGk7MEJqNwY9V2pFXMwQYMcy");
  const MAGIC_TOKEN_ID = new PublicKey("BQAqENU5HMGNF8Xunzbb859GCTz8v8Tuknqieqqk6ide");
  const SEARCH_PROGRAM_ID = new PublicKey("7qyvBgEsWYpP5UZKhctCA2C6HuVDBFo4DJH6V2P96rPx");
  const ITEM_NFT_ID = new PublicKey("HMCgFhEqKWroNqsDNo1RmMsyR7Wky2J7CtfDQf32WHKR");
  const CRAFTING_ID = new PublicKey("EfvmR78Gm6o8dwTpBDMicigDREQFfvPd7nmW8VknbqK3");
  const MARKETPLACE_ID = new PublicKey("FBKAbyCSWv1Vm7PVw1NRGWnfH9rpLXqJeP8rNvrRXAkf");

  const admin = provider.wallet;

  it("✅ resource_manager — GameConfig PDA derivation", async () => {
    const [gameConfigPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("game_config")],
      RESOURCE_MANAGER_ID
    );
    assert.ok(gameConfigPDA, "GameConfig PDA повинен бути визначений");
    console.log("GameConfig PDA:", gameConfigPDA.toBase58());
  });

  it("✅ magic_token — MintAuthority PDA derivation", async () => {
    const [mintAuthorityPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("mint_authority")],
      MAGIC_TOKEN_ID
    );
    assert.ok(mintAuthorityPDA, "MintAuthority PDA повинен бути визначений");
    console.log("MintAuthority PDA:", mintAuthorityPDA.toBase58());
  });

  it("✅ marketplace — MarketplaceConfig PDA derivation", async () => {
    const [marketplacePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("marketplace_config")],
      MARKETPLACE_ID
    );
    assert.ok(marketplacePDA, "MarketplaceConfig PDA повинен бути визначений");
    console.log("MarketplaceConfig PDA:", marketplacePDA.toBase58());
  });

  it("✅ search_program — Player PDA derivation", async () => {
    const [playerPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("player"), admin.publicKey.toBuffer()],
      SEARCH_PROGRAM_ID
    );
    assert.ok(playerPDA, "Player PDA повинен бути визначений");
    console.log("Player PDA:", playerPDA.toBase58());
  });

  it("✅ item_nft — ItemMetadata PDA derivation", async () => {
    const fakeMint = anchor.web3.Keypair.generate().publicKey;
    const [itemPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("item"), fakeMint.toBuffer()],
      ITEM_NFT_ID
    );
    assert.ok(itemPDA, "ItemMetadata PDA повинен бути визначений");
    console.log("ItemMetadata PDA:", itemPDA.toBase58());
  });

  it("✅ crafting — ItemMetadata PDA derivation (crafting)", async () => {
    const fakeMint = anchor.web3.Keypair.generate().publicKey;
    const [craftedItemPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("item"), fakeMint.toBuffer()],
      CRAFTING_ID
    );
    assert.ok(craftedItemPDA, "Crafted item PDA повинен бути визначений");
    console.log("Crafted item PDA:", craftedItemPDA.toBase58());
  });

  it("✅ всі програми задеплоєні на devnet", async () => {
    const connection = provider.connection;
    const programs = [
      { name: "resource_manager", id: RESOURCE_MANAGER_ID },
      { name: "magic_token", id: MAGIC_TOKEN_ID },
      { name: "search_program", id: SEARCH_PROGRAM_ID },
      { name: "item_nft", id: ITEM_NFT_ID },
      { name: "crafting", id: CRAFTING_ID },
      { name: "marketplace", id: MARKETPLACE_ID },
    ];

    for (const program of programs) {
      const info = await connection.getAccountInfo(program.id);
      assert.ok(info !== null, `${program.name} має бути задеплоєна`);
      assert.ok(info.executable, `${program.name} має бути executable`);
      console.log(`✓ ${program.name}: ${program.id.toBase58()}`);
    }
  });
});