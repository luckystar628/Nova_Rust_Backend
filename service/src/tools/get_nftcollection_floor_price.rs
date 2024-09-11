use std::{collections::HashMap, sync::Arc};

use tokio::sync::{Mutex, Semaphore};

pub async fn take<'tools>(
    collection_keys:&'tools HashMap<String,Vec<String>>
){
    let mut nft_collections_floor_prices:HashMap<String,HashMap<String,Option<String>>>=HashMap::new();
    let nft_collections_floor_prices=Arc::new(Mutex::new(nft_collections_floor_prices));

    // 限制并发 16
    let seamphore=Arc::new(Semaphore::new(16));
    // 缓存空间
    let hanldes=
}