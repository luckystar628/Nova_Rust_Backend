use sqlx::PgPool;
use anyhow::Result;
use crate::responses::get_user_nfts_hold_top_rp;

use super::get_user_nfts_hold;

pub async fn take<'services>(
    wallet_address:&'services str,
    conn_pool:&'services PgPool,
) -> Result<get_user_nfts_hold_top_rp::NftHoldTop> {

    let collects_hold={
        let x= get_user_nfts_hold::take(wallet_address, conn_pool).await;
        if x.is_err(){
            return Err(x.err().unwrap());
        };
        x.unwrap()
    };
   
    let mut top_gainers_nfts:Vec<get_user_nfts_hold_top_rp::NftTop>=Vec::new();
    let mut top_losser_nfts:Vec<get_user_nfts_hold_top_rp::NftTop>=Vec::new();

    collects_hold.iter().for_each(|collect_hold|{
        collect_hold.nfts_hold.iter().for_each(|nft_hold|{
            if nft_hold.unrealized_gains.is_some(){
                let unrealized_gains=nft_hold.to_owned().unrealized_gains.unwrap();
                let unrealized_gains=unrealized_gains.get(..&unrealized_gains.len()-4).unwrap().parse::<i64>().unwrap();
                if unrealized_gains>=0{
                    top_gainers_nfts.push(
                        get_user_nfts_hold_top_rp::NftTop{
                            name:nft_hold.name.to_owned(),
                            key:nft_hold.key.to_owned(),
                            token_id:nft_hold.token_id.to_owned(),
                            image:nft_hold.image.to_owned(),
                            buy_price:nft_hold.buy_price.to_owned(),
                            market_fee:nft_hold.market_fee.to_owned(),
                            floor_price:nft_hold.floor_price.to_owned(),
                            gas_fee:nft_hold.gas_fee.to_owned(),
                            unrealized_gains:format!("{}usei",unrealized_gains.to_string()),
                            attributes:nft_hold.attributes.to_owned(),
                            ts:nft_hold.ts.to_owned(),
                            tx_hash:nft_hold.tx_hash.to_owned(),
                        }
                    )
                }else {
                    top_losser_nfts.push(
                        get_user_nfts_hold_top_rp::NftTop{
                            name:nft_hold.name.to_owned(),
                            key:nft_hold.key.to_owned(),
                            token_id:nft_hold.token_id.to_owned(),
                            image:nft_hold.image.to_owned(),
                            buy_price:nft_hold.buy_price.to_owned(),
                            market_fee:nft_hold.market_fee.to_owned(),
                            floor_price:nft_hold.floor_price.to_owned(),
                            gas_fee:nft_hold.gas_fee.to_owned(),
                            unrealized_gains:format!("{}usei",unrealized_gains.to_string()),
                            attributes:nft_hold.attributes.to_owned(),
                            ts:nft_hold.ts.to_owned(),
                            tx_hash:nft_hold.tx_hash.to_owned(),
                        }
                    )
                }
            }
        });
    });

    //降序 排 top_gainers_nfts
    top_gainers_nfts.sort_by_key(|nft|{
        let unrealized_gains=nft.unrealized_gains.clone();
        let unrealized_gains_price=unrealized_gains.get(0..unrealized_gains.len()-4).unwrap().parse::<i64>().unwrap();
        unrealized_gains_price
    });

    //升序 排  top_losser_nfts
    top_losser_nfts.sort_by_key(|nft|{
        let unrealized_gains=nft.unrealized_gains.clone();
        let unrealized_gains_price=unrealized_gains.get(0..unrealized_gains.len()-4).unwrap().parse::<i64>().unwrap();
        unrealized_gains_price
    });
    top_losser_nfts.reverse();


    Ok(get_user_nfts_hold_top_rp::NftHoldTop{
        top_gainers:top_gainers_nfts,
        top_losser:top_losser_nfts
    })
}