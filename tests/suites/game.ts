import { createSuiteContext } from "../support/context";
import { registerSetupTests } from "../cases/setup";
import { registerSearchTests } from "../cases/search";
import { registerResourceTests } from "../cases/resources";
import { registerCraftingTests } from "../cases/crafting";
import { registerE2ETests } from "../cases/e2e";

describe("kozatskyi-business", () => {
  const ctx = createSuiteContext();

  before(async function () {
    this.timeout(240_000);
    await ctx.initialize();
  });

  registerSetupTests(ctx);
  registerSearchTests(ctx);
  registerResourceTests(ctx);
  registerCraftingTests(ctx);
  registerE2ETests(ctx);
});
