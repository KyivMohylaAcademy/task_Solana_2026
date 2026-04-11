/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/item_nft.json`.
 */
export type ItemNft = {
  "address": "2DqgLTXd1joDVbtu3DSbocd8C9zExybcdzYH7a6gUXno",
  "metadata": {
    "name": "itemNft",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "NFT item program for Cossack Business"
  },
  "instructions": [
    {
      "name": "burnNft",
      "docs": [
        "Burn an NFT item. Callable only from the marketplace program via CPI."
      ],
      "discriminator": [
        119,
        13,
        183,
        17,
        194,
        243,
        38,
        31
      ],
      "accounts": [
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "cpiAuth",
          "docs": [
            "Verified in handler against AUTHORIZED_MARKETPLACE_PROGRAM."
          ]
        },
        {
          "name": "holder",
          "docs": [
            "The current holder of the NFT (must sign to authorize burn of their token)."
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "nftMint",
          "docs": [
            "The NFT mint."
          ],
          "writable": true
        },
        {
          "name": "holderNftAta",
          "docs": [
            "Holder's ATA for the NFT. Thawed, burned, then closed."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "holder"
              },
              {
                "kind": "const",
                "value": [
                  6,
                  221,
                  246,
                  225,
                  215,
                  101,
                  161,
                  147,
                  217,
                  203,
                  225,
                  70,
                  206,
                  235,
                  121,
                  172,
                  28,
                  180,
                  133,
                  237,
                  95,
                  91,
                  55,
                  145,
                  58,
                  140,
                  245,
                  133,
                  126,
                  255,
                  0,
                  169
                ]
              },
              {
                "kind": "account",
                "path": "nftMint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "itemMetadata",
          "docs": [
            "ItemMetadata PDA — closed after burn, lamports returned to payer."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  105,
                  116,
                  101,
                  109,
                  95,
                  109,
                  101,
                  116,
                  97,
                  100,
                  97,
                  116,
                  97
                ]
              },
              {
                "kind": "account",
                "path": "nftMint"
              }
            ]
          }
        },
        {
          "name": "nftAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  110,
                  102,
                  116,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "associatedTokenProgram",
          "address": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "mintNft",
      "docs": [
        "Mint a new NFT item of the given type to the recipient."
      ],
      "discriminator": [
        211,
        57,
        6,
        167,
        15,
        219,
        35,
        251
      ],
      "accounts": [
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "recipient",
          "docs": [
            "The recipient wallet."
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "cpiAuth",
          "docs": [
            "Verified in handler against AUTHORIZED_CRAFTING_PROGRAM."
          ]
        },
        {
          "name": "nftMint",
          "docs": [
            "New NFT mint. Client-generated Keypair, unique per NFT."
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "recipientNftAta",
          "docs": [
            "Recipient's ATA for the NFT. Created inside the handler (after nft_mint init) if absent."
          ],
          "writable": true
        },
        {
          "name": "nftAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  110,
                  102,
                  116,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "metadata",
          "writable": true
        },
        {
          "name": "masterEdition",
          "writable": true
        },
        {
          "name": "tokenMetadataProgram",
          "address": "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
        },
        {
          "name": "itemMetadata",
          "docs": [
            "ItemMetadata PDA storing item_type and original owner."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  105,
                  116,
                  101,
                  109,
                  95,
                  109,
                  101,
                  116,
                  97,
                  100,
                  97,
                  116,
                  97
                ]
              },
              {
                "kind": "account",
                "path": "nftMint"
              }
            ]
          }
        },
        {
          "name": "associatedTokenProgram",
          "address": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "itemType",
          "type": "u8"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "itemMetadata",
      "discriminator": [
        15,
        78,
        221,
        6,
        136,
        152,
        68,
        144
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "invalidItemType",
      "msg": "Item type must be 0-3"
    },
    {
      "code": 6001,
      "name": "unauthorized",
      "msg": "Caller is not authorized to invoke this instruction"
    }
  ],
  "types": [
    {
      "name": "itemMetadata",
      "docs": [
        "On-chain metadata for a minted NFT item."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "itemType",
            "docs": [
              "Item type index (0–3)."
            ],
            "type": "u8"
          },
          {
            "name": "owner",
            "docs": [
              "The wallet that originally received this NFT."
            ],
            "type": "pubkey"
          },
          {
            "name": "mint",
            "docs": [
              "The NFT mint address."
            ],
            "type": "pubkey"
          },
          {
            "name": "bump",
            "docs": [
              "PDA bump."
            ],
            "type": "u8"
          }
        ]
      }
    }
  ]
};
