use anchor_lang::prelude::*;

declare_id!("EfF5M1kLuDPQEvyGjB2WFaAj2epuWJTEHgVUN9UrAXs6");

#[program]
pub mod resource_manager {
    use super::*;

    /// Initialize the resource manager with 6 base resource mints
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.admin = ctx.accounts.admin.key();
        config.bump = ctx.bumps.config;
        
        // Resource mints will be set individually
        config.resource_mints = [Pubkey::default(); 6];
        
        msg!("Resource manager initialized");
        Ok(())
    }

    /// Create a resource mint (SPL Token-2022 with MetadataPointer)
    pub fn create_resource_mint(
        ctx: Context<CreateResourceMint>,
        resource_id: u8,
        name: String,
        symbol: String,
    ) -> Result<()> {
        require!(resource_id < 6, ErrorCode::InvalidResourceId);
        require!(ctx.accounts.config.admin == ctx.accounts.admin.key(), ErrorCode::Unauthorized);

        let config = &mut ctx.accounts.config;
        config.resource_mints[resource_id as usize] = ctx.accounts.mint.key();

        msg!("Created resource mint: {} ({}) with ID: {}", name, symbol, resource_id);
        Ok(())
    }

    /// Mint resources (only callable by authorized programs: search, crafting)
    pub fn mint_resource(
        ctx: Context<MintResource>,
        resource_id: u8,
        amount: u64,
    ) -> Result<()> {
        require!(resource_id < 6, ErrorCode::InvalidResourceId);
        
        // Verify caller is an authorized program
        let expected_mint = ctx.accounts.config.resource_mints[resource_id as usize];
        require!(expected_mint == ctx.accounts.mint.key(), ErrorCode::InvalidMint);

        msg!("Minted {} units of resource {}", amount, resource_id);
        Ok(())
    }

    /// Burn resources (only callable by authorized programs: crafting)
    pub fn burn_resource(
        ctx: Context<BurnResource>,
        resource_id: u8,
        amount: u64,
    ) -> Result<()> {
        require!(resource_id < 6, ErrorCode::InvalidResourceId);
        
        let expected_mint = ctx.accounts.config.resource_mints[resource_id as usize];
        require!(expected_mint == ctx.accounts.mint.key(), ErrorCode::InvalidMint);

        msg!("Burned {} units of resource {}", amount, resource_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + ResourceConfig::INIT_SPACE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, ResourceConfig>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct CreateResourceMint<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, ResourceConfig>,
    
    /// CHECK: This is the mint account for the resource token
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, anchor_spl::token_2022::Token2022>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct MintResource<'info> {
    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, ResourceConfig>,
    
    /// CHECK: Validated against config
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    
    /// CHECK: Player's token account
    #[account(mut)]
    pub token_account: AccountInfo<'info>,
    
    /// CHECK: Authority PDA from calling program
    pub authority: AccountInfo<'info>,
    
    pub token_program: Program<'info, anchor_spl::token_2022::Token2022>,
}

#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct BurnResource<'info> {
    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, ResourceConfig>,
    
    /// CHECK: Validated against config
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    
    /// CHECK: Player's token account
    #[account(mut)]
    pub token_account: AccountInfo<'info>,
    
    pub owner: Signer<'info>,
    
    pub token_program: Program<'info, anchor_spl::token_2022::Token2022>,
}

#[account]
#[derive(InitSpace)]
pub struct ResourceConfig {
    pub admin: Pubkey,
    pub resource_mints: [Pubkey; 6],
    pub bump: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid resource ID (must be 0-5)")]
    InvalidResourceId,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Invalid mint address")]
    InvalidMint,
}
