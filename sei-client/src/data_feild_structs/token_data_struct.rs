use serde::{Deserialize, Serialize};

use crate::data_rp_structs::tx_rp_struct::FeeAmount;


#[derive(Serialize, Deserialize,Clone,Debug,PartialEq,Eq,Hash)]
pub struct TokenInfo{
    pub name : String,
    pub symbol :String,
    pub project:String,
    pub description:String,
    pub decimals:u8,
    pub total_supply:String,
    pub minter:String,
    pub market:String, 
    pub logo_url:String,
}


// transaction type
#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub enum TokenTransaction {
    TokenSwap(TokenSwap),
    TokenTransfer(TokenTransfer),
    ContractTokenTransfer(ContractTokenTransfer),
}


#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub struct TokenSwap{
    pub source_token:String,
    pub target_token:String,
    pub source_amount:String,
    pub target_amount:String,
    pub transaction_sender:Option<String>,
    pub fee:Vec<FeeAmount>,
    pub ts:String,
    pub tx:String,
}


#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub struct TokenTransfer{
    pub sender:String,
    pub receiver:String,
    pub amount:String,
    pub transaction_sender:Option<String>,
    pub fee:Vec<FeeAmount>,
    pub ts:String,
    pub tx:String,
}



#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub struct ContractTokenTransfer{
    pub contract_address:String,
    pub sender:String,
    pub receiver:String,
    pub amount:String,
    pub transaction_sender:Option<String>,
    pub fee:Vec<FeeAmount>,
    pub ts:String,
    pub tx:String,

}

