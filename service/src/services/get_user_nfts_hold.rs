use std::{collections::HashMap, mem::MaybeUninit};
use anyhow::Result;
use sei_client::data_feild_structs::nft_data_struct;
use sqlx::PgPool;
use crate::{erros::ServicesErrs, responses::get_user_nfts_hold_rp, tools::get_nftcollection_floor_price};


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
    let user_hold_nft_collect_floor_price=get_nftcollection_floor_price::take(&ck_hashmap,conn_pool).await;
    
    let user_nft_collects_hold:Vec<get_user_nfts_hold_rp::UserNFTCollectHold>=Vec::new();
    for collection_hold in user_wallet.nft_hold{
        
        let nfts_hold:Vec<get_user_nfts_hold_rp::UserNFTHold>=Vec::new();
        
        let collection_and_nfts_floor_price=user_hold_nft_collect_floor_price.get(&collection_hold.collect_address);

        //init ptr || option is  NONE
        let mut nft_floor_price_ptr:MaybeUninit<Option<String>>=MaybeUninit::new(None);
        let mut nft_buy_price_ptr:MaybeUninit<Option<String>>=MaybeUninit::new(None);
        let mut nft_royalties_fee_ptr:MaybeUninit<Option<String>>=MaybeUninit::new(None);
        let mut nft_market_fee_ptr:MaybeUninit<Option<String>>=MaybeUninit::new(None);
        let mut nft_unrealized_gains_ptr:MaybeUninit<Option<String>>=MaybeUninit::new(None);
        let mut ts_ptr:MaybeUninit<Option<String>>=MaybeUninit::new(None);
        let mut tx_ptr:MaybeUninit<Option<String>>=MaybeUninit::new(None);

        collection_hold.nfts_hold.iter().for_each(|nft_hold|{
            // get nft floor price
            #[warn(unused_unsafe)]
            unsafe {
                // if collection floor price is none ,the nft floor is the same
                if collection_and_nfts_floor_price.is_none(){
                    nft_floor_price_ptr.write(None);
                }else {
                    // if collection is not none , find nft floor price from nft_id
                    let nft_floor_price=collection_and_nfts_floor_price.unwrap().get(&nft_hold.token_id);
                    // if nft_id is None
                    if nft_floor_price.is_none(){
                        nft_floor_price_ptr.write(None);
                    }else {
                        nft_buy_price_ptr.write(nft_floor_price.unwrap().to_owned());
                    }
                }
            };

            
        });
    };


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