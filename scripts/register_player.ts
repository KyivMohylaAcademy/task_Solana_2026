/**
 * Register the current wallet as a player.
 *
 * Usage:
 *   pnpm exec ts-node scripts/register_player.ts [--url devnet|localnet]
 */
import * as anchor from "@coral-xyz/anchor";
import { Connection, Keypair, SystemProgram, clusterApiUrl } from "@solana/web3.js";
import fs from "fs";
import path from "path";
import { playerPda } from "../tests/helpers/setup";

async function main() {
  const args = process.argv.slice(2);
  const urlFlag = args.indexOf("--url");
  const cluster = urlFlag >= 0 ? args[urlFlag + 1] : "localnet";
  const rpcUrl = cluster === "devnet" ? clusterApiUrl("devnet") : "http://127.0.0.1:8899";

  const walletPath = process.env.ANCHOR_WALLET ?? `${process.env.HOME}/.config/solana/id.json`;
  const keypairRaw = JSON.parse(fs.readFileSync(walletPath, "utf-8"));
  const owner = Keypair.fromSecretKey(Uint8Array.from(keypairRaw));

  const connection = new Connection(rpcUrl, "confirmed");
  const wallet = new anchor.Wallet(owner);
  const provider = new anchor.AnchorProvider(connection, wallet, { commitment: "confirmed" });
  anchor.setProvider(provider);

  const idl = JSON.parse(fs.readFileSync(path.join(__dirname, "../target/idl/search.json"), "utf-8"));
  const program = new anchor.Program(idl, provider);

  const [pda] = playerPda(owner.publicKey);
  try {
    await (program.methods as any)
      .registerPlayer()
      .accounts({
        owner: owner.publicKey,
        player: pda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log(`✓ Player registered: ${pda.toBase58()}`);
  } catch (e: any) {
    if (e.message?.includes("already in use")) {
      console.log("  Player already registered.");
    } else throw e;
  }
}

main().catch(err => { console.error(err); process.exit(1); });
