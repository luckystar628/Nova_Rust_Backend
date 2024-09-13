use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServicesErrs {
   UserWalletNotFound,
   UserNotHaveNFTs,
   NFTCollectIsNone,
   NFTsTransactionsIsNone
}

impl fmt::Display for ServicesErrs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServicesErrs::UserWalletNotFound=>write!(f,"Don't have this wallet"),
            ServicesErrs::UserNotHaveNFTs=>write!(f,"The user don't hold nfts"),
            ServicesErrs::NFTCollectIsNone=>write!(f,"Don't have this nft collect data"),
            ServicesErrs::NFTsTransactionsIsNone=>write!(f,"Don't have nft transaction is none"),
        }
    }   
}
impl std::error::Error for ServicesErrs {}