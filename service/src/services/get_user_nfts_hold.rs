use std::{collections::HashMap, mem::MaybeUninit};
use anyhow::Result;
use sei_client::{data_feild_structs::nft_data_struct, data_rp_structs::tx_rp_struct::FeeAmount};
use sqlx::PgPool;
use crate::{erros::ServicesErrs, responses::get_user_nfts_hold_rp, tools::get_nftcollection_floor_price};


pub async fn take<'services>(
    wallet_address:&'services str,
    conn_pool:&'services PgPool,
) -> Result<Vec<get_user_nfts_hold_rp::UserNFTCollectHold>> {

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
    
    let mut user_nft_collects_hold:Vec<get_user_nfts_hold_rp::UserNFTCollectHold>=Vec::new();
    
    for collection_hold in user_wallet.nft_hold{
        
        let mut nfts_hold:Vec<get_user_nfts_hold_rp::UserNFTHold>=Vec::new();
        
        let collection_and_nfts_floor_price=user_hold_nft_collect_floor_price.get(&collection_hold.collect_address);

        //init ptr || option is  NONE
        let mut nft_collection_floor_price_ptr:MaybeUninit<Option<String>>=MaybeUninit::new(None);
        let mut nft_floor_price_ptr:MaybeUninit<Option<String>>=MaybeUninit::new(None);
        let mut nft_buy_price_ptr:MaybeUninit<Option<String>>=MaybeUninit::new(None);
        let mut nft_market_fee_ptr:MaybeUninit<Option<String>>=MaybeUninit::new(None);
        let mut nft_unrealized_gains_ptr:MaybeUninit<Option<String>>=MaybeUninit::new(None);
        let mut gas_ptr:MaybeUninit<Vec<FeeAmount>>=MaybeUninit::new(Vec::new());
        let mut ts_ptr:MaybeUninit<Option<String>>=MaybeUninit::new(None);
        let mut tx_ptr:MaybeUninit<Option<String>>=MaybeUninit::new(None);

        // write to ptr
        collection_hold.nfts_hold.iter().for_each(|nft_hold|{
            
            // get nft floor price
            // if collection floor price is none ,the nft floor is the same
            if collection_and_nfts_floor_price.is_none(){
                    nft_floor_price_ptr.write(None);
                    nft_collection_floor_price_ptr.write(None);
            }else {
                // if collection is not none , find nft floor price from nft_id
                let nft_floor_price=collection_and_nfts_floor_price.unwrap().get(&nft_hold.token_id);
                let nft_collection_price=collection_and_nfts_floor_price.unwrap().get(&collection_hold.collect_address).unwrap().to_owned();
                // if nft_id is None
                if nft_floor_price.is_none(){
                    nft_floor_price_ptr.write(nft_collection_price.clone());
                    nft_collection_floor_price_ptr.write(nft_collection_price);
                }else {
                    nft_floor_price_ptr.write(nft_floor_price.unwrap().to_owned());
                    nft_collection_floor_price_ptr.write(nft_collection_price);
                }
            }

            // get nft buy_price unrealized_gains market_fee  royalties_fee
            user_wallet.nft_transactions.iter().for_each(|nft_transaction|{
                match nft_transaction {
                    nft_data_struct::NftTransaction::AcceptBid(t)=>{
                        if  t.collection==collection_hold.collect_address &&
                            t.nft_id==nft_hold.token_id &&
                            &t.recipient==wallet_address{
                                nft_buy_price_ptr.write(Some(t.sale_price.to_owned()));
                                nft_market_fee_ptr.write(Some(t.marketplace_fee.to_owned()));
                                gas_ptr.write(t.fee.to_owned());
                                ts_ptr.write(Some(t.ts.to_owned()));
                                tx_ptr.write(Some(t.tx.to_owned()));
                            }
                    },
                    nft_data_struct::NftTransaction::Mint(t) => {
                        if  t.collection==collection_hold.collect_address &&
                            t.nft_id==nft_hold.token_id &&
                            &t.recipient==wallet_address{
                                nft_buy_price_ptr.write(t.price.to_owned());
                                nft_market_fee_ptr.write(None);
                                gas_ptr.write(t.fee.to_owned());
                                ts_ptr.write(Some(t.ts.to_owned()));
                                tx_ptr.write(Some(t.tx.to_owned()));
                            }
                    },
                    nft_data_struct::NftTransaction::BatchBids(t) => {
                        if  t.collection==collection_hold.collect_address &&
                            t.nft_id == nft_hold.token_id &&
                            &t.recipient==wallet_address{
                                let sale_price=t.sale_price.clone();
                                let market_fee=(sale_price.get(0..sale_price.len()-4).unwrap().parse::<f64>().unwrap() * 0.2) as u64;
                                let market_fee=format!("{}usei",market_fee.to_string());

                                nft_buy_price_ptr.write(Some(t.sale_price.to_owned()));
                                nft_market_fee_ptr.write(Some(market_fee));
                                gas_ptr.write(t.fee.to_owned());
                                ts_ptr.write(Some(t.ts.to_owned()));
                                tx_ptr.write(Some(t.tx.to_owned()));
                        }
                    },
                    nft_data_struct::NftTransaction::Transfer(t) => {
                        if  t.collection==collection_hold.collect_address &&
                            t.nft_id==nft_hold.token_id &&
                            &t.recipient==wallet_address{
                                nft_buy_price_ptr.write(None);
                                nft_market_fee_ptr.write(None);
                                gas_ptr.write(t.fee.to_owned());
                                ts_ptr.write(Some(t.ts.to_owned()));
                                tx_ptr.write(Some(t.tx.to_owned()));
                        }
                    },
                    nft_data_struct::NftTransaction::FixedSell(t) => {
                        if  t.collection==collection_hold.collect_address &&
                            t.nft_id==nft_hold.token_id &&
                            &t.recipient==wallet_address{
                                let sale_price=format!("{}usei",&t.sale_price);
                                let market_fee=(sale_price.get(0..sale_price.len()-4).unwrap().parse::<f64>().unwrap() * 0.2) as u64;
                                let market_fee=format!("{}usei",market_fee.to_string());

                                nft_buy_price_ptr.write(Some(t.sale_price.to_owned()));
                                nft_market_fee_ptr.write(Some(market_fee));
                                gas_ptr.write(t.fee.to_owned());
                                ts_ptr.write(Some(t.ts.to_owned()));
                                tx_ptr.write(Some(t.tx.to_owned()));

                        }
                    },
                    nft_data_struct::NftTransaction::PurchaseCart(t) =>{
                        if  t.collection==collection_hold.collect_address &&
                            t.nft_id==nft_hold.token_id &&
                            &t.recipient==wallet_address{
                                nft_buy_price_ptr.write(Some(t.sale_price.to_owned()));
                                nft_market_fee_ptr.write(Some(t.marketplace_fee.to_owned()));
                                gas_ptr.write(t.fee.to_owned());
                                ts_ptr.write(Some(t.ts.to_owned()));
                                tx_ptr.write(Some(t.tx.to_owned()));
                        }
                    },
                    nft_data_struct::NftTransaction::CreateAuction(t) =>{
                        if  t.collection==collection_hold.collect_address &&
                            t.nft_id==nft_hold.token_id &&
                            &t.recipient==wallet_address{
                                let market_fee=(t.auction_price.to_owned().get(0..t.auction_price.to_owned().len()-4).unwrap().parse::<f64>().unwrap() * 0.2) as u64;
                                let market_fee=format!("{}usei",market_fee.to_string());

                                nft_buy_price_ptr.write(Some(t.auction_price.to_owned()));
                                nft_market_fee_ptr.write(Some(market_fee));
                                gas_ptr.write(t.fee.to_owned());
                                ts_ptr.write(Some(t.ts.to_owned()));
                                tx_ptr.write(Some(t.tx.to_owned()));
                        }
                    },
                    _=>{},
                }
            });

            // get init vaule
            let (nft_floor_price,nft_buy_price,nft_market_fee,gas_fee,ts,tx)=unsafe {
                let nft_floor_price=nft_floor_price_ptr.assume_init_ref();
                let nft_buy_price=nft_buy_price_ptr.assume_init_ref();
                let nft_market_fee=nft_market_fee_ptr.assume_init_ref();
                let gas_fee=gas_ptr.assume_init_ref();
                let ts=ts_ptr.assume_init_ref();
                let tx=tx_ptr.assume_init_ref();
                (nft_floor_price,nft_buy_price,nft_market_fee,gas_fee,ts,tx)
            };

            // calculate unrealized gains
            if nft_floor_price.is_none(){
                nft_unrealized_gains_ptr.write(None);
            }else if nft_floor_price.is_some() && nft_buy_price.is_some() && nft_market_fee.is_some() && gas_fee.is_empty() {
                
                let nft_floor_price=nft_floor_price.to_owned().unwrap();
                let nft_market_fee=nft_market_fee.to_owned().unwrap();
                let nft_buy_price=nft_buy_price.to_owned().unwrap();

                let nft_floor_price=nft_floor_price.get(0..nft_floor_price.len()-4).unwrap().parse::<i64>().unwrap();
                let nft_market_fee=nft_market_fee.get(0..nft_market_fee.len()-4).unwrap().parse::<i64>().unwrap();
                let nft_buy_price=nft_buy_price.get(0..nft_buy_price.len()-4).unwrap().parse::<i64>().unwrap();
                let unrealized_gains=nft_floor_price-nft_market_fee-nft_buy_price;
                
                nft_unrealized_gains_ptr.write(Some(format!("{}usei",unrealized_gains.to_string())));

            }else if nft_floor_price.is_some() && nft_buy_price.is_none() && nft_market_fee.is_some() &&  gas_fee.is_empty(){
                
                let nft_floor_price=nft_floor_price.to_owned().unwrap();
                let nft_market_fee=nft_market_fee.to_owned().unwrap();

                let nft_floor_price=nft_floor_price.get(0..nft_floor_price.len()-4).unwrap().parse::<i64>().unwrap();
                let nft_market_fee=nft_market_fee.get(0..nft_market_fee.len()-4).unwrap().parse::<i64>().unwrap();
                let unrealized_gains=nft_floor_price-nft_market_fee;

                nft_unrealized_gains_ptr.write(Some(format!("{}usei",unrealized_gains.to_string())));

            }else if nft_floor_price.is_some() && nft_buy_price.is_none() && nft_market_fee.is_none()&& gas_fee.is_empty() {
                nft_unrealized_gains_ptr.write(nft_floor_price.to_owned());
            }else if nft_floor_price.is_some() && nft_buy_price.is_some() && nft_market_fee.is_some() && !gas_fee.is_empty() {
                
                let nft_floor_price=nft_floor_price.to_owned().unwrap();
                let nft_market_fee=nft_market_fee.to_owned().unwrap();
                let nft_buy_price=nft_buy_price.to_owned().unwrap();

                let nft_floor_price=nft_floor_price.get(0..nft_floor_price.len()-4).unwrap().parse::<i64>().unwrap();
                let nft_market_fee=nft_market_fee.get(0..nft_market_fee.len()-4).unwrap().parse::<i64>().unwrap();
                let nft_buy_price=nft_buy_price.get(0..nft_buy_price.len()-4).unwrap().parse::<i64>().unwrap();
                let gas=gas_fee[0].to_owned().amount.parse::<i64>().unwrap();
                let unrealized_gains=nft_floor_price-nft_market_fee-nft_buy_price-gas;
                
                nft_unrealized_gains_ptr.write(Some(format!("{}usei",unrealized_gains.to_string())));

            }else if nft_floor_price.is_some() && nft_buy_price.is_none() && nft_market_fee.is_some() &&  !gas_fee.is_empty(){
                
                let nft_floor_price=nft_floor_price.to_owned().unwrap();
                let nft_market_fee=nft_market_fee.to_owned().unwrap();

                let nft_floor_price=nft_floor_price.get(0..nft_floor_price.len()-4).unwrap().parse::<i64>().unwrap();
                let nft_market_fee=nft_market_fee.get(0..nft_market_fee.len()-4).unwrap().parse::<i64>().unwrap();
                let gas=gas_fee[0].to_owned().amount.parse::<i64>().unwrap();
                let unrealized_gains=nft_floor_price-nft_market_fee-gas;

                nft_unrealized_gains_ptr.write(Some(format!("{}usei",unrealized_gains.to_string())));

            }else if nft_floor_price.is_some() && nft_buy_price.is_none() && nft_market_fee.is_none()&& !gas_fee.is_empty() {
                
                let nft_floor_price=nft_floor_price.to_owned().unwrap();

                let nft_floor_price=nft_floor_price.get(0..nft_floor_price.len()-4).unwrap().parse::<i64>().unwrap();
                let gas=gas_fee[0].to_owned().amount.parse::<i64>().unwrap();
                let unrealized_gains=nft_floor_price-gas;
                nft_unrealized_gains_ptr.write(Some(format!("{}usei",unrealized_gains.to_string())));
            };


            nfts_hold.push(get_user_nfts_hold_rp::UserNFTHold{
                name:nft_hold.name.to_owned(),
                key:nft_hold.key.to_owned(),
                token_id:nft_hold.token_id.to_owned(),
                image:nft_hold.image.to_owned(),
                buy_price:nft_buy_price.to_owned(),
                market_fee:nft_market_fee.to_owned(),
                floor_price:nft_floor_price.to_owned(),
                gas_fee:gas_fee.to_owned(),
                unrealized_gains:unsafe{nft_unrealized_gains_ptr.assume_init_ref().to_owned()},
                attributes:nft_hold.attributes.to_owned(),
                ts:ts.to_owned(),
                tx_hash:tx.to_owned()
            });
        });


        user_nft_collects_hold.push(get_user_nfts_hold_rp::UserNFTCollectHold{
            name:collection_hold.collect_info.name.to_owned(),
            symbol:collection_hold.collect_info.symbol.to_owned(),
            creator:collection_hold.collect_info.creator.to_owned(),
            contract:collection_hold.collect_address.to_owned(),
            floor_price:unsafe {nft_collection_floor_price_ptr.assume_init_ref().to_owned()},
            nfts_hold:nfts_hold,
        });

        // drop ptr
        drop(nft_collection_floor_price_ptr);
        drop(nft_floor_price_ptr);
        drop(nft_buy_price_ptr);
        drop(nft_market_fee_ptr);
        drop(nft_unrealized_gains_ptr);
        drop(gas_ptr);
        drop(ts_ptr);
        drop(tx_ptr);
    };

    return Ok(user_nft_collects_hold);
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