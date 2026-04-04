import { expect } from "chai";
import { Keypair, PublicKey, SYSVAR_RENT_PUBKEY, SystemProgram } from "@solana/web3.js";
import { expectTxFailure } from "./utils/assert";
import { getAccount, getAssociatedTokenAddressSync } from "@solana/spl-token";

import {
  PROGRAM_IDS,
  craftItem,
  farmResources,
  getResourceBalances,
  getState,
  resourceNames,
} from "./utils/state";

describe("Crafting", () => {
  before(async () => {
    const state = await getState();
    await farmResources(state, Array(resourceNames.length).fill(10));
  });

  const recipes: { label: string; itemType: number; costs: number[] }[] = [
    { label: "Saber", itemType: 0, costs: [1, 3, 0, 1, 0, 0] },
    { label: "Staff", itemType: 1, costs: [2, 0, 1, 0, 0, 1] },
    { label: "Armor", itemType: 2, costs: [0, 2, 1, 4, 0, 0] },
    { label: "Bracelet", itemType: 3, costs: [0, 4, 2, 0, 0, 2] },
  ];

  recipes.forEach(({ label, itemType, costs }) => {
    it(`crafts ${label} and burns the recipe`, async () => {
      const state = await getState();
      const { TOKEN_2022_PROGRAM_ID } = PROGRAM_IDS;
      const before = await getResourceBalances(state.provider, state.playerResourceAtas);

      const minted = await craftItem(state, itemType);

      const after = await getResourceBalances(state.provider, state.playerResourceAtas);
      costs.forEach((cost, idx) => {
        expect(before[idx] - after[idx]).to.equal(cost);
      });

      const account = await getAccount(state.provider.connection, minted.ata, undefined, TOKEN_2022_PROGRAM_ID);
      expect(Number(account.amount)).to.equal(1);

      const metadata = await state.itemNft.account.itemMetadata.fetch(minted.metadata);
      expect(metadata.itemType).to.equal(itemType);
      expect(metadata.owner.toBase58()).to.equal(state.playerAuthority.publicKey.toBase58());
      expect(metadata.mint.toBase58()).to.equal(minted.mint.publicKey.toBase58());
    });
  });

  it("rejects unknown item type", async () => {
    const state = await getState();
    const { TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } = PROGRAM_IDS;

    const badMint = Keypair.generate();
    const [badMetadata] = PublicKey.findProgramAddressSync(
      [Buffer.from("item_meta"), badMint.publicKey.toBuffer()],
      state.itemNft.programId,
    );
    const badAta = getAssociatedTokenAddressSync(
      badMint.publicKey,
      state.playerAuthority.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
    );

    await expectTxFailure(
      state.crafting.methods
        .craft(5)
        .accountsStrict({
          payer: state.wallet.publicKey,
          player: state.playerAuthority.publicKey,
          craftingAuthority: state.craftingAuthority,
          gameConfig: state.gameConfig,
          itemMint: badMint.publicKey,
          itemMetadata: badMetadata,
          mintAuthority: state.itemMintAuthority,
          playerItemAta: badAta,
          mintWood: state.resourceMints[0],
          mintIron: state.resourceMints[1],
          mintGold: state.resourceMints[2],
          mintLeather: state.resourceMints[3],
          mintStone: state.resourceMints[4],
          mintDiamond: state.resourceMints[5],
          ataWood: state.playerResourceAtas[0],
          ataIron: state.playerResourceAtas[1],
          ataGold: state.playerResourceAtas[2],
          ataLeather: state.playerResourceAtas[3],
          ataStone: state.playerResourceAtas[4],
          ataDiamond: state.playerResourceAtas[5],
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
          resourceManagerProgram: state.resourceManager.programId,
          itemNftProgram: state.itemNft.programId,
        })
        .signers([badMint, state.playerAuthority])
        .rpc(),
      /Invalid item type/,
    );
  });
});
