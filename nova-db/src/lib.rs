pub mod tables;

use std::env;
use chrono::{DateTime, Utc};
use dotenv::dotenv;
use anyhow::Result;
use sei_client::{chain_apis, data_feild_structs::{nft_data_struct, stake_data_sturct, token_data_struct}, error::NovaDBErrs};
use sqlx::{postgres::{PgPoolOptions, PgQueryResult}, PgPool, Pool, Postgres,Row};
use tables::{NftContract, WalletInfo};

pub async fn create_db_conn_pool() -> Result<Pool<Postgres>> {
    
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").unwrap();
    let db_conn_pool=PgPoolOptions::new()
                    .max_connections(2000)
                    .idle_timeout(std::time::Duration::from_secs(30))
                    .connect(&db_url).await?;
    Ok(db_conn_pool)
}

pub async fn find_wallet_info<'nova_db>(
    wallet_address:&'nova_db str,
    conn_pool:&'nova_db PgPool
) -> Result<WalletInfo> {
    
    let query=r#"SELECT wallet_address, nft_hold, nft_transactions, token_transactions ,stake_transactions FROM "WalletInfos" WHERE wallet_address = $1"#;

    if let Ok(wallet_info) = sqlx::query_as::<_,WalletInfo>(query)
                                                        .bind(wallet_address)
                                                        .fetch_one(conn_pool).await{
        Ok(wallet_info)
    }else {
        Err(NovaDBErrs::UnfindWallet.into())
    }
}

pub async fn insert_new_wallet<'nova_db>(
    wallet_info:&'nova_db WalletInfo,
    conn_pool:&'nova_db PgPool
)->Result<PgQueryResult>{
    
    let query=r#"INSERT INTO "WalletInfos" (wallet_address, nft_hold, nft_transactions, token_transactions,stake_transactions) VALUES ($1, $2, $3, $4,$5)"#;

    let inster_event=sqlx::query::<sqlx::Postgres>(query)
                                                    .bind(&wallet_info.wallet_address)
                                                    .bind(serde_json::to_value(&wallet_info.nft_hold).unwrap())
                                                    .bind(serde_json::to_value(&wallet_info.nft_transactions).unwrap())
                                                    .bind(serde_json::to_value(&wallet_info.token_transactions).unwrap())
                                                    .bind(serde_json::to_value(&wallet_info.stake_transactions).unwrap())
                                                    .execute(conn_pool).await;
    if let Ok(result) = inster_event {
            Ok(result)
        }else {
            Err(NovaDBErrs::InsterNewWalletErr.into())
        }
}

pub async fn update_wallet_nft_hold<'nova_db>(
    wallet_address:&'nova_db str,
    collect_address:&'nova_db str,
    nft_info:&'nova_db nft_data_struct::NftInfo,
    operationl:&'nova_db str,
    conn_pool:&'nova_db PgPool
)->Result<PgQueryResult>{
    let find_wallet_nft_hold_query=r#"SELECT nft_hold FROM "WalletInfos" WHERE wallet_address = $1"#;
    let update_wallet_nft_hold_query=r#"UPDATE "WalletInfos" SET nft_hold =$2 WHERE wallet_address = $1 "#;

    match sqlx::query(find_wallet_nft_hold_query).bind(wallet_address).fetch_one( conn_pool).await{
        Ok(row)=>{
            let mut nft_hold=serde_json::from_value::<Vec<nft_data_struct::NftCollectHold>>(row.get("nft_hold")).unwrap();
            
            match operationl {
                "add"=>{
                    if nft_hold.iter().any(|nft_collect|{nft_collect.collect_address==collect_address}){
                        
                        
                        nft_hold.iter_mut().for_each(|c|{
                            if c.collect_address==collect_address{
                                if !c.nfts_hold.iter().any(|nft| nft.key == nft_info.key){
                                c.nfts_hold.push(nft_info.to_owned())
                            }
                        }
                        });
                    }else {
                        let collect_info=chain_apis::get_nft_collect_info_by_contract(None, &collect_address).await?;
                        let  nft_collect_hold=nft_data_struct::NftCollectHold{
                            collect_address:collect_address.to_owned(),
                            collect_info,
                            nfts_hold:vec![nft_info.to_owned()]
                        };
                        nft_hold.push(nft_collect_hold)
                    }
                },
                "del"=>{
                    if nft_hold.iter().any(|nft_collect|{nft_collect.collect_address==collect_address}){
                        nft_hold.iter_mut().for_each(|collect|{
                            if collect.collect_address==collect_address{
                                collect.nfts_hold.retain(|nft| nft.key!=nft_info.key)
                            }
                        })
                    }
                },
                _=> return Err(NovaDBErrs::UpdateWalletNFtHoldOperationlErr.into())
            }
            
            let update_nft_hold_event=sqlx::query(update_wallet_nft_hold_query)
                                                                    .bind(wallet_address)
                                                                    .bind(serde_json::to_value(nft_hold).unwrap())
                                                                    .execute(conn_pool)
                                                                    .await;
            if update_nft_hold_event.is_ok(){
                Ok(update_nft_hold_event.unwrap())
            }else {
                Err(NovaDBErrs::UpdateWalletNftHoldErr.into())
            }
        },
        Err(err)=>{
            match err {
                sqlx::Error::RowNotFound=>{
                    if operationl=="add"{
                        let collect_info=chain_apis::get_nft_collect_info_by_contract(None, &collect_address).await?;
                        let  nft_collect_hold=nft_data_struct::NftCollectHold{
                            collect_address:collect_address.to_owned(),
                            collect_info,
                            nfts_hold:vec![nft_info.to_owned()]
                        };
                        let wallet_info=WalletInfo{
                            wallet_address:wallet_address.to_owned(),
                            nft_hold:vec![
                                nft_collect_hold
                            ],
                            nft_transactions:Vec::new(),
                            token_transactions:Vec::new(),
                            stake_transactions:Vec::new(),
                        };
                        if let Ok(insert_event)=insert_new_wallet(&wallet_info,conn_pool).await{
                            Ok(insert_event)
                        }else {
                            Err(NovaDBErrs::UpdateWalletNftHoldErr.into())
                        }
                    }else {
                        Ok(PgQueryResult::default())
                    }
                },
                _=>{
                    Err(NovaDBErrs::UpdateWalletNftHoldErr.into())
                }
            }
        }
    }
}

pub async fn update_wallet_nft_transactions<'nova_db>(
    wallet_address:&'nova_db str,
    nft_transaction:&'nova_db nft_data_struct::NftTransaction,
    conn_pool:&'nova_db PgPool
) -> Result<PgQueryResult> {
    
    let find_wallet_nft_transactions_query=r#"SELECT nft_transactions FROM "WalletInfos" WHERE wallet_address = $1"#;
    let updat_wallet_nft_transactions_query=r#"UPDATE "WalletInfos" SET nft_transactions =$2 WHERE wallet_address = $1 "#;
    
    match sqlx::query(&find_wallet_nft_transactions_query).bind(wallet_address).fetch_one(conn_pool).await{
        Ok(row)=>{
            let mut nft_transactions=serde_json::from_value::<Vec<nft_data_struct::NftTransaction>>(row.get("nft_transactions")).unwrap();
            if !nft_transactions.iter().any(|t|{
                match (t,nft_transaction) {
                    (nft_data_struct::NftTransaction::Mint(t1), nft_data_struct::NftTransaction::Mint(t2)) =>t1.tx==t2.tx &&t1.ts==t2.ts ,
                    (nft_data_struct::NftTransaction::BatchBids(t1), nft_data_struct::NftTransaction::BatchBids(t2)) =>t1.tx==t2.tx &&t1.ts==t2.ts ,
                    (nft_data_struct::NftTransaction::OnlyCreateAuction(t1), nft_data_struct::NftTransaction::OnlyCreateAuction(t2)) => t1.tx==t2.tx &&t1.ts==t2.ts ,
                    (nft_data_struct::NftTransaction::Transfer(t1), nft_data_struct::NftTransaction::Transfer(t2)) => t1.tx==t2.tx &&t1.ts==t2.ts , 
                    (nft_data_struct::NftTransaction::FixedSell(t1), nft_data_struct::NftTransaction::FixedSell(t2)) =>t1.tx==t2.tx &&t1.ts==t2.ts ,
                    (nft_data_struct::NftTransaction::PurchaseCart(t1), nft_data_struct::NftTransaction::PurchaseCart(t2)) =>t1.tx==t2.tx &&t1.ts==t2.ts ,
                    (nft_data_struct::NftTransaction::AcceptBid(t1), nft_data_struct::NftTransaction::AcceptBid(t2)) => t1.tx==t2.tx &&t1.ts==t2.ts ,
                    (nft_data_struct::NftTransaction::CreateAuction(t1), nft_data_struct::NftTransaction::CreateAuction(t2)) =>t1.tx==t2.tx &&t1.ts==t2.ts ,
                    (nft_data_struct::NftTransaction::CancelAuction(t1), nft_data_struct::NftTransaction::CancelAuction(t2)) => t1.tx==t2.tx &&t1.ts==t2.ts ,
                    _=>true,
                }
            }){
                nft_transactions.push(nft_transaction.to_owned());
            }
            let update_nft_transaction_event=sqlx::query(updat_wallet_nft_transactions_query)
                                                                                .bind(wallet_address)
                                                                                .bind(serde_json::to_value(nft_transactions).unwrap())
                                                                                .execute(conn_pool)
                                                                                .await;
            if update_nft_transaction_event.is_ok(){
                return Ok(update_nft_transaction_event.unwrap()) ;
            }else {
                return Err(NovaDBErrs::UpdateWalletNftTransactionsErr.into());
            }
        },
        Err(err)=>{
            match err {
                sqlx::Error::RowNotFound=>{
                    let wallet_info=WalletInfo{
                        wallet_address:wallet_address.to_owned(),
                        nft_hold:Vec::new(),
                        nft_transactions:vec![nft_transaction.to_owned()],
                        token_transactions:Vec::new(),
                        stake_transactions:Vec::new(),
                    };
                    if let Ok(insert_event)=insert_new_wallet(&wallet_info, conn_pool).await{
                        return Ok(insert_event);
                    }else {
                        return Err(NovaDBErrs::UpdateWalletNftTransactionsErr.into());
                    }
                },
                _=>{
                    return Err(NovaDBErrs::UpdateWalletNftTransactionsErr.into());
                }
            }
        }
    }
}

pub async fn update_wallet_token_transactions<'nova_db>(
    wallet_address:&'nova_db str,
    token_transaction:&'nova_db token_data_struct::TokenTransaction,
    conn_pool:&'nova_db PgPool
) -> Result<PgQueryResult> {
    let find_wallet_token_transactions_query=r#"SELECT token_transactions FROM "WalletInfos" WHERE wallet_address = $1"#;
    let updat_wallet_token_transactions_query=r#"UPDATE "WalletInfos" SET token_transactions =$2 WHERE wallet_address = $1 "#;
    
    match sqlx::query(&find_wallet_token_transactions_query).bind(wallet_address).fetch_one(conn_pool).await{
        Ok(row)=>{
            let mut token_transactions=serde_json::from_value::<Vec<token_data_struct::TokenTransaction>>(row.get("token_transactions")).unwrap();
            
            if !token_transactions.iter().any(|t|{
                match (t,token_transaction) {
                    (token_data_struct::TokenTransaction::TokenSwap(t1), token_data_struct::TokenTransaction::TokenSwap(t2)) =>t1.tx==t2.tx &&t1.ts==t2.ts,
                    (token_data_struct::TokenTransaction::TokenTransfer(t1), token_data_struct::TokenTransaction::TokenTransfer(t2)) => t1.tx==t2.tx &&t1.ts==t2.ts ,
                    (token_data_struct::TokenTransaction::ContractTokenTransfer(t1), token_data_struct::TokenTransaction::ContractTokenTransfer(t2)) => t1.tx==t2.tx &&t1.ts==t2.ts ,
                    _=>true
                }
            }){
                token_transactions.push(token_transaction.to_owned());
            }
            
            let update_token_transaction_event=sqlx::query(updat_wallet_token_transactions_query)
                                                                                .bind(wallet_address)
                                                                                .bind(serde_json::to_value(token_transactions).unwrap())
                                                                                .execute(conn_pool)
                                                                                .await;
            if update_token_transaction_event.is_ok(){
                return Ok(update_token_transaction_event.unwrap()) ;
            }else {
                return Err(NovaDBErrs::UpdateWalletTokenTransactionsErr.into());
            }
        },
        Err(err)=>{
            match err {
                sqlx::Error::RowNotFound=>{
                    let wallet_info=WalletInfo{
                        wallet_address:wallet_address.to_owned(),
                        nft_hold:Vec::new(),
                        nft_transactions:Vec::new(),
                        token_transactions:vec![token_transaction.to_owned()],
                        stake_transactions:Vec::new(),
                    };
                    if let Ok(insert_event)=insert_new_wallet(&wallet_info, conn_pool).await{
                        return Ok(insert_event);
                    }else {
                        return Err(NovaDBErrs::UpdateWalletTokenTransactionsErr.into());
                    }
                },
                _=>{
                    return Err(NovaDBErrs::UpdateWalletTokenTransactionsErr.into());
                }
            }
        }
    }
}

pub async fn update_wallet_stake_transactions<'nova_db>(
    wallet_address:&'nova_db str,
    stake_transaction:&'nova_db stake_data_sturct::Stake,
    conn_pool:&'nova_db PgPool
) -> Result<PgQueryResult> {

    let find_wallet_stake_transactions_query=r#"SELECT stake_transactions FROM "WalletInfos" WHERE wallet_address = $1"#;
    let updat_wallet_stake_transactions_query=r#"UPDATE "WalletInfos" SET stake_transactions =$2 WHERE wallet_address = $1"#;

    match sqlx::query(find_wallet_stake_transactions_query).bind(wallet_address).fetch_one(conn_pool).await{
        Ok(row)=>{
            let mut stake_transactions=serde_json::from_value::<Vec<stake_data_sturct::Stake>>(row.get("stake_transactions")).unwrap();

            if !stake_transactions.iter().any(|t|{
                t._type==stake_transaction._type && 
                t.amount==stake_transaction.amount &&
                t.delegator_address==stake_transaction.delegator_address &&
                t.ts==stake_transaction.ts &&
                t.tx==stake_transaction.tx &&
                t.transaction_sender == stake_transaction.transaction_sender
            }){
                stake_transactions.push(stake_transaction.to_owned());
            }

            let update_stake_transaction_event=sqlx::query(updat_wallet_stake_transactions_query)
                                                                                .bind(wallet_address)
                                                                                .bind(serde_json::to_value(stake_transactions).unwrap())
                                                                                .execute(conn_pool)
                                                                                .await;
            if update_stake_transaction_event.is_ok(){
                return Ok(update_stake_transaction_event.unwrap()) ;
            }else {
                return Err(NovaDBErrs::UpdateWalletStakeTransactionErr.into());
            }
        },
        Err(err)=>{
            match err {
                sqlx::Error::RowNotFound=>{
                    let wallet_info=WalletInfo{
                        wallet_address:wallet_address.to_owned(),
                        nft_hold:Vec::new(),
                        nft_transactions:Vec::new(),
                        token_transactions:Vec::new(),
                        stake_transactions:vec![stake_transaction.to_owned()],
                    };
                    if let Ok(insert_event)=insert_new_wallet(&wallet_info, conn_pool).await{
                        return Ok(insert_event);
                    }else {
                        return Err(NovaDBErrs::UpdateWalletStakeTransactionErr.into());
                    }
                },
                _=>{
                    return Err(NovaDBErrs::UpdateWalletStakeTransactionErr.into());
                }
            }
        }
    }
}

pub async fn find_nft_collection<'nova_db>(
    collect_address:&'nova_db str,
    conn_pool:&'nova_db PgPool,
) -> Result<NftContract> {
    let query=r#"SELECT collection_address, collection_floor_price, nfts_floor_price FROM "NFTContracts" WHERE collection_address = $1"#;
    if let Ok(nft_contract) = sqlx::query_as::<_,NftContract>(query)
                                   .bind(collect_address)
                                   .fetch_one(conn_pool).await{
        Ok(nft_contract)
    }else {
        Err(NovaDBErrs::UnfindNFTContract.into())
    }
}





pub async fn insert_new_nft_collection<'nova_db>(
    collect_address:&'nova_db str,
    collection_floor_price:Option<tables::CollectionFloorPrice>,
    nfts_floor_price:Option<Vec<tables::NftFloorPrice>>,
    conn_pool:&'nova_db PgPool,
) -> Result<PgQueryResult> {
    let query=r#"INSERT INTO "NFTContracts" (collection_address, collection_floor_price, nfts_floor_price) VALUES ($1, $2, $3)"#;

    if collection_floor_price.is_some() && nfts_floor_price.is_some() {
        let inster_event=sqlx::query::<sqlx::Postgres>(query)
                                                            .bind(collect_address)
                                                            .bind(serde_json::to_value(collection_floor_price.unwrap()).unwrap())
                                                            .bind(serde_json::to_value(nfts_floor_price.unwrap()).unwrap())
                                                            .execute(conn_pool)
                                                            .await;
        if inster_event.is_ok(){
            return Ok(inster_event.unwrap());
        }
    }else if nfts_floor_price.is_some() {
            let collection_floor_price=tables::CollectionFloorPrice{
                floor_price:nfts_floor_price.clone().unwrap()[0].floor_price.clone(),
                ts:nfts_floor_price.clone().unwrap()[0].ts.clone(),
            };
            let inster_event=sqlx::query::<sqlx::Postgres>(query)
                                                            .bind(collect_address)
                                                            .bind(serde_json::to_value(collection_floor_price).unwrap())
                                                            .bind(serde_json::to_value(nfts_floor_price.unwrap()).unwrap())
                                                            .execute(conn_pool)
                                                            .await;
        if inster_event.is_ok(){
            return Ok(inster_event.unwrap());
        }
    }else if collection_floor_price.is_some(){
        let inster_event=sqlx::query::<sqlx::Postgres>(query)
                                                           .bind(collect_address)
                                                           .bind(serde_json::to_value(collection_floor_price.unwrap()).unwrap())
                                                           .bind(serde_json::to_value(serde_json::json!({})).unwrap())
                                                           .execute(conn_pool)
                                                           .await;
       if inster_event.is_ok(){
           return Ok(inster_event.unwrap());
       }
    }else {
        let inster_event=sqlx::query::<sqlx::Postgres>(query)
                                                            .bind(collect_address)
                                                            .bind(serde_json::to_value(serde_json::json!({})).unwrap())
                                                            .bind(serde_json::to_value(serde_json::json!({})).unwrap())
                                                            .execute(conn_pool)
                                                            .await;
        if inster_event.is_ok(){
            return Ok(inster_event.unwrap());
        }
    }

    Err(NovaDBErrs::InsterNewNFTContractErr.into())
  
}

pub async fn update_nft_collection<'nova_db>(
    collect_address:&'nova_db str,
    conn_pool:&'nova_db PgPool,
    nft_floor_price:tables::NftFloorPrice,
)->Result<PgQueryResult>{
    let query1=r#"UPDATE "NFTContracts" SET nfts_floor_price =$2 , collection_floor_price= $3 WHERE collection_address = $1 "#;
    let query2=r#"UPDATE "NFTContracts" SET nfts_floor_price =$2 WHERE collection_address = $1 "#;

    let day_now=Utc::now().date_naive();
    
    // 排除非今天的的nft
    if DateTime::parse_from_rfc3339(&nft_floor_price.ts).unwrap().with_timezone(&Utc).date_naive() == day_now{
        return Err(NovaDBErrs::NftFloorPriceNotToday.into());
    };

    let nft_collection={

        // if don't have collection , inster new
        let x=find_nft_collection(collect_address, conn_pool).await;
        if x.is_err(){
            let collection_floor_price=tables::CollectionFloorPrice{
                floor_price:nft_floor_price.floor_price.clone(),
                ts:nft_floor_price.ts.clone()
            };
            let inster_event=insert_new_nft_collection(collect_address, Some(collection_floor_price), Some(vec![nft_floor_price]), conn_pool).await;
            if inster_event.is_err(){
                return Err(NovaDBErrs::UnfindNFTContract.into());
            }else {
                return Ok(inster_event.unwrap());
            }
        };
        x.unwrap()
    };

    
    if nft_collection.collection_floor_price.is_none()  || 
        (nft_collection.collection_floor_price.is_some() && 
        nft_collection.collection_floor_price.clone().unwrap().floor_price.get(..nft_collection.collection_floor_price.clone().unwrap().floor_price.to_owned().len()-4).unwrap().parse::<u64>().unwrap() >
        nft_floor_price.floor_price.get(..nft_floor_price.floor_price.len()-4).unwrap().parse::<u64>().unwrap()  &&
        DateTime::parse_from_rfc3339(&nft_floor_price.ts).unwrap().with_timezone(&Utc).date_naive() >= DateTime::parse_from_rfc3339(&nft_collection.collection_floor_price.unwrap().ts).unwrap().with_timezone(&Utc).date_naive()
    ){  
        let new_collection_floor_price=tables::CollectionFloorPrice{
            floor_price:nft_floor_price.floor_price.clone(),
            ts:nft_floor_price.ts.clone()
        };
   
        let mut nfts_floor_price=nft_collection.nfts_floor_price;

        if nfts_floor_price.iter().any(|x| x.nft_id==nft_floor_price.nft_id){
            nfts_floor_price.iter_mut().for_each(|x| {
                if x.nft_id==nft_floor_price.nft_id &&   x.floor_price.get(..x.floor_price.len()-4).unwrap().parse::<u64>().unwrap()>nft_floor_price.floor_price.get(0..nft_floor_price.floor_price.len()-4).unwrap().parse::<u64>().unwrap(){
                    x.floor_price=nft_floor_price.floor_price.clone();
                }
            });
        }else {
            nfts_floor_price.push(tables::NftFloorPrice { nft_id:nft_floor_price.nft_id.clone(), floor_price: nft_floor_price.floor_price.clone(), ts: nft_floor_price.ts.clone() });
        }

        let update_event=sqlx::query(&query1)
                        .bind(collect_address)
                        .bind(serde_json::to_value(nfts_floor_price).unwrap())
                        .bind(serde_json::to_value(new_collection_floor_price).unwrap())
                        .execute(conn_pool).await;
        if update_event.is_err(){
            return Err(NovaDBErrs::UpdateNFTCollectionErr.into());
        }else {
            return Ok(update_event.unwrap());
        }
    }else {
        let mut nfts_floor_price=nft_collection.nfts_floor_price;

        if nfts_floor_price.iter().any(|x| x.nft_id==nft_floor_price.nft_id
        ){

            nfts_floor_price.iter_mut().for_each(|x| {
                if x.nft_id==nft_floor_price.nft_id && x.floor_price.get(..x.floor_price.len()-4).unwrap().parse::<u64>().unwrap()>nft_floor_price.floor_price.get(0..nft_floor_price.floor_price.len()-4).unwrap().parse::<u64>().unwrap(){
                        x.floor_price=nft_floor_price.floor_price.clone();
                }
            });
        }else {
            nfts_floor_price.push(tables::NftFloorPrice { nft_id:nft_floor_price.nft_id.clone(), floor_price: nft_floor_price.floor_price.clone(), ts: nft_floor_price.ts.clone() });
        }


        let update_event=sqlx::query(query2).bind(collect_address).bind(serde_json::to_value(nfts_floor_price).unwrap()).execute(conn_pool).await;
        if update_event.is_err(){
            return Err(NovaDBErrs::UpdateNFTCollectionErr.into());
        }else {
            return Ok(update_event.unwrap());
        }
    }

}



#[cfg(test)]
mod tests{
    use super::*;
    use anyhow::Ok;
    use sei_client::{data_feild_structs::{nft_data_struct, stake_data_sturct, token_data_struct::{self, TokenTransfer}}, data_rp_structs::{nft_collect_contract_rp_struct, tx_rp_struct}};
   
    #[tokio::test]
    async fn test_find_wallet_info() -> Result<()> {
        let  conn=create_db_conn_pool().await.unwrap();
        let data=find_wallet_info("sei1krvjk3r790dcsqkr96ymd44v04w9zz5dlr66z7", &conn).await;
        println!("{:#?}",data);
        Ok(())
    }

    #[tokio::test]
    async fn test_inster_new_wallet(

    ) -> Result<()> {
        let conn=create_db_conn_pool().await.unwrap();
        let wallet_address="sei1krvjk3r790dcsqkr96ymd44v04w9zz5dlr66z7".to_string();
        let collect_info=nft_collect_contract_rp_struct::NftCollectionInfo{
            name:"11111111111111111111111111111".to_string(),
            symbol:"11111111111111111111111111111".to_string(),
            creator:"11111111111111111111111111111".to_string(),
            nft_nums:"11111111111111111111111111111".to_string(),
        };
        let nft_hold=vec![
            nft_data_struct::NftCollectHold{
                collect_address:"11111111111111111111111111111".to_string(),
                collect_info:collect_info,
                nfts_hold:vec![
                    nft_data_struct::NftInfo{
                        token_id:"11111111111111111111111111111".to_string(),
                        name:"11111111111111111111111111111".to_string(),
                        key:"11111111111111111111111111111".to_string(),
                        image:"11111111111111111111111111111".to_string(),
                        royalty_percentage:25,
                        attributes:vec![
                            nft_data_struct::NftAttribute{
                                trait_type:"11111111111111111111111111111".to_string(),
                                value:"11111111111111111111111111111".to_string(),
                            }
                        ]
                    }
                ]
            }
        ];
        let nft_transaction=vec![
            nft_data_struct::NftTransaction::Transfer(
                nft_data_struct::Transfer { 
                    collection: "11111111111111111111111111111".to_string(), 
                    sender: "11111111111111111111111111111".to_string(),
                    recipient: "11111111111111111111111111111".to_string(),
                    nft_id: "11111111111111111111111111111".to_string(),
                    transaction_sender: Some("11111111111111111111111111111".to_string()),
                    fee: vec![tx_rp_struct::FeeAmount{
                        amount:"11111111111111111111111111111".to_string(),
                        denom:"11111111111111111111111111111".to_string(),
                    }],
                    ts: "11111111111111111111111111111".to_string(),
                    tx: "11111111111111111111111111111".to_string(), 
                }
            )
        ];
        let token_transactions=vec![
            token_data_struct::TokenTransaction::TokenTransfer(TokenTransfer{
                sender:"11111111111111111111111111111".to_string(),
                receiver:"11111111111111111111111111111".to_string(),
                amount:"11111111111111111111111111111".to_string(),
                transaction_sender:Some("11111111111111111111111111111".to_string()),
                fee: vec![tx_rp_struct::FeeAmount{
                    amount:"11111111111111111111111111111".to_string(),
                    denom:"11111111111111111111111111111".to_string(),
                }],
                ts: "11111111111111111111111111111".to_string(),
                tx: "11111111111111111111111111111".to_string(), 
            })
        ];

        let stake_transactions=vec![
            stake_data_sturct::Stake{
                validator_address:"11111111111111111111111111111".to_string(),
                delegator_address:"11111111111111111111111111111".to_string(),
                amount:"11111111111111111111111111111".to_string(),
                _type:stake_data_sturct::StakeType::Delegate,
                transaction_sender: Some("11111111111111111111111111111".to_string()),
                fee: vec![tx_rp_struct::FeeAmount{
                    amount:"11111111111111111111111111111".to_string(),
                    denom:"11111111111111111111111111111".to_string(),
                }],
                ts: "11111111111111111111111111111".to_string(),
                tx: "11111111111111111111111111111".to_string(), 
            }
        ];

        let inster_event=insert_new_wallet(
            &WalletInfo{
                wallet_address:wallet_address,
                nft_hold:nft_hold,
                nft_transactions:nft_transaction,
                token_transactions:token_transactions,
                stake_transactions:stake_transactions
            },
            &conn
        ).await;
        println!("{:#?}",inster_event);
        Ok(())
    }

    #[tokio::test]
    async fn test_update_wallet_nft_hold() -> Result<()> {
        let mut conn=create_db_conn_pool().await.unwrap();
        let wallet_address="sei1krvjk3r790dcsqkr96ymd44v04w9zz5dlr66z7".to_string();
        let collect_info=nft_collect_contract_rp_struct::NftCollectionInfo{
            name:"1".to_string(),
            symbol:"1".to_string(),
            creator:"1".to_string(),
            nft_nums:"1".to_string(),
        };
        let nft_hold=
            nft_data_struct::NftCollectHold{
                collect_address:"11111111111111111111111111111".to_string(),
                collect_info:collect_info,
                nfts_hold:vec![
                    nft_data_struct::NftInfo{
                        token_id:"11111111111111111111111111111".to_string(),
                        name:"11111111111111111111111111111".to_string(),
                        key:"11111111111111111111111111111".to_string(),
                        image:"11111111111111111111111111111".to_string(),
                        royalty_percentage:25,
                        attributes:vec![
                            nft_data_struct::NftAttribute{
                                trait_type:"11111111111111111111111111111".to_string(),
                                value:"11111111111111111111111111111".to_string(),
                            }
                        ]
                    }
                ]
            };
        
        let nft_info=nft_data_struct::NftInfo{
            token_id:"x".to_string(),
            name:"1".to_string(),
            key:"121".to_string(),
            image:"1".to_string(),
            royalty_percentage:25,
            attributes:vec![
                nft_data_struct::NftAttribute{
                    trait_type:"1".to_string(),
                    value:"1".to_string(),
                }
            ]
        };
        let data=update_wallet_nft_hold(
            &wallet_address,
            "sei1ts53rl9eqrdjd82hs2em7hv8g6em4xye67z9wxnhdrn4lnf8649sxtww22",
            &nft_info,"add" ,&conn).await;
        println!("{:#?}",data);
        Ok(())
    }

    #[tokio::test]
    async fn test_update_wallet_nft_transactions() -> Result<()> {
        let conn_pool=create_db_conn_pool().await.unwrap();
        let wallet_address="sei1krvjk3r790dcsqkr96ymd44v04w9zz5dlr66z7".to_string();
        let nft_transaction=nft_data_struct::NftTransaction::Mint(nft_data_struct::Mint{
            collection:"111".to_string(),
            recipient:"1".to_string(),
            nft_id:"1".to_string(),
            price:None,
            transaction_sender:None,
            fee:vec![],
            ts:"1111".to_string(),
            tx:"1111".to_string()
        });
        let data=update_wallet_nft_transactions(&wallet_address, &nft_transaction, &conn_pool).await;
        println!("{:#?}",data);
        Ok(())
    }


    #[tokio::test]
    async fn test_update_wallet_token_transactions() -> Result<()> {

        let conn_pool=create_db_conn_pool().await.unwrap();
        let wallet_address="sei1krvjk3r790dcsqkr96ymd44v04w9zz5dlr66z7".to_string();
        let token_transaction=token_data_struct::TokenTransaction::TokenTransfer(token_data_struct::TokenTransfer{
            sender:"1".to_string(),
            receiver:"12".to_string(),
            amount:"1".to_string(),
            fee:vec![],
            transaction_sender:None,
            ts:"1".to_string(),
            tx:"1".to_string()
        });
        let data=update_wallet_token_transactions(&wallet_address, &token_transaction, &conn_pool).await;
        println!("{:#?}",data);
        Ok(())
    }

    #[tokio::test]
    async fn test_update_wallet_stake_transaction() -> Result<()> {
        let conn_pool=create_db_conn_pool().await.unwrap();
        let wallet_address="sei1krvjk3r790dcsqkr96ymd44v04w9zz5dlr66z".to_string();
        let stake_transaction=stake_data_sturct::Stake{
            validator_address:"1".to_string(),
            delegator_address:"1".to_string(),
            amount:"1".to_string(),
            _type:stake_data_sturct::StakeType::Delegate,
            transaction_sender:None,
            fee:vec![],
            ts:"1".to_string(),
            tx:"1".to_string()
        };
        let data=update_wallet_stake_transactions(&wallet_address, &stake_transaction, &conn_pool).await;
        println!("{:#?}",data);
        Ok(())
    }

    #[tokio::test]
    async fn test_find_nft_contract() -> Result<()> {
        let conn_pool=create_db_conn_pool().await.unwrap();
        let data=find_nft_collection("xxxxxxxx", &conn_pool).await;
        println!("{:#?}",data);
        Ok(())
    }

    #[tokio::test]
    async fn test_inster_new_nft_contract() -> Result<()> {
        let conn_pool=create_db_conn_pool().await.unwrap();

        let price=tables::NftFloorPrice{
            nft_id:"1".to_string(),
            floor_price:"2".to_string(),
            ts:"2024-09-10T14:51:57Z".to_string()
        };

        let collection_price=tables::CollectionFloorPrice{
            floor_price:"1".to_string(),
            ts:"2024-09-10T14:51:57Z".to_string()
        };

        let data1=insert_new_nft_collection(
            "x1", None, None, &conn_pool).await;
        let data2=insert_new_nft_collection("x2", None, Some(vec![price]), &conn_pool).await;
        let data3=insert_new_nft_collection("xxxxxxxx",Some(collection_price) ,None, &conn_pool).await;

        println!("{:#?}",data1);
        println!("{:#?}",data2);
        println!("{:#?}",data3);
        Ok(())
    }

    #[tokio::test]
    async fn test_update_nft_collection() -> Result<()> {
        let conn_pool=create_db_conn_pool().await.unwrap();
        let price=tables::NftFloorPrice{
            nft_id:"3".to_string(),
            floor_price:"1usei".to_string(),
            ts:"2024-09-11T15:51:57Z".to_string()
        };
        let data=update_nft_collection("xxxxsx",&conn_pool,price).await;
        println!("{:#?}",data);
        Ok(())
    }

}