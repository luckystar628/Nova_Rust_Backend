use sei_client::data_feild_structs::nft_data_struct::NftAttribute;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub struct UserNFTCollectsHold{
    pub collections:Vec<UserNFTCollectHold>
}

#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub struct UserNFTCollectHold{
    pub name:String,
    pub symbol:String,
    pub creator:String,
    pub contract:String,
    pub floor_price:Option<String>,
    pub nfts_hold:Vec<UserNFTHold>
}

#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub struct UserNFTHold{
    pub name:String, // CollectionInfo name + # + id 
    pub key:String, // collection + - +id
    pub token_id:String,
    pub image:String,
    pub buy_price:Option<String>,
    pub market_fee:Option<String>,
    pub floor_price:Option<String>,
    pub unrealized_gains:Option<String>,
    pub attributes:Vec<NftAttribute>,
    pub ts:Option<String>,
    pub tx_hash:Option<String>,
}