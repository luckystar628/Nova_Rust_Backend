use std::mem::MaybeUninit;
use  anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use crate::{erros::ServicesErrs, responses::{get_user_nfts_hold_rp, get_user_nfts_trade_info_rp}};

use super::get_user_nfts_hold;

pub async fn take<'services>(
    wallet_address:&'services str,
    conn_pool:&'services PgPool,
) -> Result<get_user_nfts_trade_info_rp::NFTTradeInfo> {

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


    let mut age_of_nft_assets_ptr:MaybeUninit<Option<get_user_nfts_trade_info_rp::AgeOfNftAssets>>=MaybeUninit::new(None);

    // 写入 age_of_nft_assets_ptr
    if let  Ok(user_collects_hold)=get_user_nfts_hold::take(wallet_address, conn_pool).await {

        let day_now=Utc::now().date_naive();

        let mut holding_1_week_nfts:Vec<get_user_nfts_hold_rp::UserNFTHold>=Vec::new();   // =<7
        let mut holding_1_to_4_weeks_nfts:Vec<get_user_nfts_hold_rp::UserNFTHold>=Vec::new();    //  7 <x =< 28
        let mut holding_1_to_3_months_nfts:Vec<get_user_nfts_hold_rp::UserNFTHold>=Vec::new();   // 28< x =< 90
        let mut holding_3_to_6_months_nfts:Vec<get_user_nfts_hold_rp::UserNFTHold>=Vec::new();    // 90 < x =< 189
        let mut holding_6_to_12_months_nfts:Vec<get_user_nfts_hold_rp::UserNFTHold>=Vec::new();    // 180 < x =< 360
        let mut holding_more_than_1_years_nfts:Vec<get_user_nfts_hold_rp::UserNFTHold>=Vec::new();    // 360 <x

        user_collects_hold.iter().for_each(|collect_hold|{
            collect_hold.nfts_hold.iter().for_each(|nft_hold|{
                if let Some(ts) =nft_hold.ts.to_owned()  {
                    let ts=DateTime::parse_from_rfc3339(&ts).unwrap().with_timezone(&Utc).date_naive();
                    let duration=day_now.signed_duration_since(ts);
                    
                    if duration>Duration::days(360){
                        holding_more_than_1_years_nfts.push(nft_hold.to_owned())
                    }else if duration>=Duration::days(189) {
                        holding_6_to_12_months_nfts.push(nft_hold.to_owned())
                    }else if duration >=Duration::days(90) {
                        holding_3_to_6_months_nfts.push(nft_hold.to_owned())
                    }else if duration >= Duration::days(28) {
                        holding_1_to_3_months_nfts.push(nft_hold.to_owned())
                    }else if duration >=Duration::days(7) {
                        holding_1_to_4_weeks_nfts.push(nft_hold.to_owned())
                    }else {
                        holding_1_week_nfts.push(nft_hold.to_owned())
                    }
                }
            });
        });

        age_of_nft_assets_ptr.write(Some(get_user_nfts_trade_info_rp::AgeOfNftAssets{
            level1: holding_1_week_nfts, 
            level2: holding_1_to_4_weeks_nfts, 
            level3: holding_1_to_3_months_nfts, 
            level4: holding_3_to_6_months_nfts, 
            level5: holding_6_to_12_months_nfts, 
            level6: holding_more_than_1_years_nfts, 
        }));

    }else {
        age_of_nft_assets_ptr.write(None);
    }

    let mut all_buy_trades:Vec<get_user_nfts_trade_info_rp::TradeInfo>=Vec::new();
    let mut all_sell_trades:Vec<get_user_nfts_trade_info_rp::TradeInfo>=Vec::new();
    let mut all_trades:Vec<get_user_nfts_trade_info_rp::TradeInfo>=Vec::new();   // all  transaction


    let nft_transactions=user_wallet.nft_transactions;
    nft_transactions.iter().for_each(|nft_transaction|{
        match nft_transaction {
            sei_client::data_feild_structs::nft_data_struct::NftTransaction::Mint(t) => {
                let ts=t.ts.to_owned();
                let sale_price={
                    if let Some(price)  =t.price.to_owned()  {
                        price
                    }else {
                        "0usei".to_string()
                    }
                };

                let trade_info=get_user_nfts_trade_info_rp::TradeInfo{
                    ts:ts,
                    sale_price:sale_price,
                };

                all_trades.push(trade_info);
            },
            sei_client::data_feild_structs::nft_data_struct::NftTransaction::BatchBids(t) => {
                let ts=t.ts.to_owned();
                let sale_price=t.sale_price.to_owned();
                
                let trade_info=get_user_nfts_trade_info_rp::TradeInfo{
                    ts:ts,
                    sale_price:sale_price,
                };

                all_trades.push(trade_info.clone());
                if &t.recipient ==wallet_address{
                    all_buy_trades.push(trade_info.clone());
                }else if &t.sender == wallet_address {
                    all_sell_trades.push(trade_info.clone());
                };
            },
            sei_client::data_feild_structs::nft_data_struct::NftTransaction::Transfer(t) => {
                let ts=t.ts.to_owned();
                let sale_price="0usei".to_string();

                let trade_info=get_user_nfts_trade_info_rp::TradeInfo{
                    ts:ts,
                    sale_price:sale_price,
                };
                all_trades.push(trade_info.clone());
                if &t.recipient ==wallet_address{
                    all_buy_trades.push(trade_info.clone());
                }else if &t.sender == wallet_address {
                    all_sell_trades.push(trade_info.clone());
                };
            },
            sei_client::data_feild_structs::nft_data_struct::NftTransaction::FixedSell(t) => {
                let ts=t.ts.to_owned();
                let sale_price=t.sale_price.to_owned();

                let trade_info=get_user_nfts_trade_info_rp::TradeInfo{
                    ts:ts,
                    sale_price:sale_price,
                };

                all_trades.push(trade_info.clone());
                if &t.recipient ==wallet_address{
                    all_buy_trades.push(trade_info.clone());
                }else if &t.sender == wallet_address {
                    all_sell_trades.push(trade_info.clone());
                };
            },
            sei_client::data_feild_structs::nft_data_struct::NftTransaction::PurchaseCart(t) => {
                let ts=t.ts.to_owned();
                let sale_price=t.sale_price.to_owned();

                let trade_info=get_user_nfts_trade_info_rp::TradeInfo{
                    ts:ts,
                    sale_price:sale_price,
                };

                all_trades.push(trade_info.clone());
                if &t.recipient ==wallet_address{
                    all_buy_trades.push(trade_info.clone());
                }else if &t.sender == wallet_address {
                    all_sell_trades.push(trade_info.clone());
                };
            },
            sei_client::data_feild_structs::nft_data_struct::NftTransaction::AcceptBid(t) => {
                let ts=t.ts.to_owned();
                let sale_price=t.sale_price.to_owned();

                let trade_info=get_user_nfts_trade_info_rp::TradeInfo{
                    ts:ts,
                    sale_price:sale_price,
                };

                all_trades.push(trade_info.clone());
                if &t.recipient ==wallet_address{
                    all_buy_trades.push(trade_info.clone());
                }else if &t.sender == wallet_address {
                    all_sell_trades.push(trade_info.clone());
                };
            },
            sei_client::data_feild_structs::nft_data_struct::NftTransaction::CreateAuction(t) =>{
                let ts=t.ts.to_owned();
                let sale_price=t.auction_price.to_owned();

                let trade_info=get_user_nfts_trade_info_rp::TradeInfo{
                    ts:ts,
                    sale_price:sale_price,
                };

                all_trades.push(trade_info.clone());
                if &t.recipient ==wallet_address{
                    all_buy_trades.push(trade_info.clone());
                }else if &t.sender == wallet_address {
                    all_sell_trades.push(trade_info.clone());
                };
            },
            sei_client::data_feild_structs::nft_data_struct::NftTransaction::CancelAuction(t) => {
                let ts=t.ts.to_owned();
                let sale_price=t.auction_price.to_owned();

                let trade_info=get_user_nfts_trade_info_rp::TradeInfo{
                    ts:ts,
                    sale_price:sale_price,
                };

                all_trades.push(trade_info);
            },
            _=>{},
        }
    });

    
    let mut nft_transaction=get_user_nfts_trade_info_rp::NFTTransactions::new();
    get_user_nfts_trade_info_rp::NFTTransactions::add_data(&mut nft_transaction, all_trades);

    let mut buy_volume=get_user_nfts_trade_info_rp::NFTTransactions::new();
    get_user_nfts_trade_info_rp::NFTTransactions::add_data(&mut buy_volume, all_buy_trades);

    let mut sell_volume=get_user_nfts_trade_info_rp::NFTTransactions::new();
    get_user_nfts_trade_info_rp::NFTTransactions::add_data(&mut sell_volume, all_sell_trades);

    let age_of_nft_assets=unsafe {age_of_nft_assets_ptr.assume_init_read()};
    drop(age_of_nft_assets_ptr);



    Ok(get_user_nfts_trade_info_rp::NFTTradeInfo{
        age_of_nft_assets:age_of_nft_assets,
        transaction:nft_transaction,
        volume:get_user_nfts_trade_info_rp::NFTTradeVolume{
            buy_volume:buy_volume,
            sell_volume:sell_volume
        }
    })
}