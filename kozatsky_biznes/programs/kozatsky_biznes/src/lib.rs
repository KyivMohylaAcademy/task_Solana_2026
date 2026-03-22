use anchor_lang::prelude::*;

declare_id!("GFTiYZthwodVsXDzRWSVXekVEg7v3SftR3GEgui7bE6T");

#[program]
pub mod kozatsky_biznes {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
