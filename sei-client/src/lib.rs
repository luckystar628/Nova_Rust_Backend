pub mod chain_apis;
pub mod data_rp_structs;
pub mod data_feild_structs;
pub mod transaction_sort;
pub mod error;


#[cfg(test)]
mod tests {
    use chain_apis::{get_address_balances, get_address_nfts_hold_by_contract, get_nft_collect_info_by_contract, get_nfts_info_by_contract, get_token_info_by_contract, get_transaction_by_tx};
    use anyhow::Result;
    use data_rp_structs::tx_rp_struct::TransactionData;
    use transaction_sort::Transaction;
    
    use super::*;




    // sei apis tests
    #[tokio::test]
    async fn test_get_transaction_by_tx()  {
        let transaction_data_result=get_transaction_by_tx(None, "6B4A6660CBA59BDD3F7C07AAA4FDECC08EA88CA33883B4372A250C6F509104DA").await;
        println!("{:#?}",transaction_data_result);
    }

    #[tokio::test]
    async fn test_get_nft_collect_info_by_contract()  {
        let nft_collect_info=get_nft_collect_info_by_contract(None, "sei1ts53rl9eqrdjd82hs2em7hv8g6em4xye67z9wxnhdrn4lnf8649sxtww22").await;
        println!("{:#?}",nft_collect_info);
    }

    #[tokio::test]
    async fn test_get_nfts_info_by_contract()  {

        let nfts_info=get_nfts_info_by_contract(None,"sei1ts53rl9eqrdjd82hs2em7hv8g6em4xye67z9wxnhdrn4lnf8649sxtww22", &vec!["1".to_owned(),"2".to_owned()]).await;
        println!("{:#?}",nfts_info)
    }

    #[tokio::test]
    async fn test_get_address_nfts_hold_by_contract() -> Result<()> {
        let holding=get_address_nfts_hold_by_contract(None, "sei1krvjk3r790dcsqkr96ymd44v04w9zz5dlr66z7", "sei1ts53rl9eqrdjd82hs2em7hv8g6em4xye67z9wxnhdrn4lnf8649sxtww22").await;
        println!("{:#?}",holding);
        Ok(())
    }

    
    #[tokio::test]
    async fn test_get_address_token_holds() -> Result<()> {
        
        let token_balances=get_address_balances("sei1krvjk3r790dcsqkr96ymd44v04w9zz5dlr66z7").await;
        println!("{:#?}",token_balances);
        
        Ok(())
    }


    #[tokio::test]
    async fn test_get_token_info_by_contract() -> Result<()> {
        let token_info=get_token_info_by_contract(None, "sei1hrndqntlvtmx2kepr0zsfgr7nzjptcc72cr4ppk4yav58vvy7v3s4er8ed").await;
        println!("{:#?}",token_info);
        Ok(())
    }







    #[tokio::test]
    async fn test_nft_transaction_type() -> Result<()> {

        // 51029AE4B490D51B6F3DAE48D8743D3F252A489F2FDBDBD37023EC3BFACC7D9D
        //  1FC85B3D3171878EE2D848662BADDF2E4063A7E260BC7029816A90E1D1B4381E

        // 6B4A6660CBA59BDD3F7C07AAA4FDECC08EA88CA33883B4372A250C6F509104DA
        // 9C46FA0569917B81A840A172C9DB65F579B293F8C0122034ED3555956F5C62CB
        let transaction_data_result=get_transaction_by_tx(None, "96770624EB351A2158D5048C79D44B424FB47F31B7B04F2FC1F074D11163EAE5").await;
        if transaction_data_result.is_ok(){
            let data= transaction_data_result.unwrap();


            let tx_response=data.get_tx_response();
            let fee=data.get_tx().get_fee();
            let transaction_sender=data.get_tx().get_transaction_sender();
            
            for log in tx_response.logs{
                
                let a=log.transaction_event_type(
                    transaction_sender.to_owned(),
                     fee.to_owned(), 
                     tx_response.timestamp.to_owned(), 
                     tx_response.txhash.to_owned());
                
                println!("{:#?}",a);
                // println!("{:#?}",log.transaction_event_type(transacton_sender.to_owned(),fee.to_owned(),tx_response.timestamp.to_owned(),tx_response.txhash.to_owned()))
            }
        }else {
            println!("{:#?}",transaction_data_result);
        }
        Ok(())
    }

}
