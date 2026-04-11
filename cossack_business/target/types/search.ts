/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/search.json`.
 */
export type Search = {
  "address": "8idBXvmxQEwn8BCVe5W8nzJqktRsgubP1eFUJ6XQLuRc",
  "metadata": {
    "name": "search",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Search program for Cossack Business"
  },
  "instructions": [
    {
      "name": "registerPlayer",
      "docs": [
        "Register a new player PDA for the signer."
      ],
      "discriminator": [
        242,
        146,
        194,
        234,
        234,
        145,
        228,
        42
      ],
      "accounts": [
        {
          "name": "signer",
          "writable": true,
          "signer": true
        },
        {
          "name": "player",
          "docs": [
            "Player PDA — seeds use the signer's key so each wallet registers only its own player."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  108,
                  97,
                  121,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "signer"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "searchResources",
      "docs": [
        "Search for resources. Mints 3 random resource tokens to the player's ATAs.",
        "Enforces SEARCH_COOLDOWN_SECONDS between calls."
      ],
      "discriminator": [
        218,
        55,
        72,
        118,
        162,
        201,
        69,
        189
      ],
      "accounts": [
        {
          "name": "playerWallet",
          "docs": [
            "Player wallet — must sign and must be the Player PDA owner."
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "player",
          "docs": [
            "Player PDA. `has_one = owner` enforces player_wallet == player.owner."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  108,
                  97,
                  121,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "playerWallet"
              }
            ]
          }
        },
        {
          "name": "owner",
          "relations": [
            "player"
          ]
        },
        {
          "name": "gameConfig",
          "docs": [
            "GameConfig PDA from resource_manager — provides the mint addresses."
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
          "name": "mint0",
          "writable": true
        },
        {
          "name": "mint1",
          "writable": true
        },
        {
          "name": "mint2",
          "writable": true
        },
        {
          "name": "mint3",
          "writable": true
        },
        {
          "name": "mint4",
          "writable": true
        },
        {
          "name": "mint5",
          "writable": true
        },
        {
          "name": "ata0",
          "writable": true
        },
        {
          "name": "ata1",
          "writable": true
        },
        {
          "name": "ata2",
          "writable": true
        },
        {
          "name": "ata3",
          "writable": true
        },
        {
          "name": "ata4",
          "writable": true
        },
        {
          "name": "ata5",
          "writable": true
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
          "name": "resourceMintAuth",
          "writable": true,
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
          "name": "resourceManagerProgram",
          "address": "DFtQE4puDvEMk1vYHhx3gQvfjUieWj1YtkhDKoyGCG1y"
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
      "name": "player",
      "discriminator": [
        205,
        222,
        112,
        7,
        165,
        155,
        206,
        218
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "cooldownNotElapsed",
      "msg": "Search cooldown has not elapsed yet"
    },
    {
      "code": 6001,
      "name": "unauthorized",
      "msg": "Caller is not the player account owner"
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
      "name": "player",
      "docs": [
        "Per-player PDA tracking search cooldown."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "docs": [
              "The wallet that owns this player account."
            ],
            "type": "pubkey"
          },
          {
            "name": "lastSearchTimestamp",
            "docs": [
              "Unix timestamp of the last successful search (0 = never searched)."
            ],
            "type": "i64"
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
