use core::str;

use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Token{
    pub amount:String,
    pub denom:String,
}


//  rp for query token contract
#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct TokenMarketingInfo{
    pub project:String,
    pub description:String,
    pub logo:Logo,
    pub marketing:String,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Logo{
    pub url:String
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct TokenMinter{
    pub minter:String,
    pub cap :String
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct _TokenInfo{
    pub name:String,
    pub symbol:String,
    pub decimals:u8,
    pub total_supply:String
}