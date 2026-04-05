/** Smoke test for shared IDL-loading utilities and canonical program IDs. */
import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";
import { createRequire } from "module";

const require = createRequire(`${process.cwd()}/tests/bootstrap.ts`);
const {
  PROGRAM_IDS,
  PROGRAM_NAMES,
  getGamePrograms,
  readProgramIdl,
} = require("../utils/programs");

describe("bootstrap scaffold", () => {
  const provider = new anchor.AnchorProvider(
    new anchor.web3.Connection("http://127.0.0.1:8899", "confirmed"),
    new anchor.Wallet(anchor.web3.Keypair.generate()),
    anchor.AnchorProvider.defaultOptions(),
  );

  it("loads all generated IDLs through the shared helper", () => {
    const programs = getGamePrograms(provider);

    expect(Object.keys(programs)).to.have.length(PROGRAM_NAMES.length);

    for (const programName of PROGRAM_NAMES) {
      const program = programs[programName];
      const idl = readProgramIdl(programName);

      expect(program.programId.toBase58()).to.equal(PROGRAM_IDS[programName]);
      expect(idl.address).to.equal(PROGRAM_IDS[programName]);
      expect(idl.instructions.length).to.be.greaterThan(0);
    }
  });
});
