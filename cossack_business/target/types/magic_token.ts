/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/magic_token.json`.
 */
export type MagicToken = {
  "address": "5sk7gq8TwXpGFe7bxCsgWJ2k7StymKfXzkUD7HUfcMaY",
  "metadata": {
    "name": "magicToken",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "MagicToken program for Cossack Business"
  },
  "instructions": [
    {
      "name": "initMagicTokenMint",
      "docs": [
        "Initialize the single Token-2022 MagicToken mint with embedded metadata."
      ],
      "discriminator": [
        16,
        202,
        160,
        16,
        163,
        117,
        229,
        208
      ],
      "accounts": [
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "mint",
          "docs": [
            "New Token-2022 MagicToken mint. Caller generates a fresh Keypair."
          ],
          "writable": true,
          "signer": true
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
      "name": "mintMagicToken",
      "docs": [
        "Mint MagicToken to a recipient. Callable only by the marketplace program via CPI."
      ],
      "discriminator": [
        161,
        40,
        195,
        234,
        210,
        29,
        232,
        71
      ],
      "accounts": [
        {
          "name": "cpiAuth",
          "docs": [
            "The marketplace program's cpi_auth PDA."
          ]
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
          "name": "amount",
          "type": "u64"
        }
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "unauthorized",
      "msg": "Caller is not authorized to invoke this instruction"
    }
  ]
};
