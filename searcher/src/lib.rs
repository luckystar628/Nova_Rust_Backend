use std::{sync::Arc, time::Duration};

use anyhow::Result;
use sei_client::{chain_apis, data_rp_structs::tx_rp_struct, transaction_sort::Transaction};
use serde_json::Value;
use tokio::{sync::mpsc::{UnboundedReceiver, UnboundedSender}, time::sleep};
use websocket::{ClientBuilder, OwnedMessage};

const SEIRPCSERVER:&str="http://173.0.55.178:1317";
const WSURL:&str="ws://173.0.55.178:26657/websocket";

pub async fn run_wss(
    query:&str,
    transaction_sender:UnboundedSender<tx_rp_struct::TransactionData>
) -> Result<()> {

    //定义过滤投票闭包
    let is_aggreate_vote=|json:&Value|->bool{
        json.get("result")
        .and_then(|result| result.get("events"))
        .map_or(true, |event| event.get("aggregate_vote.exchange_rates").is_some())
    };

    // get tx_hash in receive msg
    let get_tx_hash=|json:&Value,keys:&Vec<&str>| ->Option<String>{
        keys.iter().fold(Some(json),|acc,&key|{

            acc.and_then(|inner| inner.get(key))
        
        }).and_then(|val| val.as_array().map(|hash_arr| {
            hash_arr.iter().filter_map(|v| v.as_str().map(|hash| hash.to_string())).collect()}))};

    let sub_msg=serde_json::json!({
        "jsonrpc": "2.0",
        "id": 420,
        "method": "subscribe",
        "params": {
             "query":query
        }
    });

    let transaction_data_sender=Arc::new(transaction_sender);

    're:loop {
        let client_res=ClientBuilder::new(WSURL).unwrap().connect_insecure();
        
        if client_res.is_err(){
            println!("WSS Erro -> {:#?}",client_res.err());
            let _ = sleep(Duration::from_secs(3));
            continue 're;
        };

        let client=Arc::new(tokio::sync::Mutex::new(client_res.unwrap()));
        let sub_msg=OwnedMessage::Text(sub_msg.to_string());
        // send sub msg
        client.lock().await.send_message(&sub_msg).unwrap();

        let transaction_data_sender=Arc::clone(&transaction_data_sender);
        let receive=tokio::spawn(async move {
            loop {
                match client.lock().await.recv_message() {
                    Ok(msg)=>{
                        match msg {
                            OwnedMessage::Text(text_msg)=>{
                                let data:Value=serde_json::from_str(&text_msg).unwrap();
                                //解析路径
                                let keys_path:&Vec<&str>=&vec!["result","events","tx.hash"];
                                if get_tx_hash(&data,keys_path).is_none() || is_aggreate_vote(&data){
                                    continue;
                                };
                                let tx_hash=get_tx_hash(&data,keys_path).unwrap();
                                while let Ok(tx_response_data) = chain_apis::get_transaction_by_tx(Some(SEIRPCSERVER), &tx_hash).await {
                                    let _ = transaction_data_sender.send(tx_response_data);
                                }
                            },
                            _=>continue,
                        }
                    },
                    Err(_)=>break,
                }
            }
        });
        let _ = receive.await;
    }


}

pub async fn save_to_db(
    mut transaction_data_receive:UnboundedReceiver<tx_rp_struct::TransactionData>,
    conn_pool:sqlx::PgPool,
) -> Result<()> {
    while let Ok(transaction) =transaction_data_receive.try_recv()  {
        
        let fee=transaction.get_tx().get_fee();
        let transaction_sender=transaction.get_tx().get_transaction_sender();
        let tx_respone=transaction.get_tx_response();
        let tx_hash=tx_respone.txhash;
        let ts=tx_respone.timestamp;
        
        for log in tx_respone.logs{
            let transaction_type=log.transaction_event_type(
                transaction_sender.to_owned(), 
                fee.to_owned(), 
                ts.to_owned(), 
                tx_hash.to_owned()
            );

            match transaction_type {
                sei_client::transaction_sort::TransactionEvent::NftMint(_) => todo!(),
                sei_client::transaction_sort::TransactionEvent::NftBatchBids(_) => todo!(),
                sei_client::transaction_sort::TransactionEvent::NftOnlyTransfer(_) => todo!(),
                sei_client::transaction_sort::TransactionEvent::NftCretaeAuction(_) => todo!(),
                sei_client::transaction_sort::TransactionEvent::NftCancelAuction(_) => todo!(),
                sei_client::transaction_sort::TransactionEvent::NftPurchaseCart(_) => todo!(),
                sei_client::transaction_sort::TransactionEvent::NftAcceptBid(_) => todo!(),
                sei_client::transaction_sort::TransactionEvent::NftFixedSell(_) => todo!(),
                sei_client::transaction_sort::TransactionEvent::NftOnlyCreateAuction(_) => todo!(),
                sei_client::transaction_sort::TransactionEvent::TokenHeihtSwap(_) => todo!(),
                sei_client::transaction_sort::TransactionEvent::TokenNormalSwap(_) => todo!(),
                sei_client::transaction_sort::TransactionEvent::TokenTransferByWei(_) => todo!(),
                sei_client::transaction_sort::TransactionEvent::TokenTransferByBank(_) => todo!(),
                sei_client::transaction_sort::TransactionEvent::TokenTransferByContract(_) => todo!(),
                sei_client::transaction_sort::TransactionEvent::Delegate(_) => todo!(),
                sei_client::transaction_sort::TransactionEvent::Undelegate(_) => todo!(),
                sei_client::transaction_sort::TransactionEvent::Unkonw => todo!(),
            }
        }
    }
    Ok(())
}