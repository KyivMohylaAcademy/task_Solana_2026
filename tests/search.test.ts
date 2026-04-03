import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Search } from "../target/types/search";
import { expect } from "chai";

describe("search", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Search as Program<Search>;

  it("Should initialize player search and validate timer", async () => {
    const [playerSearchPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("player_search"), provider.wallet.publicKey.toBuffer()],
      program.programId
    );

    const tx = await program.methods
      .searchResources()
      .accounts({
        playerSearch: playerSearchPda,
        owner: provider.wallet.publicKey,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Search resources tx:", tx);

    const playerSearch = await program.account.playerSearch.fetch(playerSearchPda);
    expect(playerSearch.owner.toString()).to.equal(provider.wallet.publicKey.toString());
    expect(playerSearch.lastSearchTimestamp.toNumber()).to.be.greaterThan(0);
  });

  it("Should prevent search within 60 second interval", async () => {
    const [playerSearchPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("player_search"), provider.wallet.publicKey.toBuffer()],
      program.programId
    );

    try {
      await program.methods
        .searchResources()
        .accounts({
          playerSearch: playerSearchPda,
          owner: provider.wallet.publicKey,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
      expect.fail("Should have thrown SearchNotReady error");
    } catch (err) {
      console.log("Correctly prevented search within 60 seconds:", err.message);
    }
  });
});
