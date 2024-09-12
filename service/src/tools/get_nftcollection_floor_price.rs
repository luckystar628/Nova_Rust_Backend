use std::{collections::HashMap, sync::Arc};
use anyhow::Result;
use sqlx::PgPool;
use tokio::sync::{Mutex, Semaphore};

use crate::erros::ServicesErrs;

pub async fn take<'tools>(
    collection_keys:&'tools HashMap<String,Vec<String>>,
    conn_pool:&'tools PgPool,
)->HashMap<String,HashMap<String,Option<String>>>{
    let mut nft_collections_floor_prices:HashMap<String,HashMap<String,Option<String>>>=HashMap::new();
    let nft_collections_floor_prices=Arc::new(Mutex::new(nft_collections_floor_prices));
    let conn_pool=Arc::new(conn_pool.to_owned());

    // 限制并发 16
    let semaphore=Arc::new(Semaphore::new(16));
    // 缓存空间
    let mut hanldes:Vec<tokio::task::JoinHandle<Result<()>>>=Vec::new();

    for (collection,nft_ids) in collection_keys.to_owned(){
        
        let semaphore=Arc::clone(&semaphore);
        let nft_collections_floor_prices=Arc::clone(&nft_collections_floor_prices);
        let conn_pool=Arc::clone(&conn_pool);

        
        let hanlde:tokio::task::JoinHandle<Result<()>>=tokio::spawn(async move{
            
            let permit=semaphore.acquire().await?;
            let nft_collections_floor_prices=&mut nft_collections_floor_prices.lock().await;
            let mut nfts_floor_price:HashMap<String,Option<String>>=HashMap::new();

            let nft_collection_data=nova_db::find_nft_collection(&collection, &conn_pool).await?;
            let collect_nfts_floor_price =nft_collection_data.nfts_floor_price;
            let nft_collection_floor_price=nft_collection_data.collection_floor_price;
            
            if nft_collection_floor_price.is_none(){
                return Err(ServicesErrs::NFTCollectIsNone.into());
            }
            if collect_nfts_floor_price.is_empty(){
                nfts_floor_price.insert(collection.to_owned(), Some(nft_collection_floor_price.unwrap().floor_price));
            }else {
                nfts_floor_price.insert(collection.to_owned(), Some(nft_collection_floor_price.unwrap().floor_price));
                
                for nft_id in nft_ids{
                    collect_nfts_floor_price.iter().for_each(|nft_floor_price|{
                        if nft_floor_price.nft_id == nft_id{
                            nfts_floor_price.insert(nft_id.to_owned(), Some(nft_floor_price.floor_price.to_owned()));
                        }
                    });
                }
                
            }
            nft_collections_floor_prices.insert(collection.to_owned(), nfts_floor_price);
            Ok(())

        });
        hanldes.push(hanlde);
    }
    futures::future::join_all(hanldes).await;
    let res=nft_collections_floor_prices.lock().await.to_owned();
    return res;
    
   
}