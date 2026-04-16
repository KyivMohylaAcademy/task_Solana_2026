import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import {
  getAccount,
  getAssociatedTokenAddressSync,
  getMint,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { expect } from "chai";
import { ItemNft } from "../target/types/item_nft";

const { PublicKey, Keypair, LAMPORTS_PER_SOL } = anchor.web3;

// Seeds — must match constants.rs
const ITEM_CONFIG_SEED = Buffer.from("item_config");
const NFT_MINT_AUTHORITY_SEED = Buffer.from("nft_mint_authority");
const MARKETPLACE_AUTHORITY_SEED = Buffer.from("marketplace_auth");

// Metaplex Token Metadata program ID (constant, same on all networks).
const MPL_TOKEN_METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

// Derive Metaplex PDAs for a given mint.
const deriveMetadataPda = (mint: anchor.web3.PublicKey) =>
  PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      MPL_TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
    ],
    MPL_TOKEN_METADATA_PROGRAM_ID
  )[0];

const deriveMasterEditionPda = (mint: anchor.web3.PublicKey) =>
  PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      MPL_TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
      Buffer.from("edition"),
    ],
    MPL_TOKEN_METADATA_PROGRAM_ID
  )[0];

describe("item_nft", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ItemNft as Program<ItemNft>;

  const [itemConfigPda] = PublicKey.findProgramAddressSync(
    [ITEM_CONFIG_SEED],
    program.programId
  );

  const [nftMintAuthorityPda] = PublicKey.findProgramAddressSync(
    [NFT_MINT_AUTHORITY_SEED],
    program.programId
  );

  // The NFT mint keypair — fresh for each test run.
  const mintKeypair = Keypair.generate();
  const metadataPda = deriveMetadataPda(mintKeypair.publicKey);
  const masterEditionPda = deriveMasterEditionPda(mintKeypair.publicKey);

  // Recipient of the NFT (the provider wallet for simplicity).
  const recipient = provider.wallet.publicKey;
  let recipientAta: anchor.web3.PublicKey;

  describe("initialize_item_config", () => {
    it("creates the ItemConfig PDA", async () => {
      await program.methods
        .initializeItemConfig()
        .accounts({ admin: provider.wallet.publicKey })
        .rpc();

      const cfg = await program.account.itemConfig.fetch(itemConfigPda);
      expect(cfg.admin.toBase58()).to.equal(
        provider.wallet.publicKey.toBase58()
      );
      expect(cfg.marketplaceProgram.toBase58()).to.equal(
        PublicKey.default.toBase58()
      );
    });
  });

  describe("set_marketplace_program", () => {
    it("rejects a non-admin caller", async () => {
      const intruder = Keypair.generate();
      const sig = await provider.connection.requestAirdrop(
        intruder.publicKey,
        LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(sig);

      let threw = false;
      try {
        await program.methods
          .setMarketplaceProgram(Keypair.generate().publicKey)
          .accounts({ admin: intruder.publicKey })
          .signers([intruder])
          .rpc();
      } catch (err) {
        threw = true;
        const text = [String(err), ...((err as any)?.logs ?? [])].join("\n");
        expect(text).to.match(/AnchorError.*2001|ConstraintHasOne|has[_ ]one/i);
      }
      expect(threw).to.equal(true);
    });

    it("admin registers the marketplace program", async () => {
      // Register the marketplace program ID. In Step 7 this will be the real
      // marketplace program, but for Step 5 any pubkey is fine as we only
      // test the negative burn path.
      const fakeMarketplace = Keypair.generate().publicKey;

      await program.methods
        .setMarketplaceProgram(fakeMarketplace)
        .accounts({ admin: provider.wallet.publicKey })
        .rpc();

      const cfg = await program.account.itemConfig.fetch(itemConfigPda);
      expect(cfg.marketplaceProgram.toBase58()).to.equal(
        fakeMarketplace.toBase58()
      );
    });
  });

  describe("mint_item_nft", () => {
    before(() => {
      // Compute the ATA address off-chain. The program creates it via
      // `init_if_needed` so we don't need to pre-create it here.
      recipientAta = getAssociatedTokenAddressSync(
        mintKeypair.publicKey,
        recipient,
        false,
        TOKEN_PROGRAM_ID
      );
    });

    it("mints a 1-of-1 NFT with metadata and master edition", async () => {
      await program.methods
        .mintItemNft("Kozak Sword", "KSWD", "https://example.com/kozak-sword.json")
        .accountsPartial({
          itemConfig: itemConfigPda,
          nftMintAuthority: nftMintAuthorityPda,
          mint: mintKeypair.publicKey,
          metadata: metadataPda,
          masterEdition: masterEditionPda,
          recipientAta,
          recipient,
          payer: provider.wallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          metadataProgram: MPL_TOKEN_METADATA_PROGRAM_ID,
        })
        .signers([mintKeypair])
        .rpc();

      // Supply must be exactly 1.
      const mintInfo = await getMint(
        provider.connection,
        mintKeypair.publicKey,
        undefined,
        TOKEN_PROGRAM_ID
      );
      expect(mintInfo.supply.toString()).to.equal("1");
      // After CreateMasterEdition, Metaplex sets mintAuthority to the
      // edition PDA (not None). We just verify it is NOT our nft_mint_authority.
      expect(mintInfo.mintAuthority?.toBase58()).to.not.equal(
        nftMintAuthorityPda.toBase58()
      );

      // Recipient's ATA holds exactly 1 token.
      const ata = await getAccount(
        provider.connection,
        recipientAta,
        undefined,
        TOKEN_PROGRAM_ID
      );
      expect(ata.amount.toString()).to.equal("1");

      // Metadata account must exist on-chain (non-zero data).
      const metadataInfo =
        await provider.connection.getAccountInfo(metadataPda);
      expect(metadataInfo).to.not.equal(null);
      expect(metadataInfo!.data.length).to.be.greaterThan(0);

      // Master edition account must exist.
      const editionInfo =
        await provider.connection.getAccountInfo(masterEditionPda);
      expect(editionInfo).to.not.equal(null);
    });
  });

  describe("burn_item_nft (gating)", () => {
    it("rejects a direct burn not signed by the marketplace program PDA", async () => {
      // A wallet cannot produce a private-key signature for the
      // marketplace_authority PDA. Passing a random keypair fails the seeds
      // constraint; passing the real PDA fails because no key can sign for it.
      const intruder = Keypair.generate();
      const sig = await provider.connection.requestAirdrop(
        intruder.publicKey,
        LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(sig);

      let threw = false;
      try {
        await program.methods
          .burnItemNft()
          .accountsPartial({
            itemConfig: itemConfigPda,
            marketplaceAuthority: intruder.publicKey,
            owner: provider.wallet.publicKey,
            mint: mintKeypair.publicKey,
            tokenAccount: recipientAta,
            metadata: metadataPda,
            masterEdition: masterEditionPda,
            tokenProgram: TOKEN_PROGRAM_ID,
            metadataProgram: MPL_TOKEN_METADATA_PROGRAM_ID,
          })
          .signers([intruder])
          .rpc();
      } catch (err) {
        threw = true;
        const text = [String(err), ...((err as any)?.logs ?? [])].join("\n");
        // Seeds constraint fails — intruder's pubkey != marketplace_authority PDA.
        expect(text).to.match(/AnchorError.*200[06]|ConstraintSeeds|seeds/i);
      }
      expect(threw, "direct burn should have thrown").to.equal(true);
    });
  });
});
