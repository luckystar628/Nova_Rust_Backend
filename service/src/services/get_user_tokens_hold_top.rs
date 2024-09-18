use anyhow::Result;
use sqlx::PgPool;

use crate::responses::get_user_tokens_hold_rp;

use super::get_user_tokens_hold;

pub async fn take<'services>(
    wallet_address:&'services str,
    conn_pool:&'services PgPool,
) -> Result<get_user_tokens_hold_rp::UserTokenTop> {

    let user_tokens_hold={
        let x=get_user_tokens_hold::take(wallet_address, conn_pool).await;
        if x.is_err(){
            return  Err(x.err().unwrap());
        };
        x.unwrap()
    };

    let mut top_gainers_tokens:Vec<get_user_tokens_hold_rp::UserTokenHold>=Vec::new();
    let mut top_losser_tokens:Vec<get_user_tokens_hold_rp::UserTokenHold>=Vec::new();

    user_tokens_hold.iter().for_each(|user_token_hold|{
        let amount=user_token_hold.amount.to_owned().parse::<isize>().unwrap();
        let buy_price=user_token_hold.buy_price.to_owned().unwrap().parse::<isize>().unwrap();

        if let Some(worth_usei)=user_token_hold.worth_usei.to_owned(){
            let worth_usei=worth_usei.parse::<isize>().unwrap();
            let buy_cost=amount*buy_price;
            
            if worth_usei-buy_cost<0{
                top_losser_tokens.push(user_token_hold.to_owned());
            }else {
                top_gainers_tokens.push(user_token_hold.to_owned());
            }
        }
    });

    top_gainers_tokens.sort_by_key(|token|{
        let amount=token.amount.to_owned().parse::<isize>().unwrap();
        let buy_price=token.buy_price.to_owned().unwrap().parse::<isize>().unwrap();
        let worth_usei=token.worth_usei.to_owned().unwrap().parse::<isize>().unwrap();

        let ug=worth_usei-(buy_price*amount);
        ug
    });


    top_losser_tokens.sort_by_key(|token|{
        let amount=token.amount.to_owned().parse::<isize>().unwrap();
        let buy_price=token.buy_price.to_owned().unwrap().parse::<isize>().unwrap();
        let worth_usei=token.worth_usei.to_owned().unwrap().parse::<isize>().unwrap();

        let ug=worth_usei-(buy_price*amount);
        ug
    });
    top_losser_tokens.reverse();
    
    Ok(get_user_tokens_hold_rp::UserTokenTop{
        top_gainers_tokens:top_gainers_tokens,
        top_losser_tokens:top_losser_tokens
    })
}