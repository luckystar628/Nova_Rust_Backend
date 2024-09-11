use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeiClientErrs {
    TxhashNotFound,
    UnkonwTransactionType,
    
    NftCollectNotHaveAddressHold,
    GetNftInfoErro,

    GetTokeninfoByContractErr,
    GetTokenMinterInfoByContractErr,
    GetTokenMarketingInfoByContractErr,
    Unkonw,

}

impl fmt::Display for SeiClientErrs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SeiClientErrs::TxhashNotFound => write!(f, "Transaction hash not found"),
            SeiClientErrs::UnkonwTransactionType=>write!(f,"Unkonw Transaction type || evm and native"),
            SeiClientErrs::NftCollectNotHaveAddressHold=>write!(f,"Generic error: addr_validate errored: decoding bech32 failed: invalid checksum (expected lr66z7 got l666z7): query wasm contract failed: invalid request"),
            SeiClientErrs::GetNftInfoErro=>write!(f,"Get info error"),
            
            SeiClientErrs::GetTokeninfoByContractErr=>write!(f,"Get token info by contract error"),
            SeiClientErrs::GetTokenMinterInfoByContractErr=>write!(f,"Get token minter by contract error"),
            SeiClientErrs::GetTokenMarketingInfoByContractErr=>write!(f,"Get token marekting info by contract error"),

            SeiClientErrs::Unkonw=>write!(f,"Unkonw error"),
            
        }
    }
}
impl std::error::Error for SeiClientErrs {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NovaDBErrs {
    UnfindWallet,
    InsterNewWalletErr,
    UpdateWalletNftHoldErr,
    UpdateWalletNFtHoldOperationlErr,
    UpdateWalletNftTransactionsErr,
    UpdateWalletTokenTransactionsErr,
    UpdateWalletStakeTransactionErr,

    UnfindNFTContract,
    InsterNewNFTContractErr,
    NftFloorPriceNotToday,
    UpdateNFTCollectionErr,
    AcquiteConnPoolErr,

}

impl fmt::Display for NovaDBErrs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NovaDBErrs::UnfindWallet=>write!(f,"Can't find wallet"),
            NovaDBErrs::InsterNewWalletErr=>write!(f,"Inster new wallet error"),
            NovaDBErrs::UpdateWalletNftHoldErr=>write!(f,"Update wallet nft hold error"),
            NovaDBErrs::UpdateWalletNFtHoldOperationlErr=>write!(f,"Operateion error in update wallet nft hold"),
            NovaDBErrs::UpdateWalletNftTransactionsErr=>write!(f,"Update wallet nft transaction error"),
            NovaDBErrs::UpdateWalletTokenTransactionsErr=>write!(f,"Update wallet token transaction error"),
            NovaDBErrs::UpdateWalletStakeTransactionErr=>write!(f,"Update wallet stake transaction error"),
            NovaDBErrs::InsterNewNFTContractErr=>write!(f,"Inster new NFT contract erro"),
            NovaDBErrs::UnfindNFTContract=>write!(f,"Unfind NFT contract"),

            NovaDBErrs::NftFloorPriceNotToday=>write!(f,"The NFT or NFT Collection floor price not today"),
            NovaDBErrs::UpdateNFTCollectionErr=>write!(f,"Update NFT Collection erro"),
            NovaDBErrs::AcquiteConnPoolErr=>write!(f,"Acquite db conn pool erro"),
        }
    }
}
impl std::error::Error for NovaDBErrs {}