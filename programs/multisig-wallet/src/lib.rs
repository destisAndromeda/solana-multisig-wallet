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

}