mod responses;
mod tools;
mod erros;
pub mod services;

#[cfg(test)]
mod tests{
    use anyhow::Result;

    use crate::{services::sync_user_trade_histroy_data, tools::token_swap_route};
    #[tokio::test]
    async fn test_get_user_nfts_hold() -> Result<()> {
       let from="usei";
       let to="ibc/CA6FBFAF399474A06263E10D0CE5AEBBE15189D6D4B2DD9ADE61007E68EB9DB0";

       let conn_pool=nova_db::create_db_conn_pool().await.unwrap();
       let wallet_address="sei1krvjk3r790dcsqkr96ymd44v04w9zz5dlr66z7";
       let data=sync_user_trade_histroy_data::take(wallet_address, &conn_pool).await;
    //    let data=token_swap_route::token_routes(from, to, 1000000).await;
    //    println!("{:#?}",data);
       
       
       Ok(())
    }
}
