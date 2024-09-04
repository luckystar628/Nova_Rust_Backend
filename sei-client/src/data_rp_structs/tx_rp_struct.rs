use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::token_contract_rp_struct;


#[derive(Serialize, Deserialize,Clone,Debug)]
pub enum TransactionData {
    Native(NativeTransactionData),
    Evm(EvmTransactionData),
    Bank(BankTransactionData),
    Stake(StakeTransactionData)
}impl TransactionData {
    
    pub fn get_tx_response(&self)->TxResponse {
        match self {
            TransactionData::Native(data) => data.tx_response.to_owned(),
            TransactionData::Evm(data) => data.tx_response.to_owned(),
            TransactionData::Bank(data)=>data.tx_response.to_owned(),
            TransactionData::Stake(data)=>data.tx_response.to_owned(),
        }
    }

    pub fn get_tx(&self)->&dyn TxRp {
        match self {
            TransactionData::Native(data) => &data.tx,
            TransactionData::Evm(data) => &data.tx,
            TransactionData::Bank(data)=>&data.tx,
            TransactionData::Stake(data)=>&data.tx,
        }
    }
}

pub trait TxRp {
    fn get_fee(&self)->Vec<FeeAmount>;
    fn get_transaction_sender(&self)->Option<String>;
    fn get_evm_message_data(&self)->Option<EvmData>;
}

impl TxRp for Tx {
    fn get_fee(&self)->Vec<FeeAmount> {
        self.auth_info.fee.amount.to_owned()
    }
    
    fn get_transaction_sender(&self)->Option<String> {
        Some(self.body.messages[0].sender.to_owned())
    }
    
    fn get_evm_message_data(&self)->Option<EvmData> {
        None
    }
}
impl TxRp for EvmTx {
    fn get_fee(&self)->Vec<FeeAmount> {
        // self.auth_info.fee.amount.to_owned()
        vec![FeeAmount{
            amount:self.body.messages[0].data.gas_fee_cap.to_owned(),
            denom:"wsei".to_string()
        }]
    }
    
    fn get_transaction_sender(&self)->Option<String> {
        None
    }
    
    fn get_evm_message_data(&self)->Option<EvmData> {
        Some(self.body.messages[0].data.to_owned())
    }
}
impl TxRp for BankTx {
    fn get_fee(&self)->Vec<FeeAmount> {
        self.auth_info.fee.amount.to_owned()
    }

    fn get_transaction_sender(&self)->Option<String> {
        Some(self.body.messages[0].from_address.to_owned())
    }

    fn get_evm_message_data(&self)->Option<EvmData> {
        None
    }
}
impl TxRp for StakeTx {
    fn get_fee(&self)->Vec<FeeAmount> {
        self.auth_info.fee.amount.to_owned()
    }

    fn get_transaction_sender(&self)->Option<String> {
        Some(self.body.messages[0].delegator_address.to_owned())
    }

    fn get_evm_message_data(&self)->Option<EvmData> {
        None
    }
}








// 原生
#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct NativeTransactionData{
    pub tx:Tx,
    pub tx_response:TxResponse
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Tx{
    pub auth_info:TxAuthInfo,
    pub body:TxBody,
        signatures:Vec<String>,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct TxResponse {
    pub code:u64,
        codespace:Value,
        data:Value,
    pub events:Value,
    pub gas_used:String,
    pub gas_wanted:String,
        height:String,
        info:Value,
    pub logs:Vec<Log>,
        raw_log:String,
    pub timestamp:String,
        tx:Value,
    pub txhash:String,

}


#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Log{
    pub events:Vec<Event>,
        log:Value,
        msg_index:Value,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Event{
    
    pub attributes:Vec<Attribute>,
    #[serde(rename = "type")]
    pub _type:String,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Attribute{
    pub key:String,
    pub value:String,
}

// Tx 组件
#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct TxAuthInfo{
    pub fee:Fee,
        signer_infos:Value,
}

#[derive(Serialize, Deserialize,Clone,Debug)]

pub struct Fee{
    pub amount:Vec<FeeAmount>,
    pub gas_limit:String,
        granter:String,
        payer:String,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct FeeAmount{
    pub amount:String,
    pub denom:String,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct TxBody{
    pub extension_options:Value,
    pub memo:Value,
    pub messages:Vec<Message>,
    pub non_critical_extension_options:Value,
    pub timeout_height:Value,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct RawLog{
    pub events:String,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Message{
    #[serde(rename = "@type")]
    pub _type:String,
    pub sender:String,
        msg:Value,
        funds:Value,
}


// Emv 
#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct EvmTransactionData{
    pub tx:EvmTx,
    pub tx_response:TxResponse
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct EvmTx{
    pub auth_info:TxAuthInfo,
    pub body:EvmTxBody,
        signatures:Vec<String>,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct EvmTxBody{
    pub extension_options:Value,
    pub memo:Value,
    pub messages:Vec<EvmMessage>,
    pub non_critical_extension_options:Value,
    pub timeout_height:Value,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct EvmMessage{
    #[serde(rename = "@type")]
    pub _type:String,
    pub data:EvmData,
        derived:Value,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct EvmData{
    #[serde(rename = "@type")]
    pub _type:String,
    pub chain_id:String,
    pub nonce:String,
    pub gas_tip_cap:String,
    pub gas_fee_cap:String,
    pub  gas_limit:String,
    pub to:String,
    pub value:String,
    pub data:String,
        accesses:Value,
    pub v:String,
    pub r:String,
    pub s:String,
}

// bank
#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct BankTransactionData{
    pub tx:BankTx,
    pub tx_response:TxResponse
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct BankTx{
    pub auth_info:TxAuthInfo,
    pub body:BankTxTxBody,
        signatures:Vec<String>,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct BankTxTxBody{
    pub extension_options:Value,
    pub memo:Value,
    pub messages:Vec<BankMessage>,
    pub non_critical_extension_options:Value,
    pub timeout_height:Value,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct BankMessage{
    #[serde(rename = "@type")]
    pub _type:String,
    pub from_address:String,
    pub to_address:String,
    pub amount:Vec<token_contract_rp_struct::Token>
}



// delegate and undelegate   || stake
#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct StakeTransactionData{
    pub tx:StakeTx,
    pub tx_response:TxResponse
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct StakeTx{
    pub auth_info:TxAuthInfo,
    pub body:StakeTxTxBody,
        signatures:Vec<String>,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct StakeTxTxBody{
    pub extension_options:Value,
    pub memo:Value,
    pub messages:Vec<StakeMessage>,
    pub non_critical_extension_options:Value,
    pub timeout_height:Value,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct StakeMessage{
    #[serde(rename = "@type")]
    pub _type:String,
    pub delegator_address:String,
    pub validator_address:String,
    pub amount:token_contract_rp_struct::Token
}