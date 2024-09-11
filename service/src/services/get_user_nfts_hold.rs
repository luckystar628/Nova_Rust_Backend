use std::collections::HashMap;
use anyhow::Result;
use sei_client::data_feild_structs::nft_data_struct;
use sqlx::PgPool;

use crate::erros::ServicesErrs;


pub async fn take<'services>(
    wallet_address:&'services str,
    conn_pool:&'services PgPool,
) -> Result<()> {

    let user_wallet={
        let searcher_user=nova_db::find_wallet_info(wallet_address, &conn_pool).await;
        if searcher_user.is_err(){
            return Err(ServicesErrs::UserWalletNotFound.into());
        };
        let wallet_info=searcher_user.unwrap();
        if wallet_info.nft_hold.is_empty(){
            return Err(ServicesErrs::UserNotHaveNFTs.into());
        }
        wallet_info
    };

    let ck_hashmap=add_ck_hashmap(&user_wallet.nft_hold);

    

    Ok(())
}

fn add_ck_hashmap<'get_nft_hold_tool>(
    user_hold_nft_collections:&'get_nft_hold_tool Vec<nft_data_struct::NftCollectHold>
)->HashMap<String,Vec<String>>{
    
    let mut ch_hashmap:HashMap<String, Vec<String>>=HashMap::new();

    user_hold_nft_collections.iter().for_each(|user_hold_nft_collection|{
        let mut nft_keys:Vec<String>=Vec::new();
        user_hold_nft_collection.nfts_hold.iter().for_each(|nft_hold|{
            nft_keys.push(nft_hold.token_id.to_owned())
        });
        ch_hashmap.insert(user_hold_nft_collection.collect_address.to_owned(), nft_keys);
    });
    
    ch_hashmap
}