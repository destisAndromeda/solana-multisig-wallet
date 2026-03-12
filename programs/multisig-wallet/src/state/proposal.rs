use anchor_lang::prelude::*;

use crate::errors::*;
use crate::state::Member;

#[account]
pub struct Proposal {
    pub multisig: Pubkey,

    pub transaction_index: u64,

    pub stale_transaction_index: u64,

    pub status: ProposalStatus,

    pub approved:  Vec<Pubkey>,

    pub rejected:  Vec<Pubkey>,
    
    pub cancelled: Vec<Pubkey>,

    pub bump: u8,
}

impl Proposal {
    pub fn size(members_len: usize) -> usize {
        8  + // anchor account descriminator
        32 + // multisig
        8  + // transaction_index
        8  + // stale_transaction_index
        9  + // status
        1  + // bump
        (4 + (members_len * 32)) + // approved  
        (4 + (members_len * 32)) + // rejected
        (4 + (members_len * 32))   // cancelled
    }

    pub fn approve(&mut self, member: Pubkey, threshold: usize) -> Result<()> {
        if let Some(vote_idx) = self.has_voted_reject(member) {
            self.remove_rejection_vote(vote_idx);
        }

        match self.approved.binary_search(&member) {
            Ok(_) => return err!(MultisigError::AlreadyApproved),
            Err(idx) => self.approved.insert(idx, member),
        } 

        if self.approved.len() >= threshold {
            self.status = ProposalStatus::Approved{
                timestamp: Clock::get()?.unix_timestamp,
            };
        }

        Ok(())
    }

    pub fn reject(&mut self, member: Pubkey, cutoff: usize) -> Result<()> {
        if let Some(idx) = self.has_voted_approve(member) {
            self.remove_approval_vote(idx);
        }
        
        match self.rejected.binary_search(&member) {
            Ok(_) => return err!(MultisigError::AlreadyRejected),
            Err(idx) => self.rejected.insert(idx, member),
        }

        if self.rejected.len() >= cutoff {
            self.status = ProposalStatus::Rejected {
                timestamp: Clock::get()?.unix_timestamp,
            };
        }

        Ok(())
    }

    pub fn cancel(&mut self, member: Pubkey, threshold: usize) -> Result<()> {
        match self.cancelled.binary_search(&member) {
            Ok(_) => return err!(MultisigError::AlreadyCancelled),
            Err(idx) => self.cancelled.insert(idx, member),
        }

        if self.cancelled.len() >= threshold {
            self.status = ProposalStatus::Cancelled {
                timestamp: Clock::get()?.unix_timestamp,
            };
        }

        Ok(())
    }

    fn has_voted_approve(&self, member: Pubkey) -> Option<usize>{
        self.approved.binary_search(&member).ok()
    }

    fn has_voted_reject(&self, member: Pubkey) -> Option<usize> {
        self.rejected.binary_search(&member).ok()
    }

    fn remove_rejection_vote(&mut self, idx: usize) {
        self.rejected.remove(idx);
    }

    fn remove_approval_vote(&mut self, idx: usize) {
        self.approved.remove(idx);
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Eq, PartialEq, Clone, Debug)]
pub enum ProposalStatus {
    Draft     { timestamp: i64 },
    Active    { timestamp: i64 },
    Approved  { timestamp: i64 },
    Rejected  { timestamp: i64 },
    Cancelled { timestamp: i64 },
    Executed  { timestamp: i64 },
}