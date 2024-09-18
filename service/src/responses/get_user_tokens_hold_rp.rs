use serde::{Deserialize, Serialize};



#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub struct UserTokenHold{
    pub name:String,
    pub demon:String,
    pub decimals:Option<u8>,
    pub logo_url:Option<String>,
    pub amount:String,
    pub worth_usei:Option<String>,
    pub buy_price:Option<String>,
}

#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub struct UserTokenTop{
    pub top_gainers_tokens:Vec<UserTokenHold>,
    pub top_losser_tokens:Vec<UserTokenHold>
}