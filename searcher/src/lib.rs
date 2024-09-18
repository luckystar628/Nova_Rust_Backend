use std::{sync::Arc, time::Duration};

use anyhow::Result;
use sei_client::{chain_apis, data_feild_structs::{nft_data_struct, token_data_struct}, data_rp_structs::tx_rp_struct, transaction_sort::{Transaction, TransactionEvent}};
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
                TransactionEvent::NftMint(msgs) => {
                    for msg in msgs{

                        let collection_address=msg.collection.to_owned();
                        
                        let nft_info_res=chain_apis::get_nfts_info_by_contract(Some(SEIRPCSERVER), &collection_address, &vec![msg.nft_id.to_owned()]).await;
                        if nft_info_res.is_err(){
                            eprintln!("{:#?}",&nft_info_res.err().unwrap());
                            continue;
                        };
                        let nft_info=nft_info_res.unwrap()[0].to_owned();
                        let update_nft_hold=nova_db::update_wallet_nft_hold(&msg.recipient, &collection_address, &nft_info, "add", &conn_pool).await;
                        let update_nft_mint=nova_db::update_wallet_nft_transactions(&collection_address,&nft_data_struct::NftTransaction::Mint(msg), &conn_pool).await;
                        if update_nft_hold.is_err(){
                            eprintln!("{:#?}",update_nft_hold.err().unwrap());
                        }else if update_nft_mint.is_err() {
                            eprintln!("{:#?}",update_nft_mint.err().unwrap());
                        }else {
                            println!("[*] Save NFT Mint in DB Sucess");
                        }
                    }
                },
                TransactionEvent::NftBatchBids(msgs) => {
                    for msg in msgs{
                        let nft_info={
                            let x=chain_apis::get_nfts_info_by_contract(Some(SEIRPCSERVER), &msg.collection, &vec![msg.nft_id.to_owned()]).await;
                            if x.is_err(){
                                eprintln!("{:#?}",&x.err().unwrap());
                                continue;
                            };
                            x.unwrap()[0].to_owned()
                        };
                        let update_sender_transaction=nova_db::update_wallet_nft_transactions(&msg.sender, &nft_data_struct::NftTransaction::BatchBids(msg.clone()), &conn_pool).await;
                        let update_receive_transaction=nova_db::update_wallet_nft_transactions(&msg.recipient,  &nft_data_struct::NftTransaction::BatchBids(msg.to_owned()),&conn_pool).await;
                        let update_sender_wallet_nft_hold=nova_db::update_wallet_nft_hold(&msg.sender, &msg.collection, &nft_info, "del", &conn_pool).await;
                        let update_receive_wallet_nft_hold=nova_db::update_wallet_nft_hold(&msg.recipient, &msg.collection, &nft_info, "add", &conn_pool).await;
                        
                        if update_sender_transaction.is_err(){
                            eprintln!("{:#?}",&update_sender_transaction.err().unwrap());
                        }else if update_receive_transaction.is_err() {
                            eprintln!("{:#?}",&update_receive_transaction.err().unwrap());
                        }else if update_sender_wallet_nft_hold.is_err() {
                            eprintln!("{:#?}",&update_sender_wallet_nft_hold.err().unwrap());
                        }else if update_receive_wallet_nft_hold.is_err() {
                            eprintln!("{:#?}",&update_receive_wallet_nft_hold.err().unwrap());
                        }else {
                            println!("[*] Save NFT BatchBids data in DB Sucess");
                        }
                    }
                },
                TransactionEvent::NftOnlyTransfer(msgs) =>{
                    for msg in msgs{
                        let nft_info={
                            let x=chain_apis::get_nfts_info_by_contract(Some(SEIRPCSERVER), &msg.collection, &vec![msg.nft_id.to_owned()]).await;
                            if x.is_err(){
                                eprintln!("{:#?}",&x.err().unwrap());
                                continue;
                            };
                            x.unwrap()[0].to_owned()
                        };
                        let update_sender_transaction=nova_db::update_wallet_nft_transactions(&msg.sender, &nft_data_struct::NftTransaction::Transfer(msg.clone()), &conn_pool).await;
                        let update_receive_transaction=nova_db::update_wallet_nft_transactions(&msg.recipient,  &nft_data_struct::NftTransaction::Transfer(msg.to_owned()),&conn_pool).await;
                        let update_sender_wallet_nft_hold=nova_db::update_wallet_nft_hold(&msg.sender, &msg.collection, &nft_info, "del", &conn_pool).await;
                        let update_receive_wallet_nft_hold=nova_db::update_wallet_nft_hold(&msg.recipient, &msg.collection, &nft_info, "add", &conn_pool).await;
                        
                        if update_sender_transaction.is_err(){
                            eprintln!("{:#?}",&update_sender_transaction.err().unwrap());
                        }else if update_receive_transaction.is_err() {
                            eprintln!("{:#?}",&update_receive_transaction.err().unwrap());
                        }else if update_sender_wallet_nft_hold.is_err() {
                            eprintln!("{:#?}",&update_sender_wallet_nft_hold.err().unwrap());
                        }else if update_receive_wallet_nft_hold.is_err() {
                            eprintln!("{:#?}",&update_receive_wallet_nft_hold.err().unwrap());
                        }else {
                            println!("[*] Save NFT OnlyTransfer data in DB Sucess");
                        }
                    }
                },
                TransactionEvent::NftCreateAuction(msgs) =>{
                    for msg in msgs{
                        let nft_info={
                            let x=chain_apis::get_nfts_info_by_contract(Some(SEIRPCSERVER), &msg.collection, &vec![msg.nft_id.to_owned()]).await;
                            if x.is_err(){
                                eprintln!("{:#?}",&x.err().unwrap());
                                continue;
                            };
                            x.unwrap()[0].to_owned()
                        };
                        let update_sender_transaction=nova_db::update_wallet_nft_transactions(&msg.sender, &nft_data_struct::NftTransaction::CreateAuction(msg.clone()), &conn_pool).await;
                        let update_receive_transaction=nova_db::update_wallet_nft_transactions(&msg.recipient,  &nft_data_struct::NftTransaction::CreateAuction(msg.to_owned()),&conn_pool).await;
                        let update_sender_wallet_nft_hold=nova_db::update_wallet_nft_hold(&msg.sender, &msg.collection, &nft_info, "del", &conn_pool).await;
                        let update_receive_wallet_nft_hold=nova_db::update_wallet_nft_hold(&msg.recipient, &msg.collection, &nft_info, "add", &conn_pool).await;
                        
                        if update_sender_transaction.is_err(){
                            eprintln!("{:#?}",&update_sender_transaction.err().unwrap());
                        }else if update_receive_transaction.is_err() {
                            eprintln!("{:#?}",&update_receive_transaction.err().unwrap());
                        }else if update_sender_wallet_nft_hold.is_err() {
                            eprintln!("{:#?}",&update_sender_wallet_nft_hold.err().unwrap());
                        }else if update_receive_wallet_nft_hold.is_err() {
                            eprintln!("{:#?}",&update_receive_wallet_nft_hold.err().unwrap());
                        }else {
                            println!("[*] Save NFT CreatelAuction data in DB Sucess");
                        }
                    }
                },
                TransactionEvent::NftCancelAuction(msgs) => {
                    for msg in msgs{
                        let nft_info={
                            let x=chain_apis::get_nfts_info_by_contract(Some(SEIRPCSERVER), &msg.collection, &vec![msg.nft_id.to_owned()]).await;
                            if x.is_err(){
                                eprintln!("{:#?}",&x.err().unwrap());
                                continue;
                            };
                            x.unwrap()[0].to_owned()
                        };
                        let update_sender_transaction=nova_db::update_wallet_nft_transactions(&msg.sender, &nft_data_struct::NftTransaction::CancelAuction(msg.clone()), &conn_pool).await;
                        let update_receive_transaction=nova_db::update_wallet_nft_transactions(&msg.recipient,  &nft_data_struct::NftTransaction::CancelAuction(msg.to_owned()),&conn_pool).await;
                        let update_sender_wallet_nft_hold=nova_db::update_wallet_nft_hold(&msg.sender, &msg.collection, &nft_info, "del", &conn_pool).await;
                        let update_receive_wallet_nft_hold=nova_db::update_wallet_nft_hold(&msg.recipient, &msg.collection, &nft_info, "add", &conn_pool).await;
                        
                        if update_sender_transaction.is_err(){
                            eprintln!("{:#?}",&update_sender_transaction.err().unwrap());
                        }else if update_receive_transaction.is_err() {
                            eprintln!("{:#?}",&update_receive_transaction.err().unwrap());
                        }else if update_sender_wallet_nft_hold.is_err() {
                            eprintln!("{:#?}",&update_sender_wallet_nft_hold.err().unwrap());
                        }else if update_receive_wallet_nft_hold.is_err() {
                            eprintln!("{:#?}",&update_receive_wallet_nft_hold.err().unwrap());
                        }else {
                            println!("[*] Save NFT CancelAuction data in DB Sucess");
                        }

                    }
                },
                TransactionEvent::NftPurchaseCart(msgs) =>{
                    for msg in msgs{
                        let nft_info={
                            let x=chain_apis::get_nfts_info_by_contract(Some(SEIRPCSERVER), &msg.collection, &vec![msg.nft_id.to_owned()]).await;
                            if x.is_err(){
                                eprintln!("{:#?}",&x.err().unwrap());
                                continue;
                            };
                            x.unwrap()[0].to_owned()
                        };
                        let update_sender_transaction=nova_db::update_wallet_nft_transactions(&msg.seller, &nft_data_struct::NftTransaction::PurchaseCart(msg.clone()), &conn_pool).await;
                        let update_receive_transaction=nova_db::update_wallet_nft_transactions(&msg.recipient,  &nft_data_struct::NftTransaction::PurchaseCart(msg.to_owned()),&conn_pool).await;
                        let update_sender_wallet_nft_hold=nova_db::update_wallet_nft_hold(&msg.seller, &msg.collection, &nft_info, "del", &conn_pool).await;
                        let update_receive_wallet_nft_hold=nova_db::update_wallet_nft_hold(&msg.recipient, &msg.collection, &nft_info, "add", &conn_pool).await;
                        
                        if update_sender_transaction.is_err(){
                            eprintln!("{:#?}",&update_sender_transaction.err().unwrap());
                        }else if update_receive_transaction.is_err() {
                            eprintln!("{:#?}",&update_receive_transaction.err().unwrap());
                        }else if update_sender_wallet_nft_hold.is_err() {
                            eprintln!("{:#?}",&update_sender_wallet_nft_hold.err().unwrap());
                        }else if update_receive_wallet_nft_hold.is_err() {
                            eprintln!("{:#?}",&update_receive_wallet_nft_hold.err().unwrap());
                        }else {
                            println!("[*] Save NFT PurchaseCart data in DB Sucess");
                        }
                    }
                },
                TransactionEvent::NftAcceptBid(msgs) =>{
                    for msg in msgs{
                        let nft_info={
                            let x=chain_apis::get_nfts_info_by_contract(Some(SEIRPCSERVER), &msg.collection, &vec![msg.nft_id.to_owned()]).await;
                            if x.is_err(){
                                eprintln!("{:#?}",&x.err().unwrap());
                                continue;
                            };
                            x.unwrap()[0].to_owned()
                        };
                        let update_sender_transaction=nova_db::update_wallet_nft_transactions(&msg.seller, &nft_data_struct::NftTransaction::AcceptBid(msg.clone()), &conn_pool).await;
                        let update_receive_transaction=nova_db::update_wallet_nft_transactions(&msg.recipient,  &nft_data_struct::NftTransaction::AcceptBid(msg.to_owned()),&conn_pool).await;
                        let update_sender_wallet_nft_hold=nova_db::update_wallet_nft_hold(&msg.seller, &msg.collection, &nft_info, "del", &conn_pool).await;
                        let update_receive_wallet_nft_hold=nova_db::update_wallet_nft_hold(&msg.recipient, &msg.collection, &nft_info, "add", &conn_pool).await;
                        
                        if update_sender_transaction.is_err(){
                            eprintln!("{:#?}",&update_sender_transaction.err().unwrap());
                        }else if update_receive_transaction.is_err() {
                            eprintln!("{:#?}",&update_receive_transaction.err().unwrap());
                        }else if update_sender_wallet_nft_hold.is_err() {
                            eprintln!("{:#?}",&update_sender_wallet_nft_hold.err().unwrap());
                        }else if update_receive_wallet_nft_hold.is_err() {
                            eprintln!("{:#?}",&update_receive_wallet_nft_hold.err().unwrap());
                        }else {
                            println!("[*] Save NFT AcceptBid data in DB Sucess");
                        }
                    }
                },
                TransactionEvent::NftFixedSell(msgs) => {
                    for msg in msgs{
                        let nft_info={
                            let x=chain_apis::get_nfts_info_by_contract(Some(SEIRPCSERVER), &msg.collection, &vec![msg.nft_id.to_owned()]).await;
                            if x.is_err(){
                                eprintln!("{:#?}",&x.err().unwrap());
                                continue;
                            };
                            x.unwrap()[0].to_owned()
                        };
                        let update_sender_transaction=nova_db::update_wallet_nft_transactions(&msg.sender, &nft_data_struct::NftTransaction::FixedSell(msg.clone()), &conn_pool).await;
                        let update_receive_transaction=nova_db::update_wallet_nft_transactions(&msg.recipient,  &nft_data_struct::NftTransaction::FixedSell(msg.to_owned()),&conn_pool).await;
                        let update_sender_wallet_nft_hold=nova_db::update_wallet_nft_hold(&msg.sender, &msg.collection, &nft_info, "del", &conn_pool).await;
                        let update_receive_wallet_nft_hold=nova_db::update_wallet_nft_hold(&msg.recipient, &msg.collection, &nft_info, "add", &conn_pool).await;
                        
                        if update_sender_transaction.is_err(){
                            eprintln!("{:#?}",&update_sender_transaction.err().unwrap());
                        }else if update_receive_transaction.is_err() {
                            eprintln!("{:#?}",&update_receive_transaction.err().unwrap());
                        }else if update_sender_wallet_nft_hold.is_err() {
                            eprintln!("{:#?}",&update_sender_wallet_nft_hold.err().unwrap());
                        }else if update_receive_wallet_nft_hold.is_err() {
                            eprintln!("{:#?}",&update_receive_wallet_nft_hold.err().unwrap());
                        }else {
                            println!("[*] Save NFT Fix sell data in DB Sucess");
                        }
                    }   
                },
                TransactionEvent::NftOnlyCreateAuction(msgs) =>{
                    for msg in msgs{
                        
                        let nft_floor_price=nova_db::tables::NftFloorPrice { 
                            nft_id:msg.nft_id.to_owned(), 
                            floor_price:msg.auction_price.to_owned(), 
                            ts: msg.ts.to_owned() };
                        let update_onlycreate_auction=nova_db::update_wallet_nft_transactions(msg.transaction_sender.to_owned().unwrap().as_str(), &nft_data_struct::NftTransaction::OnlyCreateAuction(msg.clone()), &conn_pool).await;
                        let update_nft_contraction=nova_db::update_nft_collection(&msg.collection, &conn_pool, nft_floor_price).await;
                        
                        if update_nft_contraction.is_err(){
                            eprintln!("{}",update_nft_contraction.err().unwrap())
                        }else if update_onlycreate_auction.is_err() {
                            println!("{}",update_onlycreate_auction.err().unwrap())
                        }else {
                            println!("[*] Save NFT OnlyCreateAuction and NFT Collection in DB Sucess");
                        }
                    }
                },
                TransactionEvent::TokenHeihtSwap(msgs) =>{
                    for msg in msgs{
                        let wallet_address=msg.clone().transaction_sender.unwrap();
                        let update_token_transaction=nova_db::update_wallet_token_transactions(&wallet_address, &token_data_struct::TokenTransaction::TokenSwap(msg), &conn_pool).await;
                        if update_token_transaction.is_err(){
                            eprintln!("{}",update_token_transaction.err().unwrap())
                        }else {
                            println!("[*] Save Token Swap in DB Sucess");
                        }
                    }
                },
                TransactionEvent::TokenNormalSwap(msgs) => {
                    for msg in msgs{
                        let wallet_address=msg.clone().transaction_sender.unwrap();
                        let update_token_transaction=nova_db::update_wallet_token_transactions(&wallet_address, &token_data_struct::TokenTransaction::TokenSwap(msg), &conn_pool).await;
                        if update_token_transaction.is_err(){
                            eprintln!("{}",update_token_transaction.err().unwrap())
                        }else {
                            println!("[*] Save Token Swap in DB Sucess");
                        }
                    }
                },
                TransactionEvent::TokenTransferByWei(msgs) =>{
                    for msg in msgs{
                        let sender=msg.sender.to_owned();
                        let reveiver=msg.receiver.to_owned();

                        let update_sender_token_transaction=nova_db::update_wallet_token_transactions(&sender, &token_data_struct::TokenTransaction::TokenTransfer(msg.to_owned()), &conn_pool).await;
                        let update_reveiver_token_transaction=nova_db::update_wallet_token_transactions(&reveiver, &token_data_struct::TokenTransaction::TokenTransfer(msg.to_owned()), &conn_pool).await;
                        
                        if update_reveiver_token_transaction.is_err(){
                            eprintln!("{}",update_reveiver_token_transaction.err().unwrap());
                        }else if update_sender_token_transaction.is_err() {
                            eprintln!("{}",update_sender_token_transaction.err().unwrap());
                        }else {
                            println!("[*] Save Token Transfer in DB Sucess");
                        }
                    }
                },
                TransactionEvent::TokenTransferByBank(msgs) => {
                    for msg in msgs{
                        let sender=msg.sender.to_owned();
                        let reveiver=msg.receiver.to_owned();

                        let update_sender_token_transaction=nova_db::update_wallet_token_transactions(&sender, &token_data_struct::TokenTransaction::TokenTransfer(msg.to_owned()), &conn_pool).await;
                        let update_reveiver_token_transaction=nova_db::update_wallet_token_transactions(&reveiver, &token_data_struct::TokenTransaction::TokenTransfer(msg.to_owned()), &conn_pool).await;
                        
                        if update_reveiver_token_transaction.is_err(){
                            eprintln!("{}",update_reveiver_token_transaction.err().unwrap());
                        }else if update_sender_token_transaction.is_err() {
                            eprintln!("{}",update_sender_token_transaction.err().unwrap());
                        }else {
                            println!("[*] Save Token Transfer in DB Sucess");
                        }
                    }
                },
                TransactionEvent::TokenTransferByContract(msgs) => {
                    for msg in msgs{
                        
                        let sender=msg.sender.to_owned();
                        let reveiver=msg.receiver.to_owned();

                        let update_sender_token_transaction=nova_db::update_wallet_token_transactions(&sender, &token_data_struct::TokenTransaction::ContractTokenTransfer(msg.to_owned()), &conn_pool).await;
                        let update_reveiver_token_transaction=nova_db::update_wallet_token_transactions(&reveiver, &token_data_struct::TokenTransaction::ContractTokenTransfer(msg.to_owned()), &conn_pool).await;
                        
                        if update_reveiver_token_transaction.is_err(){
                            eprintln!("{}",update_reveiver_token_transaction.err().unwrap());
                        }else if update_sender_token_transaction.is_err() {
                            eprintln!("{}",update_sender_token_transaction.err().unwrap());
                        }else {
                            println!("[*] Save Contract Token Transfer in DB Sucess");
                        }
                    }
                },
                TransactionEvent::Delegate(msgs) =>{
                    for msg in msgs{
                        let update_stake_transaction=nova_db::update_wallet_stake_transactions(&msg.delegator_address, &msg, &conn_pool).await;
                        if update_stake_transaction.is_err(){
                            eprintln!("{}",update_stake_transaction.err().unwrap());
                        }else {
                            println!("[*] Save Stake in DB Sucess");
                        }
                    }
                },
                TransactionEvent::Undelegate(msgs) =>{
                    for msg in msgs{
                        let update_stake_transaction=nova_db::update_wallet_stake_transactions(&msg.delegator_address, &msg, &conn_pool).await;
                        if update_stake_transaction.is_err(){
                            eprintln!("{}",update_stake_transaction.err().unwrap());
                        }else {
                            println!("[*] Save Stake in DB Sucess");
                        }
                    }
                },
                TransactionEvent::Unkonw =>{
                    println!("Unkonw Transaction");
                },
            }
        }
    }
    Ok(())
}