use std::collections::HashMap;
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use sei_client::data_feild_structs::nft_data_struct;
use sqlx::PgPool;
use crate::{erros::ServicesErrs, responses::get_user_income_nfts_hold_rp};
use sei_client::chain_apis;

pub async fn take<'services>(
    wallet_address:&'services str,
    conn_pool:&'services PgPool
) -> Result<Vec<get_user_income_nfts_hold_rp::IncomeNFTsCollect>> {

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
    
    let nft_transactions=user_wallet.nft_transactions;

    // set nft buy transactions
    let mut nft_buy_transactions:Vec<get_user_income_nfts_hold_rp::BuyNFTTransaction>=Vec::new();
    // set nft sell transactions
    let mut nft_sell_transactions:Vec<get_user_income_nfts_hold_rp::SellNFTTransaction>=Vec::new();

    // push  sell and buy transaction to vec from nft_transactions
    for nft_transaction in nft_transactions {
        match nft_transaction {
            nft_data_struct::NftTransaction::Mint(t) => {
                if &t.recipient==wallet_address{
                    nft_buy_transactions.push(
                      
                            get_user_income_nfts_hold_rp::BuyNFTTransaction{
                                collection_address:t.collection.to_owned(),
                                key:format!("{}-{}",&t.collection,&t.nft_id),
                                token_id:t.nft_id.to_owned(),
                                buy_price:{
                                    if t.price.is_none(){
                                        "0usei".to_string()
                                    }else {
                                        t.price.unwrap().to_owned()
                                    }
                                },
                                marketplace_fee:"0usei".to_string(),
                                fee:t.fee.to_owned(),
                                ts:t.ts.to_owned(),
                                tx:t.tx.to_owned()
                            }
                        
                    );
                }
            },
            nft_data_struct::NftTransaction::BatchBids(t) =>{
                if &t.recipient==wallet_address{
                    let market_fee=t.sale_price.clone();
                    let market_fee=(market_fee.get(0..market_fee.len()-4).unwrap().parse::<f64>().unwrap() * 0.2) as u64;
                    let market_fee=format!("{}usei",market_fee.to_string());

                    nft_buy_transactions.push(
                            get_user_income_nfts_hold_rp::BuyNFTTransaction{
                                collection_address:t.collection.to_owned(),
                                key:format!("{}-{}",&t.collection,&t.nft_id),
                                token_id:t.nft_id.to_owned(),
                                buy_price:t.sale_price.to_owned(),
                                marketplace_fee:market_fee,
                                fee:t.fee.to_owned(),
                                ts:t.ts.to_owned(),
                                tx:t.tx.to_owned()
                            }
                        
                    )
                };
                if &t.sender==wallet_address{
                    let nft_info=chain_apis::get_nfts_info_by_contract(Some(chain_apis::SERVER_RPC), &t.collection, &vec![t.nft_id.to_owned()]).await;
                    if nft_info.is_ok(){
                        let sale_price=t.sale_price.clone();
                        let royalty_percentage=nft_info.unwrap()[0].to_owned().royalty_percentage as f64 / 100_f64;
                        let royalty_fee=(sale_price.get(0..sale_price.len()-4).unwrap().parse::<f64>().unwrap() * royalty_percentage) as u64;
                        let royalty_fee=format!("{}usei",royalty_fee.to_string());

                        nft_sell_transactions.push(
                            
                                get_user_income_nfts_hold_rp::SellNFTTransaction{
                                    collection_address:t.collection.to_owned(),
                                    key:format!("{}-{}",&t.collection,&t.nft_id),
                                    token_id:t.nft_id.to_owned(),
                                    sell_price:t.sale_price.to_owned(),
                                    royalties:royalty_fee,
                                    fee:t.fee.to_owned(),
                                    ts:t.ts.to_owned(),
                                    tx:t.tx.to_owned()
                                }
                            
                        )
                    }
                }
            },
            nft_data_struct::NftTransaction::Transfer(t) => {
                if &t.recipient==wallet_address{
                    nft_buy_transactions.push(
                        
                            get_user_income_nfts_hold_rp::BuyNFTTransaction{
                                collection_address:t.collection.to_owned(),
                                key:format!("{}-{}",&t.collection,&t.nft_id),
                                token_id:t.nft_id.to_owned(),
                                buy_price:"0usei".to_string(),
                                marketplace_fee:"0usei".to_string(),
                                fee:t.fee.to_owned(),
                                ts:t.ts.to_owned(),
                                tx:t.tx.to_owned()
                            }
                        
                    )
                };
                if &t.sender==wallet_address{
                    nft_sell_transactions.push(
                        
                            get_user_income_nfts_hold_rp::SellNFTTransaction{
                                collection_address:t.collection.to_owned(),
                                key:format!("{}-{}",&t.collection,&t.nft_id),
                                token_id:t.nft_id.to_owned(),
                                sell_price:"0usei".to_string(),
                                royalties:"0usei".to_string(),
                                fee:t.fee.to_owned(),
                                ts:t.ts.to_owned(),
                                tx:t.tx.to_owned()
                            }
                        
                    );
                }
            },
            nft_data_struct::NftTransaction::FixedSell(t) => {
                if &t.recipient==wallet_address{
                    let market_fee=t.sale_price.clone();
                    let market_fee=(market_fee.get(0..market_fee.len()-4).unwrap().parse::<f64>().unwrap() * 0.2) as u64;
                    let market_fee=format!("{}usei",market_fee.to_string());

                    nft_buy_transactions.push(
                       
                            get_user_income_nfts_hold_rp::BuyNFTTransaction{
                                collection_address:t.collection.to_owned(),
                                key:format!("{}-{}",&t.collection,&t.nft_id),
                                token_id:t.nft_id.to_owned(),
                                buy_price:t.sale_price.to_owned(),
                                marketplace_fee:market_fee,
                                fee:t.fee.to_owned(),
                                ts:t.ts.to_owned(),
                                tx:t.tx.to_owned()
                            }
                    );
                };
                if &t.sender==wallet_address{
                    let nft_info=chain_apis::get_nfts_info_by_contract(Some(chain_apis::SERVER_RPC), &t.collection, &vec![t.nft_id.to_owned()]).await;
                    if nft_info.is_ok(){
                        let sale_price=t.sale_price.clone();
                        let royalty_percentage=nft_info.unwrap()[0].to_owned().royalty_percentage as f64 / 100_f64;
                        let royalty_fee=(sale_price.get(0..sale_price.len()-4).unwrap().parse::<f64>().unwrap() * royalty_percentage) as u64;
                        let royalty_fee=format!("{}usei",royalty_fee.to_string());

                        nft_sell_transactions.push(
                           
                                get_user_income_nfts_hold_rp::SellNFTTransaction{
                                    collection_address:t.collection.to_owned(),
                                    key:format!("{}-{}",&t.collection,&t.nft_id),
                                    token_id:t.nft_id.to_owned(),
                                    sell_price:t.sale_price.to_owned(),
                                    royalties:royalty_fee,
                                    fee:t.fee.to_owned(),
                                    ts:t.ts.to_owned(),
                                    tx:t.tx.to_owned()
                                }
                        );
                    }
                }
            },
            nft_data_struct::NftTransaction::PurchaseCart(t) => {
                if &t.recipient==wallet_address{
                    nft_buy_transactions.push(
                       
                            get_user_income_nfts_hold_rp::BuyNFTTransaction{
                                collection_address:t.collection.to_owned(),
                                key:format!("{}-{}",&t.collection,&t.nft_id),
                                token_id:t.nft_id.to_owned(),
                                buy_price:t.sale_price.to_owned(),
                                marketplace_fee:t.marketplace_fee.to_owned(),
                                fee:t.fee.to_owned(),
                                ts:t.ts.to_owned(),
                                tx:t.tx.to_owned()
                            }
                    );
                };
                if &t.sender==wallet_address{
                    nft_sell_transactions.push(
                        
                            get_user_income_nfts_hold_rp::SellNFTTransaction{
                                collection_address:t.collection.to_owned(),
                                key:format!("{}-{}",&t.collection,&t.nft_id),
                                token_id:t.nft_id.to_owned(),
                                sell_price:t.sale_price.to_owned(),
                                royalties:t.royalties.to_owned(),
                                fee:t.fee.to_owned(),
                                ts:t.ts.to_owned(),
                                tx:t.tx.to_owned()
                            }
                    );
                }
            },
            nft_data_struct::NftTransaction::AcceptBid(t) => {
                if &t.recipient==wallet_address{
                    nft_buy_transactions.push(
                        
                            get_user_income_nfts_hold_rp::BuyNFTTransaction{
                                collection_address:t.collection.to_owned(),
                                key:format!("{}-{}",&t.collection,&t.nft_id),
                                token_id:t.nft_id.to_owned(),
                                buy_price:t.sale_price.to_owned(),
                                marketplace_fee:t.marketplace_fee.to_owned(),
                                fee:t.fee.to_owned(),
                                ts:t.ts.to_owned(),
                                tx:t.tx.to_owned()
                            }
                    );
                };
                if &t.sender==wallet_address{
                    nft_sell_transactions.push(
                       
                            get_user_income_nfts_hold_rp::SellNFTTransaction{
                                collection_address:t.collection.to_owned(),
                                key:format!("{}-{}",&t.collection,&t.nft_id),
                                token_id:t.nft_id.to_owned(),
                                sell_price:t.sale_price.to_owned(),
                                royalties:t.royalties.to_owned(),
                                fee:t.fee.to_owned(),
                                ts:t.ts.to_owned(),
                                tx:t.tx.to_owned()
                            }
                    );
                }
            },
            nft_data_struct::NftTransaction::CreateAuction(t) => {
                if &t.recipient==wallet_address{
                    let market_fee=t.auction_price.clone();
                    let market_fee=(market_fee.get(0..market_fee.len()-4).unwrap().parse::<f64>().unwrap() * 0.2) as u64;
                    let market_fee=format!("{}usei",market_fee.to_string());

                    nft_buy_transactions.push(
                       
                            get_user_income_nfts_hold_rp::BuyNFTTransaction{
                                collection_address:t.collection.to_owned(),
                                key:format!("{}-{}",&t.collection,&t.nft_id),
                                token_id:t.nft_id.to_owned(),
                                buy_price:t.auction_price.to_owned(),
                                marketplace_fee:market_fee,
                                fee:t.fee.to_owned(),
                                ts:t.ts.to_owned(),
                                tx:t.tx.to_owned()
                            }
                    );
                };
                if &t.sender==wallet_address{
                    let nft_info=chain_apis::get_nfts_info_by_contract(Some(chain_apis::SERVER_RPC), &t.collection, &vec![t.nft_id.to_owned()]).await;
                    
                    if nft_info.is_ok(){
                        let sale_price=t.auction_price.clone();
                        let royalty_percentage=nft_info.unwrap()[0].to_owned().royalty_percentage as f64 / 100_f64;
                        let royalty_fee=(sale_price.get(0..sale_price.len()-4).unwrap().parse::<f64>().unwrap() * royalty_percentage) as u64;
                        let royalty_fee=format!("{}usei",royalty_fee.to_string());
                        nft_sell_transactions.push(
                          
                                get_user_income_nfts_hold_rp::SellNFTTransaction{
                                    collection_address:t.collection.to_owned(),
                                    key:format!("{}-{}",&t.collection,&t.nft_id),
                                    token_id:t.nft_id.to_owned(),
                                    sell_price:t.auction_price.to_owned(),
                                    royalties:royalty_fee,
                                    fee:t.fee.to_owned(),
                                    ts:t.ts.to_owned(),
                                    tx:t.tx.to_owned()
                                }
                        );
                    }
                }
            },
            _=>{},
        }
    };

    if  nft_buy_transactions.is_empty() || nft_sell_transactions.is_empty(){
        return  Err(ServicesErrs::NFTsTransactionsIsNone.into());
    }

    // 合并集合
    let mut buy_transactions:HashMap<String,HashMap<String,Vec<get_user_income_nfts_hold_rp::BuyNFTTransaction>>>=HashMap::new();
    let mut sell_transactions:HashMap<String,HashMap<String,Vec<get_user_income_nfts_hold_rp::SellNFTTransaction>>>=HashMap::new();

    
    nft_buy_transactions.iter().for_each(|nft_buy_transaction|{
        buy_transactions.entry(nft_buy_transaction.collection_address.to_owned())
                        .or_insert_with(HashMap::new)
                        .entry(nft_buy_transaction.token_id.to_owned())
                        .or_insert_with(Vec::new)
                        .push(nft_buy_transaction.clone())
            
    });

    nft_sell_transactions.iter().for_each(|nft_sell_transaction|{
        
        sell_transactions.entry(nft_sell_transaction.collection_address.to_owned())
                         .or_insert_with(HashMap::new)
                         .entry(nft_sell_transaction.token_id.to_owned())
                         .or_insert_with(Vec::new)
                         .push(nft_sell_transaction.clone())
    });


    let time_now=Utc::now();

    // 过滤历史交易记录
    buy_transactions.iter_mut().for_each(|(_,nft_ids)|{
        nft_ids.iter_mut().for_each(|(_,transactions)|{
            
            let mut shortest_diff: Option<Duration> = None;
            let mut closest_transaction: Option<&get_user_income_nfts_hold_rp::BuyNFTTransaction> = None;

            transactions.iter().for_each(|transaction|{
                let time=DateTime::parse_from_rfc3339(&transaction.ts).unwrap().with_timezone(&Utc);
                let time_diff=time_now.signed_duration_since(time);
                if shortest_diff.map_or(true, |d| time_diff<d){
                    shortest_diff=Some(time_diff);
                    closest_transaction=Some(transaction)
                }
            });
            if let Some(closest_transaction) = closest_transaction {
                let time=closest_transaction.ts.clone();
                transactions.retain(|transaction| transaction.ts==time);
            }
        });
    });
    
    sell_transactions.iter_mut().for_each(|(_,nft_ids)|{
        nft_ids.iter_mut().for_each(|(_,transactions)|{

            let mut shortest_diff: Option<Duration> = None;
            let mut closest_transaction: Option<&get_user_income_nfts_hold_rp::SellNFTTransaction> = None;
            
            transactions.iter().for_each(|transaction|{
                let time=DateTime::parse_from_rfc3339(&transaction.ts).unwrap().with_timezone(&Utc);
                let time_diff=time_now.signed_duration_since(time);
                if shortest_diff.map_or(true, |d| time_diff<d){
                    shortest_diff=Some(time_diff);
                    closest_transaction=Some(transaction)
                }
            });
            if let Some(closest_transaction) = closest_transaction {
                let time=closest_transaction.ts.clone();
                transactions.retain(|transaction| transaction.ts==time);
            }
        });
    });


    let mut income_nfts_collects:Vec<get_user_income_nfts_hold_rp::IncomeNFTsCollect>=Vec::new();
    // 匹配买卖
    'b:for buy_transaction in buy_transactions.iter(){
        's:for sell_transaction in sell_transactions.iter(){
            //同一集合
            if buy_transaction.0 != sell_transaction.0{
                continue 's;
            };

            let collection_info=chain_apis::get_nft_collect_info_by_contract(Some(chain_apis::SERVER_RPC), buy_transaction.0.as_str()).await;
            
            if collection_info.is_err(){
                continue 'b;
            };

            let collection_info=collection_info.unwrap();
            let mut income_nfts:Vec<get_user_income_nfts_hold_rp::IncomeNFT>=Vec::new();


            let buy_nfts=buy_transaction.1;
            let sell_nfts=sell_transaction.1;

            'b_nft:for buy_nft in buy_nfts{
                's_nft:for sell_nft in sell_nfts{
                    //同一id
                    if buy_nft.0 !=sell_nft.0{
                        continue 's_nft;
                    };
                    
                    let nft_info=chain_apis::get_nfts_info_by_contract(Some(chain_apis::SERVER_RPC), buy_transaction.0.as_str(), &vec![buy_nft.0.to_owned()]).await;
                    if nft_info.is_err(){
                        continue 'b_nft;
                    };
                    let nft_info=nft_info.unwrap()[0].to_owned();


                    let buy_t=buy_nft.1[0].to_owned();
                    let sell_t=sell_nft.1[0].to_owned();

                    let buy_time=DateTime::parse_from_rfc3339(&buy_t.ts).unwrap();
                    let sell_time=DateTime::parse_from_rfc3339(&sell_t.ts).unwrap();

                    match buy_time.cmp(&sell_time) {
                        std::cmp::Ordering::Less=>{
                            
                            let buy_price=buy_t.buy_price;
                            let sell_price=sell_t.sell_price;
                            let maket_fee=buy_t.marketplace_fee;
                            let royalties_fee=sell_t.royalties;
                            let buy_fee={
                                if buy_t.fee.is_empty(){
                                    "0usei".to_string()
                                }else {
                                    format!("{}usei",buy_t.fee[0].amount)
                                }
                            };
                            let sell_fee={
                                if sell_t.fee.is_empty(){
                                    "0usei".to_string()
                                }else {
                                    format!("{}usei",sell_t.fee[0].amount)
                                }
                            };


                            let buy_price_i64=buy_price.get(..&buy_price.len()-4).unwrap().parse::<i64>().unwrap();
                            let sell_price_i64=sell_price.get(..&sell_fee.len()-4).unwrap().parse::<i64>().unwrap();
                            let market_fee_i64=maket_fee.get(..&maket_fee.len()-4).unwrap().parse::<i64>().unwrap();
                            let royalties_fee_i64=royalties_fee.get(..&royalties_fee.len()-4).unwrap().parse::<i64>().unwrap();
                            let buy_fee_i64=buy_fee.get(..&buy_fee.len()-4).unwrap().parse::<i64>().unwrap();
                            let sell_fee_i64=buy_fee.get(..&sell_fee.len()-4).unwrap().parse::<i64>().unwrap();

                            let realized_gains_i64=sell_price_i64-buy_price_i64-market_fee_i64-royalties_fee_i64-sell_fee_i64-buy_fee_i64;
                            let pay_fee_i64=market_fee_i64+royalties_fee_i64+buy_fee_i64+sell_fee_i64;

                            let hold_time=sell_time.signed_duration_since(buy_time);

                            income_nfts.push(get_user_income_nfts_hold_rp::IncomeNFT{
                                name:nft_info.name,
                                key:nft_info.key,
                                token_id:nft_info.token_id,
                                image:nft_info.image,
                                buy_price:format!("{}usei",buy_fee_i64.to_string()),
                                sell_price:format!("{}usei",sell_price_i64.to_string()),
                                hold_time:format_duration(hold_time),
                                realized_gains:format!("{}usei",realized_gains_i64.to_string()),
                                paid_fee:format!("{}usei",pay_fee_i64.to_string())
                            })
                        
                        },
                        _=>{},
                    }

                }
            }
            
            income_nfts_collects.push(get_user_income_nfts_hold_rp::IncomeNFTsCollect{
                name:collection_info.name,
                creator:collection_info.creator,
                contract:buy_transaction.0.to_string(),
                income_nfts:income_nfts
            })
        }
    }


    Ok(income_nfts_collects)
}


fn format_duration(duration: Duration) -> String {
    let seconds = duration.num_seconds();
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;

    format!("{}h{}min{}s", hours, minutes, seconds)
}