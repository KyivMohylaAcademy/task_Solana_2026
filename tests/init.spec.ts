import { expect } from "chai";
import { getMagicMintInfo, getState } from "./utils/state";

describe("Initialization", () => {
  it("stores game and magic configs", async () => {
    const state = await getState();
    const config = await state.resourceManager.account.gameConfig.fetch(state.gameConfig);

    expect(config.admin.toBase58()).to.equal(state.wallet.publicKey.toBase58());
    expect(config.magicTokenMint.toBase58()).to.equal(state.magicMint.toBase58());
    expect(config.searchProgram.toBase58()).to.equal(state.search.programId.toBase58());
    expect(config.craftingProgram.toBase58()).to.equal(state.crafting.programId.toBase58());
    expect(config.itemNftProgram.toBase58()).to.equal(state.itemNft.programId.toBase58());
    expect(config.marketplaceProgram.toBase58()).to.equal(state.marketplace.programId.toBase58());

    const magicConfig = await state.magicToken.account.magicTokenConfig.fetch(state.magicConfig);
    expect(magicConfig.admin.toBase58()).to.equal(state.wallet.publicKey.toBase58());
    expect(magicConfig.marketplaceProgram.toBase58()).to.equal(state.marketplace.programId.toBase58());

    const mintInfo = await getMagicMintInfo(state);
    expect(mintInfo.decimals).to.equal(9);
    expect(mintInfo.mintAuthority?.toBase58()).to.equal(state.magicMintAuthority.toBase58());
  });
});
