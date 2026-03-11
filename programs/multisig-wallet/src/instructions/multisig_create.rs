use anchor_lang::prelude::*;

use crate::state::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct MultisigCreateArgs {
    pub config_authority: Option<Pubkey>,
    
    pub threshold: u16,

    pub time_lock: u32,

    pub members: Vec<Member>,

    pub memo: u64,
}

#[derive(Accounts)]
#[instruction(args: MultisigCreateArgs)]
pub struct MultisigCreate<'info> {
    #[account(
        init,
        payer = creator,
        space = Multisig::size(args.members.len()),
        seeds = [SEED_PREFIX, SEED_MULTISIG, create_key.key().as_ref()],
        bump,
    )]
    pub multisig: Account<'info, Multisig>,

    pub create_key: Signer<'info>,

    #[account(mut)]
    pub creator: Signer<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> MultisigCreate<'info> {
    pub fn multisig_create(ctx: Context<Self>, args: MultisigCreateArgs) -> Result<()> {
        let mut members = args.members;
        members.sort_by_key(|m| m.key);

        let mut multisig = &mut ctx.accounts.multisig;
        multisig.create_key = ctx.accounts.create_key.key();
        multisig.threshold = args.threshold;
        multisig.time_lock = args.time_lock;
        multisig.transaction_index = 0;
        multisig.members = members;
        multisig.bump = ctx.bumps.multisig;

        Ok(())
    }
}
