// import modules
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked},
};

// 使用 src/states/escrow.rs 定義的 Escrow 數據
use crate::states::Escrow;

#[derive(Accounts)]
#[instruction(seed: u64, initializer_amount: u64)] // 用於初始化的參數
pub struct Initialize<'info> { // 定義初始化結構
    #[account(mut)] // mutable
    pub initializer: Signer<'info>, // Signer Type Account
    pub mint_a: Account<'info, Mint>, // Token A 的 Mint Account
    pub mint_b: Account<'info, Mint>, // Token B 的 Mint Account
    #[account(
        mut,
        constraint = initializer_ata_a.amount >= initializer_amount, // 限制:確認有足夠 token 進行初始化
        associated_token::mint = mint_a, // 確保是 Token A 的 ATA
        associated_token::authority = initializer // 確保 Token 擁有者是 initializer
    )]
    pub initializer_ata_a: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = initializer, // 指定誰付 rent
        space = Escrow::INIT_SPACE, // 指定空間大小
        seeds = [b"state".as_ref(), &seed.to_le_bytes()], //生成 PDA
        bump
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(
        init_if_needed,
        payer = initializer, // 指定誰付 rent
        associated_token::mint = mint_a, // 確保是 Token A 的 ATA
        associated_token::authority = escrow // 確保擁有者是 Escrow
    )]
    pub vault: Account<'info, TokenAccount>, // Escorw 裡面存 Token A 的 account
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    // 設置 Escrow Account
    pub fn initialize_escrow(
        &mut self,
        seed: u64,
        bumps: &InitializeBumps,
        initializer_amount: u64,
        taker_amount: u64,
    ) -> Result<()> {
        self.escrow.set_inner(Escrow {
            seed,
            bump: bumps.escrow,
            initializer: self.initializer.key(),
            mint_a: self.mint_a.key(),
            mint_b: self.mint_b.key(),
            initializer_amount,
            taker_amount,
        });
        Ok(())
    }

    // deposit token into Escrow
    pub fn deposit(&mut self, initializer_amount: u64) -> Result<()> {
        transfer_checked(
            self.into_deposit_context(),
            initializer_amount,
            self.mint_a.decimals,
        )
    }

    // deposit 所需要的 context
    // cpi: Cross-Program Invocation
    fn into_deposit_context(&self) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
        let cpi_accounts = TransferChecked {
            from: self.initializer_ata_a.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.initializer.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}
