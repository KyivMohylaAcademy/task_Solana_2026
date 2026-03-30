/** Shared program names, IDs and IDL loaders for local scripts and tests. */
import * as anchor from "@coral-xyz/anchor";
import { existsSync, readFileSync } from "fs";
import path from "path";

/** Ordered list of all Anchor programs that make up the game. */
export const PROGRAM_NAMES = [
  "resource_manager",
  "item_nft",
  "crafting",
  "search",
  "marketplace",
  "magic_token",
] as const;

/** Union of supported game program names. */
export type ProgramName = (typeof PROGRAM_NAMES)[number];
/** Runtime Anchor program handle with a generic IDL shape. */
export type GameProgram = anchor.Program<anchor.Idl>;
/** Map of every deployed game program keyed by its canonical name. */
export type GamePrograms = Record<ProgramName, GameProgram>;

/** Canonical local program IDs mirrored from the Rust crates. */
export const PROGRAM_IDS: Record<ProgramName, string> = {
  resource_manager: "CwwxNgkg1s8rjRAAN9zcvLgBCBhXTvCu4L1oAupBqiTe",
  item_nft: "31YqF1ymwThcZTyGCmx6Uqnvjev15JRkWvMSJoxc3wve",
  crafting: "A14WMVRTuuS4JtVcg22BuiWHvhJx1ZhxJS5CrWfy2tHh",
  search: "5vrMHniMhyCnZBK5PWTMMF2w886LDc1Kd3GdN17cbPGh",
  marketplace: "3cPgZBSjpvcuD5FmhGQfCSBFXnz3ZMs573u8UDszgpeW",
  magic_token: "Bvw1CY1ZBu7jE2zmmKkWKe75LfoQvudwT11YxGYaLGW",
};

const REPO_ROOT = path.resolve(__dirname, "..");
const IDL_DIRECTORY = path.join(REPO_ROOT, "target", "idl");

/** Resolves the generated IDL path for a given program. */
const getIdlPath = (name: ProgramName): string => {
  return path.join(IDL_DIRECTORY, `${name}.json`);
};

/** Reads and parses the generated IDL for one program from `target/idl`. */
export const readProgramIdl = (name: ProgramName): anchor.Idl => {
  const idlPath = getIdlPath(name);

  if (!existsSync(idlPath)) {
    throw new Error(
      `IDL for program "${name}" was not found at ${idlPath}. Run "anchor build" first.`,
    );
  }

  return JSON.parse(readFileSync(idlPath, "utf8")) as anchor.Idl;
};

/** Returns the canonical public key for a named program. */
export const getProgramPublicKey = (
  name: ProgramName,
): anchor.web3.PublicKey => {
  return new anchor.web3.PublicKey(PROGRAM_IDS[name]);
};

/** Instantiates Anchor program clients for every deployed game program. */
export const getGamePrograms = (
  provider: anchor.AnchorProvider,
): GamePrograms => {
  return Object.fromEntries(
    PROGRAM_NAMES.map((name) => {
      const idl = readProgramIdl(name);
      return [name, new anchor.Program(idl, provider)];
    }),
  ) as GamePrograms;
};
