use anyhow::Result;
use sei_client::data_feild_structs::nft_data_struct;
use sqlx::PgPool;
use crate::erros::ServicesErrs;


pub async fn take<'services>(
    wallet_address:&'services str,
    conn_pool:&'services PgPool
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
    
    let nft_transactions=user_wallet.nft_transactions;

    // set nft buy transactions
    let mut nft_buy_transactions:Vec<nft_data_struct::NftTransaction>=Vec::new();
    // set nft sell transactions
    let mut nft_sell_transactions:Vec<nft_data_struct::NftTransaction>=Vec::new();

    // push  sell and buy transaction to vec from nft_transactions
    nft_transactions.iter().for_each(|nft_transaction|{
        match nft_transaction {
            nft_data_struct::NftTransaction::Mint(t) => {
                if &t.recipient==wallet_address{
                    nft_buy_transactions.push(
                        nft_data_struct::NftTransaction::Mint(t.to_owned())
                    );
                }
            },
            nft_data_struct::NftTransaction::BatchBids(t) =>{
                if &t.recipient==wallet_address{
                    nft_buy_transactions.push(
                        nft_data_struct::NftTransaction::BatchBids(t.to_owned())
                    )
                };
                if &t.sender==wallet_address{
                    nft_sell_transactions.push(
                        nft_data_struct::NftTransaction::BatchBids(t.to_owned())
                    )
                }
            },
            nft_data_struct::NftTransaction::Transfer(t) => {
                if &t.recipient==wallet_address{
                    nft_buy_transactions.push(nft_data_struct::NftTransaction::Transfer(t.to_owned()));
                };
                if &t.sender==wallet_address{
                    nft_sell_transactions.push(nft_data_struct::NftTransaction::Transfer(t.to_owned()));
                }
            },
            nft_data_struct::NftTransaction::FixedSell(t) => {
                if &t.recipient==wallet_address{
                    nft_buy_transactions.push(nft_data_struct::NftTransaction::FixedSell(t.to_owned()));
                };
                if &t.sender==wallet_address{
                    nft_sell_transactions.push(nft_data_struct::NftTransaction::FixedSell(t.to_owned()));
                }
            },
            nft_data_struct::NftTransaction::PurchaseCart(t) => {
                if &t.recipient==wallet_address{
                    nft_buy_transactions.push(nft_data_struct::NftTransaction::PurchaseCart(t.to_owned()));
                };
                if &t.sender==wallet_address{
                    nft_sell_transactions.push(nft_data_struct::NftTransaction::PurchaseCart(t.to_owned()));
                }
            },
            nft_data_struct::NftTransaction::AcceptBid(t) => {
                if &t.recipient==wallet_address{
                    nft_buy_transactions.push(nft_data_struct::NftTransaction::AcceptBid(t.to_owned()));
                };
                if &t.sender==wallet_address{
                    nft_sell_transactions.push(nft_data_struct::NftTransaction::AcceptBid(t.to_owned()));
                }
            },
            nft_data_struct::NftTransaction::CretaeAuction(t) => {
                if &t.recipient==wallet_address{
                    nft_buy_transactions.push(nft_data_struct::NftTransaction::CretaeAuction(t.to_owned()));
                };
                if &t.sender==wallet_address{
                    nft_sell_transactions.push(nft_data_struct::NftTransaction::CretaeAuction(t.to_owned()));
                }
            },
            _=>{},
        }
    });

    if  nft_buy_transactions.is_empty() || nft_sell_transactions.is_empty(){
        return  Err(ServicesErrs::NFTsTransactionsIsNone.into());
    }




    Ok(())
}