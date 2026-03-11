mod errors;
mod instructions;
mod state;

use anchor_lang::prelude::*;
use crate::state::*;
use crate::instructions::*;

declare_id!("B7snjmiBzGURpwn9ZLzQFaRCjhHJBpbnLBtLzL9oqA8G");

#[program]
pub mod multisig_wallet {
    use super::*;

    pub fn multisig_create(ctx: Context<MultisigCreate>, args: MultisigCreateArgs) -> Result<()> {
        MultisigCreate::multisig_create(ctx, args)
    }

    pub fn proposal_create(ctx: Context<ProposalCreate>, args: ProposalCreateArgs) -> Result<()> {
        ProposalCreate::proposal_create(ctx, args)
    }
}