use anchor_lang::prelude::*;

declare_id!("5kmzuRxAsfP5nkezpVLF5HSdSh3ZsYV1WajcZ2z4s8YS");

#[program]
pub mod task_solana_2026 {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
