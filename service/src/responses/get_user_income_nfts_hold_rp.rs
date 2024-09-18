use sei_client::data_rp_structs::tx_rp_struct::FeeAmount;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub struct IncomeNFTsCollect{
    pub name:String,
    pub creator:String,
    pub contract:String,
    pub income_nfts:Vec<IncomeNFT>
}

#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub struct IncomeNFT{
    pub name:String, // CollectionInfo name + # + id 
    pub key:String, // collection + - +id
    pub token_id:String,
    pub image:String,
    pub buy_price:String,
    pub sell_price:String,
    pub hold_time:String,
    pub realized_gains:String,
    pub paid_fee:String,
}

#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub enum NFTTransaction {
    Buy(BuyNFTTransaction),
    Sell(SellNFTTransaction)
}

#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub struct BuyNFTTransaction{
    pub collection_address:String,
    pub key:String, // collection + - +id
    pub token_id:String,
    pub buy_price:String,
    pub marketplace_fee:String,
    pub fee:Vec<FeeAmount>,
    pub ts:String,
    pub tx:String,
}

#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub struct SellNFTTransaction{
    pub collection_address:String,
    pub key:String, // collection + - +id
    pub token_id:String,
    pub sell_price:String,
    pub royalties:String,
    pub fee:Vec<FeeAmount>,
    pub ts:String,
    pub tx:String,
}
