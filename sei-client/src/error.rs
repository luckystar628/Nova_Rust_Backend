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
