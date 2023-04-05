#[derive(scale::Encode, scale::Decode, Eq, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    // The caller is not the owner of the contract
    NotOwner,
    // The contract is paused
    ContractIsPaused,
    // The params are invalid
    InvalidParams,
    // The account is not found
    AccountNotFound,
    // The contract does not have enough balance to pay
    NotEnoughBalanceInTreasury,
    // The transfer failed
    TransferFailed,
    // The beneficiary has no unclaimed payments
    NoUnclaimedPayments,
    // Some of the beneficiaries have unclaimed payments
    PaymentsNotUpToDate,
    // Not all the payments are claimed in the last period
    NotAllClaimedInPeriod,
}
