use anchor_lang::error_code;

#[error_code]
pub enum MultisigError {
    #[msg("Invalid Owners")]
    InvalidOwners,

    #[msg("Invalid Threshold")]
    InvalidThreshold,

    #[msg("Not An Owner")]
    NotAnOwner,

    #[msg("Already Approved")]
    AlreadyApproved,

    #[msg("Already Rejected")]
    AlreadyRejected,

    #[msg("Already Executed")]
    AlreadyExecuted,

    #[msg("Already Cancelld")]
    AlreadyCancelled,

    #[msg("Not Enough Signers")]
    NotEnoughSigners,

    #[msg("Not a member")]
    NotAMember,
}