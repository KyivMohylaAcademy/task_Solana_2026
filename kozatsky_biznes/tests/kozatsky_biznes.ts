import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { KozatskyBiznes } from "../target/types/kozatsky_biznes";

describe("kozatsky_biznes", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.kozatskyBiznes as Program<KozatskyBiznes>;

  it("Is initialized!", async () => {
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
