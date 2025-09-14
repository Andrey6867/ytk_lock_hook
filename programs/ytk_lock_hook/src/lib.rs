// ВЕРСИЯ v6 (Возврат к стабильности)

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount}; // Используем классический 'token'

declare_id!("8X1TrJV9aW3Sn4ucWXMadVLDB94JMfWM9ya5L9pvPYgC");

// --- Константы (без изменений) ---
const LOCK_DURATION_SECS: i64 = 600;
const MAX_CAPACITY: u32 = 256;
const STORE_ENTRY_SIZE: usize = 32 + 8;
const STORE_HEADER_SIZE: usize = 8 + 1 + 32 + 4 + 4;
const fn store_space_for_max() -> usize {
    STORE_HEADER_SIZE + (MAX_CAPACITY as usize) * STORE_ENTRY_SIZE
}

#[program]
pub mod ytk_lock_hook {
    use super::*;

    pub fn init_config(ctx: Context<InitConfig>, capacity: u32) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.bump = ctx.bumps.config;
        config.mint = ctx.accounts.mint.key();
        config.authority = ctx.accounts.authority.key();

        let store = &mut ctx.accounts.store;
        store.bump = ctx.bumps.store;
        store.mint = config.mint;
        store.capacity = capacity.min(MAX_CAPACITY);
        store.entries = Vec::with_capacity(store.capacity as usize);
        Ok(())
    }

    pub fn add_to_whitelist(ctx: Context<ManageWhitelist>) -> Result<()> {
        ctx.accounts.whitelist_entry.bump = ctx.bumps.whitelist_entry;
        Ok(())
    }

    pub fn remove_from_whitelist(_ctx: Context<RemoveFromWhitelist>) -> Result<()> {
        Ok(())
    }

    // Возвращаем простую сигнатуру функции
    pub fn execute(ctx: Context<Execute>, amount: u64) -> Result<()> {
        let store = &mut ctx.accounts.store;
        let source_owner = ctx.accounts.owner.key();
        let destination_owner = ctx.accounts.destination_token.owner;

        let is_source_whitelisted = ctx.accounts.source_whitelist_entry.is_some();
        let is_destination_whitelisted = ctx.accounts.destination_whitelist_entry.is_some();

        if !is_source_whitelisted {
            if let Some(unlock_timestamp) = find_lock(&store.entries, source_owner) {
                let now = Clock::get()?.unix_timestamp;
                if now < unlock_timestamp {
                    return err!(ErrorCode::AccountLocked);
                }
            }
        }

        if !is_destination_whitelisted {
            let pre_balance = ctx.accounts.destination_token.amount;
            let post_balance = pre_balance.checked_add(amount).ok_or(ErrorCode::Overflow)?;
            let one_token = 10u64.pow(ctx.accounts.mint.decimals as u32);

            if post_balance > one_token {
                let now = Clock::get()?.unix_timestamp;
                let new_unlock_timestamp = now.saturating_add(LOCK_DURATION_SECS);
                
                let capacity = store.capacity;
                upsert_lock(&mut store.entries, destination_owner, new_unlock_timestamp, capacity);
            }
        }

        Ok(())
    }
}

/* ----------------------- ACCOUNTS ----------------------- */
#[derive(Accounts)]
pub struct InitConfig<'info> {
    #[account(init, payer = authority, space = 8 + LockConfig::INIT_SPACE, seeds = [b"cfg", mint.key().as_ref()], bump)]
    pub config: Account<'info, LockConfig>,
    #[account(init, payer = authority, space = store_space_for_max(), seeds = [b"store", mint.key().as_ref()], bump)]
    pub store: Account<'info, LockStore>,
    /// CHECK: Not read from.
    pub mint: UncheckedAccount<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ManageWhitelist<'info> {
    #[account(seeds = [b"cfg", config.mint.as_ref()], bump = config.bump)]
    pub config: Account<'info, LockConfig>,
    /// CHECK: Not read from.
    pub whitelisted_address: UncheckedAccount<'info>,
    #[account(init, payer = authority, seeds = [b"whitelist", config.key().as_ref(), whitelisted_address.key().as_ref()], bump, space = 8 + WhitelistEntry::INIT_SPACE)]
    pub whitelist_entry: Account<'info, WhitelistEntry>,
    #[account(mut, address = config.authority)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RemoveFromWhitelist<'info> {
    #[account(seeds = [b"cfg", config.mint.as_ref()], bump = config.bump)]
    pub config: Account<'info, LockConfig>,
    /// CHECK: Not read from.
    pub whitelisted_address: UncheckedAccount<'info>,
    #[account(mut, seeds = [b"whitelist", config.key().as_ref(), whitelisted_address.key().as_ref()], bump = whitelist_entry.bump, close = authority)]
    pub whitelist_entry: Account<'info, WhitelistEntry>,
    #[account(mut, address = config.authority)]
    pub authority: Signer<'info>,
}


// Возвращаем автоматическую обработку аккаунтов
#[derive(Accounts)]
pub struct Execute<'info> {
    #[account(token::mint = mint)]
    pub source_token: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    #[account(token::mint = mint)]
    pub destination_token: Account<'info, TokenAccount>,

    /// CHECK: The owner of the source token account, validated by the token program
    pub owner: UncheckedAccount<'info>,
    #[account(seeds = [b"cfg", mint.key().as_ref()], bump = config.bump)]
    pub config: Account<'info, LockConfig>,
    #[account(mut, seeds = [b"store", mint.key().as_ref()], bump = store.bump)]
    pub store: Account<'info, LockStore>,
    #[account(seeds = [b"whitelist", config.key().as_ref(), owner.key().as_ref()], bump)]
    /// CHECK: This may not exist
    pub source_whitelist_entry: Option<Account<'info, WhitelistEntry>>,
    #[account(seeds = [b"whitelist", config.key().as_ref(), destination_token.owner.as_ref()], bump)]
    /// CHECK: This may not exist
    pub destination_whitelist_entry: Option<Account<'info, WhitelistEntry>>,
}

/* ------------------------ STATE & ERRORS (без изменений) ------------------------- */
#[account]
#[derive(InitSpace)]
pub struct LockConfig { pub bump: u8, pub mint: Pubkey, pub authority: Pubkey }
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct HolderLock { pub key: Pubkey, pub until: i64 }
#[account]
pub struct LockStore { pub bump: u8, pub mint: Pubkey, pub capacity: u32, pub entries: Vec<HolderLock> }
#[account]
#[derive(InitSpace)]
pub struct WhitelistEntry { pub bump: u8 }

#[error_code]
pub enum ErrorCode {
    #[msg("This account is currently locked")]
    AccountLocked,
    #[msg("Arithmetic overflow")]
    Overflow,
}

fn find_lock(entries: &[HolderLock], key: Pubkey) -> Option<i64> {
    entries.iter().find(|e| e.key == key).map(|e| e.until)
}
fn upsert_lock(entries: &mut Vec<HolderLock>, key: Pubkey, until: i64, capacity: u32) {
    if let Some(entry) = entries.iter_mut().find(|e| e.key == key) {
        entry.until = until;
    } else if entries.len() < capacity as usize {
        entries.push(HolderLock { key, until });
    }
}