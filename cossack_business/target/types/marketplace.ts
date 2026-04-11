/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/marketplace.json`.
 */
export type Marketplace = {
  "address": "6mYp9XMhdaqcRq9xh4EDBmRDGaDEEphzEJzpPF5KEpvX",
  "metadata": {
    "name": "marketplace",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Marketplace program for Cossack Business"
  },
  "instructions": [
    {
      "name": "sellItem",
      "docs": [
        "Sell an NFT to the game. Burns the NFT, mints MagicToken to seller."
      ],
      "discriminator": [
        44,
        114,
        171,
        76,
        76,
        10,
        150,
        246
      ],
      "accounts": [
        {
          "name": "seller",
          "docs": [
            "The seller — must sign, must hold the NFT."
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "gameConfig",
          "docs": [
            "GameConfig — provides item_prices."
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  103,
                  97,
                  109,
                  101,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                182,
                28,
                126,
                59,
                227,
                7,
                200,
                202,
                28,
                233,
                16,
                200,
                104,
                75,
                123,
                157,
                156,
                76,
                32,
                221,
                51,
                104,
                39,
                85,
                185,
                255,
                108,
                41,
                9,
                47,
                80,
                124
              ]
            }
          }
        },
        {
          "name": "nftMint",
          "docs": [
            "The NFT mint (classic SPL Token)."
          ],
          "writable": true
        },
        {
          "name": "sellerNftAta",
          "docs": [
            "Seller's ATA for the NFT. Must have balance ≥ 1."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "seller"
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
            "ItemMetadata PDA — provides item_type for price lookup; closed by burn_nft CPI."
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
            ],
            "program": {
              "kind": "const",
              "value": [
                18,
                37,
                178,
                126,
                232,
                150,
                63,
                79,
                228,
                91,
                195,
                56,
                72,
                253,
                172,
                9,
                82,
                138,
                42,
                232,
                161,
                174,
                40,
                219,
                109,
                110,
                210,
                159,
                190,
                30,
                55,
                160
              ]
            }
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
            ],
            "program": {
              "kind": "const",
              "value": [
                18,
                37,
                178,
                126,
                232,
                150,
                63,
                79,
                228,
                91,
                195,
                56,
                72,
                253,
                172,
                9,
                82,
                138,
                42,
                232,
                161,
                174,
                40,
                219,
                109,
                110,
                210,
                159,
                190,
                30,
                55,
                160
              ]
            }
          }
        },
        {
          "name": "cpiAuth",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  112,
                  105,
                  95,
                  97,
                  117,
                  116,
                  104
                ]
              }
            ]
          }
        },
        {
          "name": "magicTokenMint",
          "docs": [
            "MagicToken mint (Token-2022)."
          ],
          "writable": true
        },
        {
          "name": "sellerMagicAta",
          "writable": true
        },
        {
          "name": "magicMintAuth",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  97,
                  103,
                  105,
                  99,
                  95,
                  109,
                  105,
                  110,
                  116,
                  95,
                  97,
                  117,
                  116,
                  104
                ]
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                72,
                110,
                137,
                236,
                230,
                70,
                165,
                12,
                249,
                6,
                207,
                248,
                23,
                41,
                171,
                112,
                115,
                183,
                225,
                181,
                202,
                83,
                27,
                234,
                5,
                205,
                248,
                232,
                126,
                2,
                251,
                33
              ]
            }
          }
        },
        {
          "name": "itemNftProgram",
          "address": "2DqgLTXd1joDVbtu3DSbocd8C9zExybcdzYH7a6gUXno"
        },
        {
          "name": "magicTokenProgram",
          "address": "5sk7gq8TwXpGFe7bxCsgWJ2k7StymKfXzkUD7HUfcMaY"
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "token2022Program",
          "address": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
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
    }
  ],
  "accounts": [
    {
      "name": "gameConfig",
      "discriminator": [
        45,
        146,
        146,
        33,
        170,
        69,
        96,
        133
      ]
    },
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
      "name": "notNftHolder",
      "msg": "Seller does not hold the NFT"
    },
    {
      "code": 6001,
      "name": "invalidItemType",
      "msg": "Item type must be 0-3"
    }
  ],
  "types": [
    {
      "name": "gameConfig",
      "docs": [
        "Global game configuration. Spec-compliant: exactly 5 fields + bump."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "admin",
            "docs": [
              "The admin pubkey (deployer wallet)."
            ],
            "type": "pubkey"
          },
          {
            "name": "resourceMints",
            "docs": [
              "The 6 resource mint addresses, indexed by resource ID (0–5)."
            ],
            "type": {
              "array": [
                "pubkey",
                6
              ]
            }
          },
          {
            "name": "magicTokenMint",
            "docs": [
              "The MagicToken mint address."
            ],
            "type": "pubkey"
          },
          {
            "name": "itemPrices",
            "docs": [
              "Prices in MagicToken for each of the 4 item types."
            ],
            "type": {
              "array": [
                "u64",
                4
              ]
            }
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
    },
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
