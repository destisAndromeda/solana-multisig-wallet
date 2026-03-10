use anchor_lang::prelude::*;

use crate::state::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ProposalVoteArgs {
    memo: Option<String>,
}

#[derive(Accounts)]
pub struct ProposalVote<'info> {
    #[account(
        mut,
        seeds = [SEED_PREFIX, SEED_MULTISIG, multisig.create_key.as_ref()],
        bump = multisig.bump, 
    )]
    pub multisig: Account<'info, Multisig>,

    #[account(mut)]
    pub member: Signer<'info>,

    #[account(
        mut,
        seeds = [
            SEED_PREFIX,
            multisig.key().as_ref(),
            SEED_TRANSACTION,
            &proposal.transaction_index.to_le_bytes(),
            SEED_PROPOSAL,
            ],
        bump = proposal.bump,
    )]
    pub proposal: Account<'info, Proposal>, 
}

impl<'info> ProposalVote<'info> {
    pub fn approve(ctx: Context<Self>, _args: ProposalVoteArgs) -> Result<()> {
        let multisig = &mut ctx.accounts.multisig;
        let proposal = &mut ctx.accounts.proposal;
        let member   = &mut ctx.accounts.member;

        let threshold = usize::from(multisig.threshold);

        proposal.approve(member.key(), threshold)?;

        Ok(())
    }

    pub fn reject(ctx: Context<Self>, _args: ProposalVoteArgs) -> Result<()> {
        let multisig = &mut ctx.accounts.multisig;
        let proposal = &mut ctx.accounts.proposal;
        let member   = &mut ctx.accounts.member;

        let cutoff = usize::from(multisig.cutoff());

        proposal.reject(member.key(), cutoff)?;

        Ok(())
    }

    pub fn cancel(ctx: Context<ProposalVote>, _args: ProposalVoteArgs) -> Result<()> {
        let multisig = &mut ctx.accounts.multisig;
        let proposal = &mut ctx.accounts.proposal;
        let member   = &mut ctx.accounts.member;

        proposal
            .cancelled
            .retain(|k| multisig.is_member(*k).is_some());

        let threshold = usize::from(multisig.threshold);

        proposal.cancel(member.key(), threshold)?;

        Ok(())
    }
}