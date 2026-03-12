use anchor_lang::prelude::*;

use crate::state::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ProposalCreateArgs {
    pub transaction_index: u64,
    pub draft: bool,
}

#[derive(Accounts)]
#[instruction(args: ProposalCreateArgs)]
pub struct ProposalCreate<'info> {
    #[account(
        seeds = [SEED_PREFIX, SEED_MULTISIG, multisig.create_key.as_ref()],
        bump  = multisig.bump,
    )]
    pub multisig: Account<'info, Multisig>,

    #[account(
        init,
        payer = rent_payer,
        space = Proposal::size(multisig.members.len()),
        seeds = [
            SEED_PREFIX,
            multisig.key().as_ref(),
            SEED_TRANSACTION,
            &args.transaction_index.to_le_bytes(),
            SEED_PROPOSAL,
        ],
        bump,
    )]
    pub proposal: Account<'info, Proposal>,

    pub signer: Signer<'info>,

    #[account(mut)]
    pub rent_payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> ProposalCreate<'info> {
    pub fn proposal_create(ctx: Context<Self>, args: ProposalCreateArgs) -> Result<()> {
        let mut proposal = &mut ctx.accounts.proposal;
        
        let multisig = &ctx.accounts.multisig;

        proposal.multisig          = multisig.create_key.key();
        proposal.transaction_index = args.transaction_index;
        proposal.status            = if args.draft {
            ProposalStatus::Draft {
                timestamp: Clock::get()?.unix_timestamp,
            }
        } else {
            ProposalStatus::Active {
                timestamp: Clock::get()?.unix_timestamp,
            }
        };

        proposal.bump      = ctx.bumps.proposal;
        proposal.approved  = vec![];
        proposal.rejected  = vec![];
        proposal.cancelled = vec![];

        Ok(())
    }
}