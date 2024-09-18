use std::{collections::HashMap, mem::MaybeUninit};
use anyhow::Result;
use sei_client::chain_apis;
use sqlx::PgPool;

use crate::{erros::ServicesErrs, responses::get_user_tokens_hold_rp, tools::token_swap_route};

pub async fn take<'services>(
    wallet_address:&'services str,
    conn_pool:&'services PgPool,
) -> Result<Vec<get_user_tokens_hold_rp::UserTokenHold>> {
    
    let wallet_balances={
        let x=chain_apis::get_address_balances(wallet_address).await;
        if x.is_err(){
            return Err(x.err().unwrap());
        };
        x.unwrap()
    };

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


    let mut tokens_transactions:HashMap<String,Vec<sei_client::data_feild_structs::token_data_struct::TokenTransaction>>=HashMap::new();

    let token_demos_amount={
        let mut x: HashMap<String,String>=HashMap::new();
        wallet_balances.iter().for_each(|token|{
            x.insert(token.denom.to_owned(),token.amount.to_owned());
        });
        x
    };
    //分类 token transaction
    let token_transactions=user_wallet.token_transactions;
       
    token_transactions.iter().for_each(|token_transaction|{
        
        match token_transaction {
            sei_client::data_feild_structs::token_data_struct::TokenTransaction::TokenSwap(t) =>{
                if token_demos_amount.get(&t.target_token).is_some(){
                    tokens_transactions.entry(t.target_token.to_owned())
                                   .or_insert_with(Vec::new)
                                   .push(sei_client::data_feild_structs::token_data_struct::TokenTransaction::TokenSwap(t.to_owned()))
                }
            },
            sei_client::data_feild_structs::token_data_struct::TokenTransaction::TokenTransfer(t) => {

                tokens_transactions.entry("usei".to_string())
                                   .or_insert_with(Vec::new)
                                   .push(sei_client::data_feild_structs::token_data_struct::TokenTransaction::TokenTransfer(t.to_owned()));
            },
            sei_client::data_feild_structs::token_data_struct::TokenTransaction::ContractTokenTransfer(t) => {
                if token_demos_amount.get(&t.contract_address).is_some(){
                    tokens_transactions.entry(t.contract_address.to_owned())
                    .or_insert_with(Vec::new)
                    .push(sei_client::data_feild_structs::token_data_struct::TokenTransaction::ContractTokenTransfer(t.to_owned()))
                }
            },
        }
    });


    let mut user_tokens_hold:Vec<get_user_tokens_hold_rp::UserTokenHold>=Vec::new();

    for (token_demon,_) in &tokens_transactions{
        if token_demon=="usei"{
            user_tokens_hold.push(get_user_tokens_hold_rp::UserTokenHold{
                name:"usei".to_string(),
                demon:"usei".to_string(),
                decimals:Some(6),
                logo_url:Some(String::from("https://raw.githubusercontent.com/astroport-fi/astroport-token-lists/main/img/sei.png")),
                amount:token_demos_amount.get("usei").unwrap().to_owned(),
                worth_usei:Some(token_demos_amount.get("usei").unwrap().to_owned()),
                buy_price:None
            });
        }else if token_demon.get(..7).unwrap()=="factory"{
            
            let token_demon_transactions=tokens_transactions.get(token_demon).unwrap().to_owned();
            
            // token_demon_transactions.sort_by(|a,b|{
                
            //     let a_ts_str=match a {
            //         sei_client::data_feild_structs::token_data_struct::TokenTransaction::TokenSwap(t) => &t.ts,
            //         sei_client::data_feild_structs::token_data_struct::TokenTransaction::ContractTokenTransfer(t) =>&t.ts,
            //         sei_client::data_feild_structs::token_data_struct::TokenTransaction::TokenTransfer(t) => &t.ts,
            //     };

            //     let b_ts_str=match b {
            //         sei_client::data_feild_structs::token_data_struct::TokenTransaction::TokenSwap(t) => &t.ts,
            //         sei_client::data_feild_structs::token_data_struct::TokenTransaction::ContractTokenTransfer(t) =>&t.ts,
            //         sei_client::data_feild_structs::token_data_struct::TokenTransaction::TokenTransfer(t) => &t.ts,
            //     };

            //     let a_time=DateTime::parse_from_rfc3339(a_ts_str).unwrap().with_timezone(&Utc);
            //     let b_time=DateTime::parse_from_rfc3339(b_ts_str).unwrap().with_timezone(&Utc);

            //     b_time.cmp(&a_time)
            // });


            let demon={
                let mut indexs:Vec<usize>=vec![];
                let demon_vec:Vec<char>=token_demon.chars().collect();
                demon_vec.iter().enumerate().for_each(|(index,t)|{
                    if t.to_string()=="/".to_string(){
                        indexs.push(index)
                    }
                });
                token_demon.get(indexs[1]+1..).unwrap()
            };

            let buy_price={

                let mut token_amount:usize=0;
                let mut usei:usize=0;

                token_demon_transactions.iter().for_each(|token_demon_transaction|{
                    match token_demon_transaction {
                        sei_client::data_feild_structs::token_data_struct::TokenTransaction::TokenSwap(t) =>{
                            if &t.target_token==token_demon&& &t.source_token=="usei"{
                                let target_amount=t.target_amount.parse::<usize>().unwrap();
                                let source_amount=t.source_amount.parse::<usize>().unwrap();
                                token_amount+=target_amount;
                                usei+=source_amount;
                            }
                        },
                        _=>{},
                    }
                });

                if usei ==0{
                    0
                }else {
                    token_amount /usei
                }
            };

            let demon_amount=token_demos_amount.get(token_demon).unwrap();

            let mut decimals_ptr:MaybeUninit<Option<u8>>=MaybeUninit::new(None);
            let mut logo_url_ptr:MaybeUninit<Option<String>>=MaybeUninit::new(None);
            let mut worth_usei_ptr:MaybeUninit<Option<String>>=MaybeUninit::new(None);

            let token_route_data=token_swap_route::token_routes(token_demon, "usei", demon_amount.parse::<usize>().unwrap()).await;

            if token_route_data.is_ok(){
                let swap_info=token_route_data.unwrap()[0].to_owned();
                decimals_ptr.write(Some(swap_info.decimals_in));
                worth_usei_ptr.write(Some(swap_info.amount_out));
            }

            unsafe{
                user_tokens_hold.push(
                    get_user_tokens_hold_rp::UserTokenHold{
                        name:demon.to_string(),
                        demon:token_demon.to_string(),
                        decimals:decimals_ptr.assume_init_read(),
                        logo_url:logo_url_ptr.assume_init_read(),
                        amount:demon_amount.to_string(),
                        worth_usei:worth_usei_ptr.assume_init_read(),
                        buy_price:Some(format!("{}usei",buy_price.to_string())),
                    }
                )
            }

            
        }else if token_demon.get(..3).unwrap()=="ibc" {
            
            let demon_amount=token_demos_amount.get(token_demon).unwrap();

            let buy_price={

                let mut token_amount:usize=0;
                let mut usei:usize=0;

                let token_demon_transactions=tokens_transactions.get(token_demon).unwrap().to_owned();
                token_demon_transactions.iter().for_each(|token_demon_transaction|{
                    match token_demon_transaction {
                        sei_client::data_feild_structs::token_data_struct::TokenTransaction::TokenSwap(t) =>{
                            if &t.target_token==token_demon&& &t.source_token=="usei"{
                                let target_amount=t.target_amount.parse::<usize>().unwrap();
                                let source_amount=t.source_amount.parse::<usize>().unwrap();
                                token_amount+=target_amount;
                                usei+=source_amount;
                            }
                        },
                        _=>{},
                    }
                });

                if usei ==0{
                    0
                }else {
                    token_amount /usei
                }
            };

            let mut name_ptr:MaybeUninit<String>=MaybeUninit::new(token_demon.to_owned());
            let mut demon_ptr:MaybeUninit<String>=MaybeUninit::new(token_demon.to_owned());
            let mut decimals_ptr:MaybeUninit<Option<u8>>=MaybeUninit::new(None);
            let mut logo_url_ptr:MaybeUninit<Option<String>>=MaybeUninit::new(None);
            let mut worth_usei_ptr:MaybeUninit<Option<String>>=MaybeUninit::new(None);

            let ibc_info=chain_apis::get_ibc_info(Some(chain_apis::SERVER_RPC), token_demon).await;
            if ibc_info.is_ok(){
                let ibc_info=ibc_info.unwrap();
                demon_ptr.write(ibc_info.base_denom.to_owned());
                name_ptr.write(ibc_info.base_denom);
            };

            let token_route_data=token_swap_route::token_routes(token_demon, "usei", demon_amount.parse::<usize>().unwrap()).await;
            if token_route_data.is_ok(){
                let swap_info=token_route_data.unwrap()[0].to_owned();
                decimals_ptr.write(Some(swap_info.decimals_in));
                worth_usei_ptr.write(Some(swap_info.amount_out));
            };

            unsafe{
                user_tokens_hold.push(
                    get_user_tokens_hold_rp::UserTokenHold{
                        name:name_ptr.assume_init_read(),
                        demon:demon_ptr.assume_init_read(),
                        decimals:decimals_ptr.assume_init_read(),
                        logo_url:logo_url_ptr.assume_init_read(),
                        amount:demon_amount.to_string(),
                        worth_usei:worth_usei_ptr.assume_init_read(),
                        buy_price:Some(format!("{}usei",buy_price.to_string())),
                    }
                )
            }

        }else {
            let demon_amount=token_demos_amount.get(token_demon).unwrap();

            let buy_price={

                let mut token_amount:usize=0;
                let mut usei:usize=0;

                let token_demon_transactions=tokens_transactions.get(token_demon).unwrap().to_owned();
                token_demon_transactions.iter().for_each(|token_demon_transaction|{
                    match token_demon_transaction {
                        sei_client::data_feild_structs::token_data_struct::TokenTransaction::TokenSwap(t) =>{
                            if &t.target_token==token_demon&& &t.source_token=="usei"{
                                let target_amount=t.target_amount.parse::<usize>().unwrap();
                                let source_amount=t.source_amount.parse::<usize>().unwrap();
                                token_amount+=target_amount;
                                usei+=source_amount;
                            }
                        },
                        _=>{},
                    }
                });

                if usei ==0{
                    0
                }else {
                    token_amount /usei
                }
            };

            
            let mut name_ptr:MaybeUninit<String>=MaybeUninit::new(token_demon.to_owned());
            let mut demon_ptr:MaybeUninit<String>=MaybeUninit::new(token_demon.to_owned());
            let mut decimals_ptr:MaybeUninit<Option<u8>>=MaybeUninit::new(None);
            let mut logo_url_ptr:MaybeUninit<Option<String>>=MaybeUninit::new(None);
            let mut worth_usei_ptr:MaybeUninit<Option<String>>=MaybeUninit::new(None);

            if let Ok(token_smart_contract_info) =chain_apis::get_token_info_by_contract(Some(chain_apis::SERVER_RPC), token_demon).await  {
                name_ptr.write(token_smart_contract_info.name.to_owned());
                decimals_ptr.write(Some(token_smart_contract_info.decimals.to_owned()));
                logo_url_ptr.write(Some(token_smart_contract_info.logo_url.to_owned()));
            }

            let token_route_data=token_swap_route::token_routes(token_demon, "usei", demon_amount.parse::<usize>().unwrap()).await;
            if token_route_data.is_ok(){
                let swap_info=token_route_data.unwrap()[0].to_owned();
                worth_usei_ptr.write(Some(swap_info.amount_out));
            };


            unsafe{
                user_tokens_hold.push(
                    get_user_tokens_hold_rp::UserTokenHold{
                        name:name_ptr.assume_init_read(),
                        demon:demon_ptr.assume_init_read(),
                        decimals:decimals_ptr.assume_init_read(),
                        logo_url:logo_url_ptr.assume_init_read(),
                        amount:demon_amount.to_string(),
                        worth_usei:worth_usei_ptr.assume_init_read(),
                        buy_price:Some(format!("{}usei",buy_price.to_string())),
                    }
                )
            }
        }

    }
    

    Ok(user_tokens_hold)
}