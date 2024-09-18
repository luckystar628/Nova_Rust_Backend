use std::sync::Arc;

use reqwest::Client;
use sei_client::{chain_apis, data_feild_structs::{nft_data_struct, token_data_struct}, data_rp_structs::tx_rp_struct, transaction_sort::{Transaction, TransactionEvent}};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use anyhow::{anyhow, Result};
use tokio::sync::Semaphore;



#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub struct SeiScanTransaction{
    pub created:String,
    pub hash:String,
        height:Value,
        is_clear_admin:Value,
        is_execute:Value,
        is_ibc:Value,
        is_instantiate:Value,
        is_migrate:Value,
        is_send:Value,
        is_signer:Value,
        is_store_code:Value,
        is_update_admin:Value,
        messages:Value,
        sender:Value,
    pub success:bool,
}



pub async fn take<'services>(
    wallet_address:&'services str,
    conn_pool:&'services PgPool,
) -> Result<()> {

    let client=Client::new();

    let get_transaction_count_url=format!("https://celatone-api-prod.alleslabs.dev/v1/sei/pacific-1/accounts/{}/txs-count?is_send=false&is_ibc=false&is_instantiate=false&is_store_code=false&is_execute=false&is_migrate=false&is_update_admin=false&is_clear_admin=false&is_move_publish=false&is_move_upgrade=false&is_move_execute=false&is_move_script=false&is_opinit=false&is_wasm=true",wallet_address);

    let data_rp:Value=client.get(get_transaction_count_url)
                            .send()
                            .await?
                            .json()
                            .await?;
    

    let wallet_transaction_count={
        let x = data_rp.get("count");
        if x.is_none(){
            return Err(anyhow!("Don't have transaction count"));
        };
        x.unwrap().as_u64().unwrap()
    };

    let get_transactions_url=format!("https://celatone-api-prod.alleslabs.dev/v1/sei/pacific-1/accounts/{}/txs?limit={}&offset=0&is_wasm=true&is_move=false&is_initia=false&is_send=false&is_ibc=false&is_instantiate=false&is_store_code=false&is_execute=false&is_migrate=false&is_update_admin=false&is_clear_admin=false&is_move_publish=false&is_move_upgrade=false&is_move_execute=false&is_move_script=false&is_opinit=false",wallet_address,wallet_transaction_count);

    let data_rp:Value=client.get(get_transactions_url)
                            .send()
                            .await?
                            .json()
                            .await?;

    
    let wallet_transactions=Box::new({
        let items=data_rp.get("items");
        if items.is_none(){
            return Err(anyhow!("The wallet don't have transactions"));
        }
        serde_json::from_value::<Vec<SeiScanTransaction>>(items.unwrap().to_owned()).unwrap()
    });

    let mut wallet_transaction_hashs:Vec<String>=Vec::new();
    wallet_transactions.iter().for_each(|t|{
        if t.success{
            wallet_transaction_hashs.push(t.hash.get(2..).unwrap().to_uppercase())
        }
    });


    // 限制并发 16
    let semaphore=Arc::new(Semaphore::new(16));
    let conn_pool=Arc::new(conn_pool.to_owned());
    
    let mut hanldes:Vec<tokio::task::JoinHandle<()>>=Vec::new();

    for hash in wallet_transaction_hashs{

        let semaphore=Arc::clone(&semaphore);
        let conn_pool=Arc::clone(&conn_pool);

        let handle=tokio::spawn(async move {
            let data=chain_apis::get_transaction_by_tx(None, &hash).await;
            if data.is_ok(){
                let transaction_data=data.unwrap();
                let _=save_to_db(transaction_data, &conn_pool).await;
            }
        });
        
        hanldes.push(handle);
    }

    futures::future::join_all(hanldes).await;


    Ok(())
}



async fn save_to_db(
    transaction:tx_rp_struct::TransactionData,
    conn_pool:&PgPool
) -> Result<()> {

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
                        
                        let nft_info_res=chain_apis::get_nfts_info_by_contract(Some(chain_apis::SERVER_RPC), &collection_address, &vec![msg.nft_id.to_owned()]).await;
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
                            let x=chain_apis::get_nfts_info_by_contract(Some(chain_apis::SERVER_RPC), &msg.collection, &vec![msg.nft_id.to_owned()]).await;
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
                            let x=chain_apis::get_nfts_info_by_contract(Some(chain_apis::SERVER_RPC), &msg.collection, &vec![msg.nft_id.to_owned()]).await;
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
                            let x=chain_apis::get_nfts_info_by_contract(Some(chain_apis::SERVER_RPC), &msg.collection, &vec![msg.nft_id.to_owned()]).await;
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
                            let x=chain_apis::get_nfts_info_by_contract(Some(chain_apis::SERVER_RPC), &msg.collection, &vec![msg.nft_id.to_owned()]).await;
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
                            let x=chain_apis::get_nfts_info_by_contract(Some(chain_apis::SERVER_RPC), &msg.collection, &vec![msg.nft_id.to_owned()]).await;
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
                            let x=chain_apis::get_nfts_info_by_contract(Some(chain_apis::SERVER_RPC), &msg.collection, &vec![msg.nft_id.to_owned()]).await;
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
                            let x=chain_apis::get_nfts_info_by_contract(Some(chain_apis::SERVER_RPC), &msg.collection, &vec![msg.nft_id.to_owned()]).await;
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
                        
   
                        if update_onlycreate_auction.is_err() {
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
    

    Ok(())
}