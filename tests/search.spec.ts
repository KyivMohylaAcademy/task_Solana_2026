import { expect } from "chai";

import { expectTxFailure } from "./utils/assert";
import { PROGRAM_IDS, getResourceBalances, getState } from "./utils/state";

describe("Search", () => {
  it("mints three resources and enforces cooldown", async () => {
    const state = await getState();
    const { TOKEN_2022_PROGRAM_ID } = PROGRAM_IDS;
    const {
      provider,
      search,
      playerPda,
      searchAuthority,
      resourceMints,
      playerResourceAtas,
      resourceAuthority,
      gameConfig,
    } = state;

    const before = await getResourceBalances(provider, playerResourceAtas);

    await search.methods
      .searchResources()
      .accountsStrict({
        owner: state.playerAuthority.publicKey,
        player: playerPda,
        gameConfig,
        searchAuthority,
        resourceAuthority,
        mintWood: resourceMints[0],
        mintIron: resourceMints[1],
        mintGold: resourceMints[2],
        mintLeather: resourceMints[3],
        mintStone: resourceMints[4],
        mintDiamond: resourceMints[5],
        ataWood: playerResourceAtas[0],
        ataIron: playerResourceAtas[1],
        ataGold: playerResourceAtas[2],
        ataLeather: playerResourceAtas[3],
        ataStone: playerResourceAtas[4],
        ataDiamond: playerResourceAtas[5],
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        resourceManagerProgram: state.resourceManager.programId,
      })
      .signers([state.playerAuthority])
      .rpc();

    const after = await getResourceBalances(provider, playerResourceAtas);
    const minted = after.reduce((acc, v, idx) => acc + (v - before[idx]), 0);
    expect(minted).to.equal(3);

    await expectTxFailure(
      search.methods
        .searchResources()
        .accountsStrict({
          owner: state.playerAuthority.publicKey,
          player: playerPda,
          gameConfig,
          searchAuthority,
          resourceAuthority,
          mintWood: resourceMints[0],
          mintIron: resourceMints[1],
          mintGold: resourceMints[2],
          mintLeather: resourceMints[3],
          mintStone: resourceMints[4],
          mintDiamond: resourceMints[5],
          ataWood: playerResourceAtas[0],
          ataIron: playerResourceAtas[1],
          ataGold: playerResourceAtas[2],
          ataLeather: playerResourceAtas[3],
          ataStone: playerResourceAtas[4],
          ataDiamond: playerResourceAtas[5],
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          resourceManagerProgram: state.resourceManager.programId,
        })
        .signers([state.playerAuthority])
        .rpc(),
      /Search cooldown active/,
    );
  });
});
