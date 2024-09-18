use sei_client::{data_feild_structs::nft_data_struct::NftAttribute, data_rp_structs::tx_rp_struct::FeeAmount};
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub struct NftHoldTop{
    pub top_gainers:Vec<NftTop>,
    pub top_losser:Vec<NftTop>
}


#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub struct NftTop{
    pub name:String, // CollectionInfo name + # + id 
    pub key:String, // collection + - +id
    pub token_id:String,
    pub image:String,
    pub buy_price:Option<String>,
    pub market_fee:Option<String>,
    pub floor_price:Option<String>,
    pub gas_fee:Vec<FeeAmount>,
    pub unrealized_gains:String,
    pub attributes:Vec<NftAttribute>,
    pub ts:Option<String>,
    pub tx_hash:Option<String>,
}