use serde::{Deserialize, Serialize};
use crate::data_rp_structs::{token_contract_rp_struct, tx_rp_struct::FeeAmount};

#[derive(Serialize, Deserialize,Clone,Debug)]
pub enum StakeType {
    Delegate,
    Undelegate
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Stake{
    pub validator_address:String,
    pub delegator_address:String,
    pub amount:String,
    pub _type:StakeType,
    pub transaction_sender:Option<String>,
    pub fee:Vec<FeeAmount>,
    pub ts:String,
    pub tx:String,
}