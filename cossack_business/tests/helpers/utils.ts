import * as anchor from "@coral-xyz/anchor";
import {
  Keypair,
  PublicKey,
  SYSVAR_RENT_PUBKEY,
  ComputeBudgetProgram,
} from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  createAssociatedTokenAccountInstruction,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  provider,
  rmProgram,
  searchProgram,
  craftProgram,
  inProgram,
  admin,
  resourceMints,
  searchCallerAuth,
  craftCallerAuth,
  nftAuthorityPda,
  itemNftConfigPda,
} from "./setup";

export { TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID };

export const METAPLEX_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

export const RECIPES = [
  [1, 3, 0, 1, 0, 0],
  [2, 0, 1, 0, 0, 1],
  [0, 2, 1, 4, 0, 0],
  [0, 4, 2, 0, 0, 2],
];

export function findMetadataPda(mint: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      METAPLEX_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
    ],
    METAPLEX_PROGRAM_ID
  )[0];
}

export function findMasterEditionPda(mint: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      METAPLEX_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
      Buffer.from("edition"),
    ],
    METAPLEX_PROGRAM_ID
  )[0];
}

export async function createResourceAtas(player: PublicKey): Promise<PublicKey[]> {
  const atas: PublicKey[] = [];
  for (let i = 0; i < 6; i++) {
    const ata = getAssociatedTokenAddressSync(
      resourceMints[i].publicKey,
      player,
      true,
      TOKEN_2022_PROGRAM_ID
    );
    atas.push(ata);

    const ataInfo = await provider.connection.getAccountInfo(ata);
    if (!ataInfo) {
      const ix = createAssociatedTokenAccountInstruction(
        provider.wallet.publicKey,
        ata,
        player,
        resourceMints[i].publicKey,
        TOKEN_2022_PROGRAM_ID
      );
      const tx = new anchor.web3.Transaction().add(ix);
      await provider.sendAndConfirm(tx);
    }
  }
  return atas;
}

export async function doMultipleSearches(
  playerKp: Keypair,
  count: number,
  gameConfigPda: PublicKey,
  mintAuthorityPda: PublicKey,
) {
  await rmProgram.methods
    .updateSearchCooldown(new anchor.BN(1))
    .accounts({ admin: admin.publicKey, gameConfig: gameConfigPda })
    .rpc();

  const [playerPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("player"), playerKp.publicKey.toBuffer()],
    searchProgram.programId
  );

  const atas: PublicKey[] = [];
  for (let i = 0; i < 6; i++) {
    atas.push(
      getAssociatedTokenAddressSync(
        resourceMints[i].publicKey,
        playerKp.publicKey,
        true,
        TOKEN_2022_PROGRAM_ID
      )
    );
  }

  const remaining = [
    ...resourceMints.map((m) => ({
      pubkey: m.publicKey, isSigner: false, isWritable: true,
    })),
    ...atas.map((a) => ({
      pubkey: a, isSigner: false, isWritable: true,
    })),
  ];

  for (let s = 0; s < count; s++) {
    await new Promise((resolve) => setTimeout(resolve, 1500));
    try {
      await searchProgram.methods
        .searchResources()
        .accounts({
          player: playerKp.publicKey,
          playerAccount: playerPda,
          callerAuthority: searchCallerAuth,
          gameConfig: gameConfigPda,
          mintAuthority: mintAuthorityPda,
          resourceManagerProgram: rmProgram.programId,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .remainingAccounts(remaining)
        .signers([playerKp])
        .rpc();
    } catch {
      await new Promise((resolve) => setTimeout(resolve, 2000));
      await searchProgram.methods
        .searchResources()
        .accounts({
          player: playerKp.publicKey,
          playerAccount: playerPda,
          callerAuthority: searchCallerAuth,
          gameConfig: gameConfigPda,
          mintAuthority: mintAuthorityPda,
          resourceManagerProgram: rmProgram.programId,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .remainingAccounts(remaining)
        .signers([playerKp])
        .rpc();
    }
  }

  await rmProgram.methods
    .updateSearchCooldown(new anchor.BN(2))
    .accounts({ admin: admin.publicKey, gameConfig: gameConfigPda })
    .rpc();
}

export async function craftNftForPlayer(
  playerKp: Keypair,
  gameConfigPda: PublicKey,
): Promise<{ mint: Keypair; itemType: number } | null> {
  const atas: PublicKey[] = [];
  const balances: number[] = [];
  for (let i = 0; i < 6; i++) {
    const ata = getAssociatedTokenAddressSync(
      resourceMints[i].publicKey,
      playerKp.publicKey,
      true,
      TOKEN_2022_PROGRAM_ID
    );
    atas.push(ata);
    const info = await provider.connection.getTokenAccountBalance(ata);
    balances.push(parseInt(info.value.amount));
  }

  let itemType = -1;
  for (let r = 0; r < RECIPES.length; r++) {
    let canAfford = true;
    for (let i = 0; i < 6; i++) {
      if (balances[i] < RECIPES[r][i]) { canAfford = false; break; }
    }
    if (canAfford) { itemType = r; break; }
  }

  if (itemType === -1) return null;

  const nftMint = Keypair.generate();
  const neededIds: number[] = [];
  for (let i = 0; i < 6; i++) {
    if (RECIPES[itemType][i] > 0) neededIds.push(i);
  }

  const resourceRemaining = neededIds.flatMap((id) => {
    const mint = resourceMints[id].publicKey;
    const ata = getAssociatedTokenAddressSync(
      mint, playerKp.publicKey, true, TOKEN_2022_PROGRAM_ID
    );
    return [
      { pubkey: mint, isSigner: false, isWritable: true },
      { pubkey: ata, isSigner: false, isWritable: true },
    ];
  });

  const [itemMetadataPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("item_metadata"), nftMint.publicKey.toBuffer()],
    inProgram.programId
  );
  const playerNftAta = getAssociatedTokenAddressSync(
    nftMint.publicKey, playerKp.publicKey, false, TOKEN_PROGRAM_ID
  );
  const metadataAccount = findMetadataPda(nftMint.publicKey);
  const masterEdition = findMasterEditionPda(nftMint.publicKey);

  const nftRemaining = [
    { pubkey: nftMint.publicKey, isSigner: true, isWritable: true },
    { pubkey: playerNftAta, isSigner: false, isWritable: true },
    { pubkey: metadataAccount, isSigner: false, isWritable: true },
    { pubkey: masterEdition, isSigner: false, isWritable: true },
    { pubkey: METAPLEX_PROGRAM_ID, isSigner: false, isWritable: false },
    { pubkey: itemMetadataPda, isSigner: false, isWritable: true },
    { pubkey: itemNftConfigPda, isSigner: false, isWritable: false },
    { pubkey: nftAuthorityPda, isSigner: false, isWritable: false },
    { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
  ];

  await craftProgram.methods
    .craftItem(itemType, Buffer.from(neededIds))
    .accounts({
      player: playerKp.publicKey,
      callerAuthority: craftCallerAuth,
      gameConfig: gameConfigPda,
      resourceManagerProgram: rmProgram.programId,
      itemNftProgram: inProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      token2022Program: TOKEN_2022_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .remainingAccounts([...resourceRemaining, ...nftRemaining])
    .preInstructions([
      ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 }),
    ])
    .signers([playerKp, nftMint])
    .rpc();

  return { mint: nftMint, itemType };
}
