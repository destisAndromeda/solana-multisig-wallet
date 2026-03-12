use anchor_lang::prelude::*;

use crate::errors::*;

#[account]
pub struct Multisig {
    /// Key that is used to seed the multisig PDA.
    pub create_key: Pubkey,
    /// Threshold for signatures.
    pub threshold: u16,
    /// How many seconds must pass between transaction voting settlement and execution.
    pub time_lock: u32,
    /// Last transaction index. 0 means no transactions have been created.
    pub transaction_index: u64,

    pub stale_transaction_index: u64,

    /// Members of the multisig.
    pub members: Vec<Member>,
    /// Bump for the multisig PDA seed.
    pub bump: u8,
}

impl Multisig {
    pub fn size(members_length: usize) -> usize {
        8  + //anchor account discriminator
        32 + //create_key
        2  + // threshold
        4  + // time_lock
        8  + // transaction_index
        8  + // stale_transaction_index
        4  + // vector length
        1  + // bump
        (members_length * Member::INIT_SPACE)
    }

    /// Return member index if the pubkey is a member of the multisig.
    pub fn is_member(&self, member_pubkey: Pubkey) -> Option<usize> {
        self.members
            .binary_search_by_key(&member_pubkey, |m| m.key)
            .ok()
    }

    pub fn add_member(&mut self, new_member: Member) {
        self.members.push(new_member);
        self.members.sort_by_key(|m| m.key);
    }

    pub fn remove_member(&mut self, member_pubkey: Pubkey) -> Result<()> {
        let old_member_index = match self.is_member(member_pubkey) {
            Some(old_member_index) => old_member_index,
            None => return err!(MultisigError::NotAMember),
        };

        self.members.remove(old_member_index);
        Ok(())
    }

    fn num_voters(members: &[Member]) -> usize {
        members
            .iter()
            .filter(|m| m.permission.has(Permission::Vote))
            .count()
    }

    pub fn cutoff(&self) -> usize {
        Self::num_voters(&self.members)
            .checked_sub(usize::from(self.threshold))
            .unwrap()
            .checked_add(1)
            .unwrap()
    }

    pub fn member_has_permission(&self, member_pubkey: Pubkey, permission: Permission) -> bool {
        match self.is_member(member_pubkey) {
            Some(idx) => self.members[idx].permission.has(permission),
            _ => false,
        }
    }
}

#[derive(
    AnchorSerialize, AnchorDeserialize, InitSpace, Eq, PartialEq, Clone
)]
pub struct Member {
    pub key: Pubkey,
    pub permission: Permissions, 
}

#[derive(
    AnchorSerialize, AnchorDeserialize, InitSpace, Eq, PartialEq, Clone, Copy, Debug
)]
pub enum Permission {
    Initiate = 1 << 0,
    Vote = 1 << 1,
    Execute = 1 << 2,
}

#[derive(
    AnchorSerialize, AnchorDeserialize, InitSpace, Eq, PartialEq, Default, Debug, Clone, Copy
)]
pub struct Permissions {
    pub mask: u8,
}

impl Permissions {
    pub fn from_vec(permissions: &[Permission]) -> Self {
        let mut mask = 0;
        for permission in permissions {
            mask |= *permission as u8;
        }
        Self { mask }
    }

    pub fn has(&self, permission: Permission) -> bool {
        self.mask & (permission as u8) != 0
    }
}