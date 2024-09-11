use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServicesErrs {
   UserWalletNotFound,
   UserNotHaveNFTs,
}

impl fmt::Display for ServicesErrs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServicesErrs::UserWalletNotFound=>write!(f,"Don't have this wallet"),
            ServicesErrs::UserNotHaveNFTs=>write!(f,"The user don't hold nfts"),
        }
    }
}
impl std::error::Error for ServicesErrs {}