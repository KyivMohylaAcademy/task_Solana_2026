/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/resource_manager.json`.
 */
export type ResourceManager = {
  "address": "DFtQE4puDvEMk1vYHhx3gQvfjUieWj1YtkhDKoyGCG1y",
  "metadata": {
    "name": "resourceManager",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Resource manager program for Cossack Business"
  },
  "instructions": [
    {
      "name": "adminMintResource",
      "docs": [
        "Admin-only resource minting for test setup. Gated by GameConfig.admin."
      ],
      "discriminator": [
        109,
        201,
        186,
        218,
        29,
        205,
        28,
        34
      ],
      "accounts": [
        {
          "name": "admin",
          "docs": [
            "Must be the GameConfig admin (deployer wallet)."
          ],
          "writable": true,
          "signer": true,
          "relations": [
            "gameConfig"
          ]
        },
        {
          "name": "gameConfig",
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
            ]
          }
        },
        {
          "name": "mint",
          "writable": true
        },
        {
          "name": "recipientAta",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "recipient"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mint"
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
          "name": "recipient"
        },
        {
          "name": "resourceMintAuth",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  114,
                  101,
                  115,
                  111,
                  117,
                  114,
                  99,
                  101,
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
            ]
          }
        },
        {
          "name": "tokenProgram",
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
      "args": [
        {
          "name": "resourceId",
          "type": "u8"
        },
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "burnResource",
      "docs": [
        "Burn resource tokens. Callable only by the crafting program via CPI."
      ],
      "discriminator": [
        252,
        54,
        4,
        35,
        74,
        224,
        187,
        19
      ],
      "accounts": [
        {
          "name": "cpiAuth",
          "docs": [
            "The authorized caller's cpi_auth PDA. Must be from the crafting program."
          ]
        },
        {
          "name": "gameConfig",
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
            ]
          }
        },
        {
          "name": "mint",
          "writable": true
        },
        {
          "name": "sourceAta",
          "docs": [
            "The owner's ATA to burn from. Must already exist."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "owner"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mint"
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
          "name": "owner",
          "docs": [
            "The token holder (must sign — SPL Token enforces ATA ownership)."
          ],
          "signer": true
        },
        {
          "name": "tokenProgram",
          "address": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
        }
      ],
      "args": [
        {
          "name": "resourceId",
          "type": "u8"
        },
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "initResourceMint",
      "docs": [
        "Initialize a single Token-2022 resource mint with embedded metadata."
      ],
      "discriminator": [
        49,
        140,
        127,
        77,
        177,
        166,
        189,
        170
      ],
      "accounts": [
        {
          "name": "payer",
          "docs": [
            "Payer for account creation."
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "mint",
          "docs": [
            "New Token-2022 mint. Caller generates a fresh Keypair and passes it as signer."
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "resourceMintAuth",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  114,
                  101,
                  115,
                  111,
                  117,
                  114,
                  99,
                  101,
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
            ]
          }
        },
        {
          "name": "tokenProgram",
          "address": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "resourceId",
          "type": "u8"
        },
        {
          "name": "name",
          "type": "string"
        },
        {
          "name": "symbol",
          "type": "string"
        },
        {
          "name": "uri",
          "type": "string"
        }
      ]
    },
    {
      "name": "initialize",
      "docs": [
        "Create the GameConfig PDA with all mint addresses and item prices."
      ],
      "discriminator": [
        175,
        175,
        109,
        31,
        13,
        152,
        155,
        237
      ],
      "accounts": [
        {
          "name": "admin",
          "writable": true,
          "signer": true
        },
        {
          "name": "gameConfig",
          "writable": true,
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
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "resourceMints",
          "type": {
            "array": [
              "pubkey",
              6
            ]
          }
        },
        {
          "name": "magicTokenMint",
          "type": "pubkey"
        },
        {
          "name": "itemPrices",
          "type": {
            "array": [
              "u64",
              4
            ]
          }
        }
      ]
    },
    {
      "name": "mintResource",
      "docs": [
        "Mint resource tokens. Callable only by the search or crafting program via CPI."
      ],
      "discriminator": [
        2,
        118,
        133,
        91,
        220,
        176,
        214,
        105
      ],
      "accounts": [
        {
          "name": "cpiAuth",
          "docs": [
            "The authorized caller's cpi_auth PDA (seeds: [\"cpi_auth\"] from the caller's program ID)."
          ]
        },
        {
          "name": "gameConfig",
          "docs": [
            "GameConfig PDA — used to verify the mint address matches the resource_id."
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
            ]
          }
        },
        {
          "name": "mint",
          "docs": [
            "The resource mint matching resource_id in GameConfig."
          ],
          "writable": true
        },
        {
          "name": "recipientAta",
          "docs": [
            "Recipient's ATA; initialized if it doesn't exist yet."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "recipient"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mint"
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
          "name": "recipient",
          "docs": [
            "The recipient wallet."
          ]
        },
        {
          "name": "resourceMintAuth",
          "docs": [
            "PDA that holds mint authority over resource mints."
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  114,
                  101,
                  115,
                  111,
                  117,
                  114,
                  99,
                  101,
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
            ]
          }
        },
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "tokenProgram",
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
      "args": [
        {
          "name": "resourceId",
          "type": "u8"
        },
        {
          "name": "amount",
          "type": "u64"
        }
      ]
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
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "invalidResourceId",
      "msg": "Resource ID must be 0-5"
    },
    {
      "code": 6001,
      "name": "invalidMint",
      "msg": "Mint address does not match the resource ID in GameConfig"
    },
    {
      "code": 6002,
      "name": "unauthorized",
      "msg": "Caller is not authorized to invoke this instruction"
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
    }
  ]
};
