import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  createAssociatedTokenAccountInstruction,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { expect } from "chai";
import { ResourceManager } from "../target/types/resource_manager";
import { MagicToken } from "../target/types/magic_token";
import { ItemNft } from "../target/types/item_nft";
import { Search } from "../target/types/search";
import { Crafting } from "../target/types/crafting";
import { Marketplace } from "../target/types/marketplace";

const TOKEN_METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

describe("Козацький бізнес", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const resourceManager = anchor.workspace
    .ResourceManager as Program<ResourceManager>;
  const magicToken = anchor.workspace.MagicToken as Program<MagicToken>;
  const itemNft = anchor.workspace.ItemNft as Program<ItemNft>;
  const search = anchor.workspace.Search as Program<Search>;
  const crafting = anchor.workspace.Crafting as Program<Crafting>;
  const marketplace = anchor.workspace.Marketplace as Program<Marketplace>;

  const admin = provider.wallet;

  let gameConfigPda: PublicKey;
  let mintAuthorityPda: PublicKey;
  let magicTokenConfigPda: PublicKey;
  let magicTokenMintPda: PublicKey;
  let magicMintAuthorityPda: PublicKey;
  let playerPda: PublicKey;

  const resourceMints: PublicKey[] = [];

  const playerResourceAccounts: PublicKey[] = [];

  before(async () => {
    [gameConfigPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("game_config")],
      resourceManager.programId
    );
    [mintAuthorityPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("mint_authority")],
      resourceManager.programId
    );
    [magicTokenConfigPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("magic_token_config")],
      magicToken.programId
    );
    [magicTokenMintPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("magic_token_mint")],
      magicToken.programId
    );
    [magicMintAuthorityPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("magic_mint_authority")],
      magicToken.programId
    );
    [playerPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("player"), admin.publicKey.toBuffer()],
      search.programId
    );

    for (let i = 0; i < 6; i++) {
      const [mintPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("resource_mint"), Buffer.from([i])],
        resourceManager.programId
      );
      resourceMints.push(mintPda);
    }
  });

  describe("resource_manager", () => {
    it("Initializes game config", async () => {
      const tx = await resourceManager.methods
        .initialize()
        .accounts({
          gameConfig: gameConfigPda,
          admin: admin.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const config = await resourceManager.account.gameConfig.fetch(
        gameConfigPda
      );
      expect(config.admin.toBase58()).to.equal(admin.publicKey.toBase58());
    });

    it("Creates all 6 resource mints", async () => {
      for (let i = 0; i < 6; i++) {
        await resourceManager.methods
          .createResourceMint(i)
          .accounts({
            gameConfig: gameConfigPda,
            resourceMint: resourceMints[i],
            mintAuthority: mintAuthorityPda,
            admin: admin.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_2022_PROGRAM_ID,
          })
          .rpc();
      }

      const config = await resourceManager.account.gameConfig.fetch(
        gameConfigPda
      );
      for (let i = 0; i < 6; i++) {
        expect(config.resourceMints[i].toBase58()).to.equal(
          resourceMints[i].toBase58()
        );
      }
    });

    it("Fails to create resource mint with invalid ID", async () => {
      try {
        const [fakeMint] = PublicKey.findProgramAddressSync(
          [Buffer.from("resource_mint"), Buffer.from([6])],
          resourceManager.programId
        );
        await resourceManager.methods
          .createResourceMint(6)
          .accounts({
            gameConfig: gameConfigPda,
            resourceMint: fakeMint,
            mintAuthority: mintAuthorityPda,
            admin: admin.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_2022_PROGRAM_ID,
          })
          .rpc();
        expect.fail("Should have thrown");
      } catch (e: any) {
        expect(e.error?.errorCode?.code || e.message).to.contain(
          "InvalidResourceId"
        );
      }
    });

    it("Mints resources to player token accounts", async () => {
      for (let i = 0; i < 6; i++) {
        const ata = getAssociatedTokenAddressSync(
          resourceMints[i],
          admin.publicKey,
          false,
          TOKEN_2022_PROGRAM_ID
        );
        playerResourceAccounts.push(ata);

        const ix = createAssociatedTokenAccountInstruction(
          admin.publicKey,
          ata,
          admin.publicKey,
          resourceMints[i],
          TOKEN_2022_PROGRAM_ID
        );
        const tx = new anchor.web3.Transaction().add(ix);
        await provider.sendAndConfirm(tx);
      }

      for (let i = 0; i < 6; i++) {
        await resourceManager.methods
          .mintResource(i, new anchor.BN(10))
          .accounts({
            gameConfig: gameConfigPda,
            resourceMint: resourceMints[i],
            mintAuthority: mintAuthorityPda,
            playerTokenAccount: playerResourceAccounts[i],
            tokenProgram: TOKEN_2022_PROGRAM_ID,
          })
          .rpc();
      }
    });

    it("Burns resources from player token accounts", async () => {
      await resourceManager.methods
        .burnResource(0, new anchor.BN(2))
        .accounts({
          resourceMint: resourceMints[0],
          playerTokenAccount: playerResourceAccounts[0],
          player: admin.publicKey,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .rpc();
    });
  });

  describe("magic_token", () => {
    it("Initializes MagicToken mint", async () => {
      const tx = await magicToken.methods
        .initialize()
        .accounts({
          magicTokenConfig: magicTokenConfigPda,
          magicTokenMint: magicTokenMintPda,
          mintAuthority: magicMintAuthorityPda,
          admin: admin.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .rpc();

      const config = await magicToken.account.magicTokenConfig.fetch(
        magicTokenConfigPda
      );
      expect(config.mint.toBase58()).to.equal(magicTokenMintPda.toBase58());
      expect(config.admin.toBase58()).to.equal(admin.publicKey.toBase58());
    });

    it("Mints MagicToken (direct call for testing)", async () => {
      const magicAta = getAssociatedTokenAddressSync(
        magicTokenMintPda,
        admin.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      const createAtaIx = createAssociatedTokenAccountInstruction(
        admin.publicKey,
        magicAta,
        admin.publicKey,
        magicTokenMintPda,
        TOKEN_2022_PROGRAM_ID
      );
      const tx = new anchor.web3.Transaction().add(createAtaIx);
      await provider.sendAndConfirm(tx);

      await magicToken.methods
        .mintMagicToken(new anchor.BN(100))
        .accounts({
          magicTokenConfig: magicTokenConfigPda,
          magicTokenMint: magicTokenMintPda,
          mintAuthority: magicMintAuthorityPda,
          playerTokenAccount: magicAta,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .rpc();
    });
  });

  describe("search", () => {
    it("Registers a player", async () => {
      const tx = await search.methods
        .registerPlayer()
        .accounts({
          playerAccount: playerPda,
          owner: admin.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const player = await search.account.player.fetch(playerPda);
      expect(player.owner.toBase58()).to.equal(admin.publicKey.toBase58());
      expect(player.lastSearchTimestamp.toNumber()).to.equal(0);
    });

    it("Fails to register same player twice", async () => {
      try {
        await search.methods
          .registerPlayer()
          .accounts({
            playerAccount: playerPda,
            owner: admin.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
        expect.fail("Should have thrown");
      } catch (e: any) {
        expect(e.message).to.exist;
      }
    });

    it("Searches for resources (mints 3 random resources)", async () => {
      const remainingAccounts = [];
      for (let i = 0; i < 6; i++) {
        remainingAccounts.push({
          pubkey: resourceMints[i],
          isSigner: false,
          isWritable: true,
        });
      }
      for (let i = 0; i < 6; i++) {
        remainingAccounts.push({
          pubkey: playerResourceAccounts[i],
          isSigner: false,
          isWritable: true,
        });
      }

      const tx = await search.methods
        .searchResources()
        .accounts({
          playerAccount: playerPda,
          owner: admin.publicKey,
          gameConfig: gameConfigPda,
          mintAuthority: mintAuthorityPda,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          resourceManagerProgram: resourceManager.programId,
        })
        .remainingAccounts(remainingAccounts)
        .rpc();
    });

    it("Fails search when cooldown not expired", async () => {
      const remainingAccounts = [];
      for (let i = 0; i < 6; i++) {
        remainingAccounts.push({
          pubkey: resourceMints[i],
          isSigner: false,
          isWritable: true,
        });
      }
      for (let i = 0; i < 6; i++) {
        remainingAccounts.push({
          pubkey: playerResourceAccounts[i],
          isSigner: false,
          isWritable: true,
        });
      }

      try {
        await search.methods
          .searchResources()
          .accounts({
            playerAccount: playerPda,
            owner: admin.publicKey,
            gameConfig: gameConfigPda,
            mintAuthority: mintAuthorityPda,
            tokenProgram: TOKEN_2022_PROGRAM_ID,
            resourceManagerProgram: resourceManager.programId,
          })
          .remainingAccounts(remainingAccounts)
          .rpc();
        expect.fail("Should have thrown - cooldown not expired");
      } catch (e: any) {
        expect(e.error?.errorCode?.code || e.message).to.contain(
          "CooldownNotExpired"
        );
      }
    });
  });

  describe("item_nft", () => {
    let itemMintKeypair: Keypair;
    let itemAta: PublicKey;
    let itemMetadataPda: PublicKey;
    let metaplexMetadataPda: PublicKey;

    it("Creates an NFT item (Шабля козака)", async () => {
      itemMintKeypair = Keypair.generate();

      [itemMetadataPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("item_metadata"),
          itemMintKeypair.publicKey.toBuffer(),
        ],
        itemNft.programId
      );

      itemAta = getAssociatedTokenAddressSync(
        itemMintKeypair.publicKey,
        admin.publicKey,
        false,
        TOKEN_PROGRAM_ID
      );

      [metaplexMetadataPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("metadata"),
          TOKEN_METADATA_PROGRAM_ID.toBuffer(),
          itemMintKeypair.publicKey.toBuffer(),
        ],
        TOKEN_METADATA_PROGRAM_ID
      );

      const tx = await itemNft.methods
        .createItem(0)
        .accounts({
          itemMetadata: itemMetadataPda,
          itemMint: itemMintKeypair.publicKey,
          playerItemAccount: itemAta,
          metadataAccount: metaplexMetadataPda,
          player: admin.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          metadataProgram: TOKEN_METADATA_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([itemMintKeypair])
        .rpc();

      const metadata = await itemNft.account.itemMetadata.fetch(
        itemMetadataPda
      );
      expect(metadata.itemType).to.equal(0);
      expect(metadata.owner.toBase58()).to.equal(admin.publicKey.toBase58());
      expect(metadata.mint.toBase58()).to.equal(
        itemMintKeypair.publicKey.toBase58()
      );
    });

    it("Fails to create item with invalid type", async () => {
      const fakeMint = Keypair.generate();
      const [fakeMetadata] = PublicKey.findProgramAddressSync(
        [Buffer.from("item_metadata"), fakeMint.publicKey.toBuffer()],
        itemNft.programId
      );
      const fakeAta = getAssociatedTokenAddressSync(
        fakeMint.publicKey,
        admin.publicKey,
        false,
        TOKEN_PROGRAM_ID
      );
      const [fakeMetaplex] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("metadata"),
          TOKEN_METADATA_PROGRAM_ID.toBuffer(),
          fakeMint.publicKey.toBuffer(),
        ],
        TOKEN_METADATA_PROGRAM_ID
      );

      try {
        await itemNft.methods
          .createItem(10)
          .accounts({
            itemMetadata: fakeMetadata,
            itemMint: fakeMint.publicKey,
            playerItemAccount: fakeAta,
            metadataAccount: fakeMetaplex,
            player: admin.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            metadataProgram: TOKEN_METADATA_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([fakeMint])
          .rpc();
        expect.fail("Should have thrown");
      } catch (e: any) {
        expect(e.error?.errorCode?.code || e.message).to.contain(
          "InvalidItemType"
        );
      }
    });

    it("Burns an NFT item", async () => {
      const tx = await itemNft.methods
        .burnItem()
        .accounts({
          itemMetadata: itemMetadataPda,
          itemMint: itemMintKeypair.publicKey,
          playerItemAccount: itemAta,
          player: admin.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();

      const info = await provider.connection.getAccountInfo(itemMetadataPda);
      expect(info).to.be.null;
    });
  });

  describe("crafting", () => {
    let craftItemMint: Keypair;
    let craftItemAta: PublicKey;
    let craftItemMetadataPda: PublicKey;
    let craftMetaplexPda: PublicKey;

    before(async () => {
      for (let i = 0; i < 6; i++) {
        await resourceManager.methods
          .mintResource(i, new anchor.BN(20))
          .accounts({
            gameConfig: gameConfigPda,
            resourceMint: resourceMints[i],
            mintAuthority: mintAuthorityPda,
            playerTokenAccount: playerResourceAccounts[i],
            tokenProgram: TOKEN_2022_PROGRAM_ID,
          })
          .rpc();
      }
    });

    it("Crafts a Шабля козака (item type 0)", async () => {
      craftItemMint = Keypair.generate();

      [craftItemMetadataPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("item_metadata"),
          craftItemMint.publicKey.toBuffer(),
        ],
        itemNft.programId
      );

      craftItemAta = getAssociatedTokenAddressSync(
        craftItemMint.publicKey,
        admin.publicKey,
        false,
        TOKEN_PROGRAM_ID
      );

      [craftMetaplexPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("metadata"),
          TOKEN_METADATA_PROGRAM_ID.toBuffer(),
          craftItemMint.publicKey.toBuffer(),
        ],
        TOKEN_METADATA_PROGRAM_ID
      );

      const tx = await crafting.methods
        .craftItem(0)
        .accounts({
          player: admin.publicKey,
          gameConfig: gameConfigPda,
          woodMint: resourceMints[0],
          ironMint: resourceMints[1],
          goldMint: resourceMints[2],
          leatherMint: resourceMints[3],
          stoneMint: resourceMints[4],
          diamondMint: resourceMints[5],
          playerWoodAccount: playerResourceAccounts[0],
          playerIronAccount: playerResourceAccounts[1],
          playerGoldAccount: playerResourceAccounts[2],
          playerLeatherAccount: playerResourceAccounts[3],
          playerStoneAccount: playerResourceAccounts[4],
          playerDiamondAccount: playerResourceAccounts[5],
          itemMetadata: craftItemMetadataPda,
          itemMint: craftItemMint.publicKey,
          playerItemAccount: craftItemAta,
          metadataAccount: craftMetaplexPda,
          resourceManagerProgram: resourceManager.programId,
          itemNftProgram: itemNft.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          token2022Program: TOKEN_2022_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          metadataProgram: TOKEN_METADATA_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([craftItemMint])
        .rpc();

      const metadata = await itemNft.account.itemMetadata.fetch(
        craftItemMetadataPda
      );
      expect(metadata.itemType).to.equal(0);
    });

    it("Fails crafting with invalid item type", async () => {
      const fakeMint = Keypair.generate();
      const [fakeMetadata] = PublicKey.findProgramAddressSync(
        [Buffer.from("item_metadata"), fakeMint.publicKey.toBuffer()],
        itemNft.programId
      );
      const fakeAta = getAssociatedTokenAddressSync(
        fakeMint.publicKey,
        admin.publicKey,
        false,
        TOKEN_PROGRAM_ID
      );
      const [fakeMetaplex] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("metadata"),
          TOKEN_METADATA_PROGRAM_ID.toBuffer(),
          fakeMint.publicKey.toBuffer(),
        ],
        TOKEN_METADATA_PROGRAM_ID
      );

      try {
        await crafting.methods
          .craftItem(10) 
          .accounts({
            player: admin.publicKey,
            gameConfig: gameConfigPda,
            woodMint: resourceMints[0],
            ironMint: resourceMints[1],
            goldMint: resourceMints[2],
            leatherMint: resourceMints[3],
            stoneMint: resourceMints[4],
            diamondMint: resourceMints[5],
            playerWoodAccount: playerResourceAccounts[0],
            playerIronAccount: playerResourceAccounts[1],
            playerGoldAccount: playerResourceAccounts[2],
            playerLeatherAccount: playerResourceAccounts[3],
            playerStoneAccount: playerResourceAccounts[4],
            playerDiamondAccount: playerResourceAccounts[5],
            itemMetadata: fakeMetadata,
            itemMint: fakeMint.publicKey,
            playerItemAccount: fakeAta,
            metadataAccount: fakeMetaplex,
            resourceManagerProgram: resourceManager.programId,
            itemNftProgram: itemNft.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            token2022Program: TOKEN_2022_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            metadataProgram: TOKEN_METADATA_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([fakeMint])
          .rpc();
        expect.fail("Should have thrown");
      } catch (e: any) {
        expect(e.error?.errorCode?.code || e.message).to.contain(
          "InvalidItemType"
        );
      }
    });

    describe("marketplace", () => {
      it("Sells an item for MagicToken", async () => {
        const magicAta = getAssociatedTokenAddressSync(
          magicTokenMintPda,
          admin.publicKey,
          false,
          TOKEN_2022_PROGRAM_ID
        );

        const tx = await marketplace.methods
          .sellItem()
          .accounts({
            player: admin.publicKey,
            itemMetadata: craftItemMetadataPda,
            itemMint: craftItemMint.publicKey,
            playerItemAccount: craftItemAta,
            magicTokenConfig: magicTokenConfigPda,
            magicTokenMint: magicTokenMintPda,
            magicMintAuthority: magicMintAuthorityPda,
            playerMagicAccount: magicAta,
            itemNftProgram: itemNft.programId,
            magicTokenProgram: magicToken.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            token2022Program: TOKEN_2022_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .rpc();

        const info = await provider.connection.getAccountInfo(
          craftItemMetadataPda
        );
        expect(info).to.be.null;
      });

      it("Fails to sell non-existent item", async () => {
        const emptyMint = Keypair.generate();
        const [emptyMetadata] = PublicKey.findProgramAddressSync(
          [Buffer.from("item_metadata"), emptyMint.publicKey.toBuffer()],
          itemNft.programId
        );

        try {
          await marketplace.methods
            .sellItem()
            .accounts({
              player: admin.publicKey,
              itemMetadata: emptyMetadata,
              itemMint: emptyMint.publicKey,
              playerItemAccount: getAssociatedTokenAddressSync(
                emptyMint.publicKey,
                admin.publicKey,
                false,
                TOKEN_PROGRAM_ID
              ),
              magicTokenConfig: magicTokenConfigPda,
              magicTokenMint: magicTokenMintPda,
              magicMintAuthority: magicMintAuthorityPda,
              playerMagicAccount: getAssociatedTokenAddressSync(
                magicTokenMintPda,
                admin.publicKey,
                false,
                TOKEN_2022_PROGRAM_ID
              ),
              itemNftProgram: itemNft.programId,
              magicTokenProgram: magicToken.programId,
              tokenProgram: TOKEN_PROGRAM_ID,
              token2022Program: TOKEN_2022_PROGRAM_ID,
              associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
              systemProgram: SystemProgram.programId,
            })
            .rpc();
          expect.fail("Should have thrown");
        } catch (e: any) {
          expect(e.message).to.exist;
        }
      });
    });
  });

  describe("Integration: Full game flow", () => {
    it("Complete flow: craft Посох -> sell for MagicToken", async () => {
      const posohMint = Keypair.generate();
      const [posohMetadata] = PublicKey.findProgramAddressSync(
        [Buffer.from("item_metadata"), posohMint.publicKey.toBuffer()],
        itemNft.programId
      );
      const posohAta = getAssociatedTokenAddressSync(
        posohMint.publicKey,
        admin.publicKey,
        false,
        TOKEN_PROGRAM_ID
      );
      const [posohMetaplex] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("metadata"),
          TOKEN_METADATA_PROGRAM_ID.toBuffer(),
          posohMint.publicKey.toBuffer(),
        ],
        TOKEN_METADATA_PROGRAM_ID
      );

      await crafting.methods
        .craftItem(1)
        .accounts({
          player: admin.publicKey,
          gameConfig: gameConfigPda,
          woodMint: resourceMints[0],
          ironMint: resourceMints[1],
          goldMint: resourceMints[2],
          leatherMint: resourceMints[3],
          stoneMint: resourceMints[4],
          diamondMint: resourceMints[5],
          playerWoodAccount: playerResourceAccounts[0],
          playerIronAccount: playerResourceAccounts[1],
          playerGoldAccount: playerResourceAccounts[2],
          playerLeatherAccount: playerResourceAccounts[3],
          playerStoneAccount: playerResourceAccounts[4],
          playerDiamondAccount: playerResourceAccounts[5],
          itemMetadata: posohMetadata,
          itemMint: posohMint.publicKey,
          playerItemAccount: posohAta,
          metadataAccount: posohMetaplex,
          resourceManagerProgram: resourceManager.programId,
          itemNftProgram: itemNft.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          token2022Program: TOKEN_2022_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          metadataProgram: TOKEN_METADATA_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([posohMint])
        .rpc();

      const itemData = await itemNft.account.itemMetadata.fetch(posohMetadata);
      expect(itemData.itemType).to.equal(1);

      const magicAta = getAssociatedTokenAddressSync(
        magicTokenMintPda,
        admin.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      await marketplace.methods
        .sellItem()
        .accounts({
          player: admin.publicKey,
          itemMetadata: posohMetadata,
          itemMint: posohMint.publicKey,
          playerItemAccount: posohAta,
          magicTokenConfig: magicTokenConfigPda,
          magicTokenMint: magicTokenMintPda,
          magicMintAuthority: magicMintAuthorityPda,
          playerMagicAccount: magicAta,
          itemNftProgram: itemNft.programId,
          magicTokenProgram: magicToken.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          token2022Program: TOKEN_2022_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const closedMeta = await provider.connection.getAccountInfo(
        posohMetadata
      );
      expect(closedMeta).to.be.null;
    });
  });

  describe("Security: Unauthorized access", () => {
    it("Fails to create resource mint by non-admin", async () => {
      const fakeAdmin = Keypair.generate();

      const sig = await provider.connection.requestAirdrop(
        fakeAdmin.publicKey,
        1_000_000_000
      );
      await provider.connection.confirmTransaction(sig);

      const [fakeMintPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("resource_mint"), Buffer.from([0])],
        resourceManager.programId
      );

      try {
        await resourceManager.methods
          .createResourceMint(0)
          .accounts({
            gameConfig: gameConfigPda,
            resourceMint: fakeMintPda,
            mintAuthority: mintAuthorityPda,
            admin: fakeAdmin.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_2022_PROGRAM_ID,
          })
          .signers([fakeAdmin])
          .rpc();
        expect.fail("Should have thrown");
      } catch (e: any) {
        expect(e.message).to.exist;
      }
    });
  });
});
