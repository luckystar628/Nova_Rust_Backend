use std::sync::Arc;
use base64::{Engine as _, engine::general_purpose};
use reqwest::{self, Client};
use anyhow::Result;
use serde_json::{json, Value};
use tokio::sync::Mutex;
use crate::{data_feild_structs::{nft_data_struct::{self, NftAttribute}, token_data_struct}, data_rp_structs::{nft_collect_contract_rp_struct::{self, NFtAllInfo, NftCollectionInfo}, token_contract_rp_struct, tx_rp_struct::{BankTransactionData, EvmTransactionData, NativeTransactionData, StakeTransactionData, TransactionData}}, error::SeiClientErrs};

const ARCHIVE_RPC:&str="https://rest.sei-archive.pacific-1.seinetwork.io";

pub  async fn get_transaction_by_tx<'apis>(
    rpc_url:Option<&'apis str>,
    tx_hash:&'apis str
) -> Result<TransactionData> {
    
    let rpc_url=rpc_url.unwrap_or(ARCHIVE_RPC);
    let url=format!("{}/cosmos/tx/v1beta1/txs/{}",rpc_url,tx_hash);
    let client=Client::new();
    // let client=Arc::new(Client::builder().proxy(reqwest::Proxy::all("http://127.0.0.1:8080").unwrap()).build().unwrap());
    let rp:Value=client.get(url)
                    .send().await?
                    .json().await?;
    let rp_hashmap=rp.as_object().unwrap();            
    if rp_hashmap.get("code").is_some(){
        if rp_hashmap.get("code").unwrap().as_i64().unwrap()== 5{
            return Err(SeiClientErrs::TxhashNotFound.into());
        }
        return Err(SeiClientErrs::Unkonw.into());
    };
    // println!("{:#?}",rp);
    let native_transaction_data_res=serde_json::from_value::<NativeTransactionData>(rp.to_owned());
    let evm_transaction_data_res=serde_json::from_value::<EvmTransactionData>(rp.to_owned());
    let bank_transaction_data_res=serde_json::from_value::<BankTransactionData>(rp.to_owned());
    let stake_transaction_data_res=serde_json::from_value::<StakeTransactionData>(rp);

    if native_transaction_data_res.is_ok(){
        return Ok(TransactionData::Native(native_transaction_data_res.unwrap()));
    }else if evm_transaction_data_res.is_ok() {
        return Ok(TransactionData::Evm(evm_transaction_data_res.unwrap()));
    }else if bank_transaction_data_res.is_ok() {
        return Ok(TransactionData::Bank(bank_transaction_data_res.unwrap()));
    }else if stake_transaction_data_res.is_ok() {
        return Ok(TransactionData::Stake(stake_transaction_data_res.unwrap()));
    }else {
       
        // println!("{:#?}",evm_transaction_data_res);
        // println!("{:#?}",native_transaction_data_res);
        // println!("{:#?}",stake_transaction_data_res);
        return Err(SeiClientErrs::UnkonwTransactionType.into());
    }
}

pub async fn get_nft_collect_info_by_contract<'apis>(
    rpc_url:Option<&'apis str>,
    contract_address:&'apis str
) -> Result<NftCollectionInfo> {
    
    let rpc_url=rpc_url.unwrap_or(ARCHIVE_RPC);
    let client=Arc::new(Client::new());
    let rpc_url=Arc::new(rpc_url.to_owned());
    let contract_address=Arc::new(contract_address.to_owned());
    
    let rps=Arc::new(Mutex::new(Vec::new()));

    // 查询一次 需要进行 3 次 get 请求
    let querys=vec![
        general_purpose::STANDARD.encode(json!({"contract_info":{}}).to_string()),
        general_purpose::STANDARD.encode(json!({"minter":{}}).to_string()),
        general_purpose::STANDARD.encode(json!({"num_tokens":{}}).to_string()),
    ];

    let mut handles=vec![];

    for query in querys{

        let client=Arc::clone(&client);
        let rpc_url=Arc::clone(&rpc_url);
        let contract_address=Arc::clone(&contract_address);
        let rps=Arc::clone(&rps);

        let handle = tokio::spawn(async move {
            let url=format!("{}/cosmwasm/wasm/v1/contract/{}/smart/{}",rpc_url,contract_address,query);
            match client.get(url).send().await {
                Ok(rp)=>{
                    let rp:Value=rp.json().await.unwrap();
                    rps.lock().await.push(rp);
                },
                Err(_)=>{},
            }
        });

        handles.push(handle);
    };

    futures::future::join_all(handles).await;

    let query_rps=rps.lock().await.to_vec();

    let mut name = String::new();
    let mut symbol = String::new();
    let mut creator = String::new();
    let mut nft_nums = String::new();
    query_rps.iter().for_each(|rp|{
            if let Value::Object(map) = rp {
                if let Some(data) = map.get("data") {
                    if let Value::Object(data_map) = data {
                        if let Some(Value::Number(num)) = data_map.get("count") {
                            nft_nums = num.to_string();
                        }
                        if let Some(Value::String(n)) = data_map.get("name") {
                            name = n.clone();
                        }
                        if let Some(Value::String(s)) = data_map.get("symbol") {
                            symbol = s.clone();
                        }
                        if let Some(Value::String(m)) = data_map.get("minter") {
                            creator = m.clone();
                        }
                    }
                }
            }
    });

    Ok(NftCollectionInfo{
        name,
        symbol,
        creator,
        nft_nums
    })
}

pub async fn get_nfts_info_by_contract<'apis>(
    rpc_url:Option<&'apis str>,
    contract_address:&'apis str,
    nfts_id:&'apis Vec<String>
) -> Result<Vec<nft_data_struct::NftInfo>> {

    let rpc_url=rpc_url.unwrap_or(ARCHIVE_RPC);
    let client=Arc::new(Client::new());
    // let client=Arc::new(Client::builder().proxy(reqwest::Proxy::all("http://127.0.0.1:9999").unwrap()).build().unwrap());
    let rpc_url=Arc::new(rpc_url.to_owned());
    let contract_address=Arc::new(contract_address.to_owned());
    let nfts_info=Arc::new(Mutex::new(Vec::new()));

    let mut querys=vec![];
    nfts_id.iter().for_each(|nft_id|{
        querys.push(
            (nft_id.to_owned(),general_purpose::STANDARD.encode(json!({"all_nft_info":{"token_id":nft_id}}).to_string())),
        )
    });

    let mut handles=vec![];
    for (nft_id,query) in querys{

        let client=Arc::clone(&client);
        let rpc_url=Arc::clone(&rpc_url);
        let contract_address=Arc::clone(&contract_address);
        let nfts_info=Arc::clone(&nfts_info);

        let handle=tokio::spawn(async move {
            let url=format!("{}/cosmwasm/wasm/v1/contract/{}/smart/{}",rpc_url,contract_address,query);
            match client.get(url).send().await {
                Ok(rp)=>{
                    let rp:Value=rp.json().await.unwrap();
                    if let Some(data) =rp.get("data")  {
                        if let Ok(nft_all_info) =serde_json::from_value::<NFtAllInfo>(data.to_owned())  {
                            // 查询 链上数据库 nft info
                            match client.get(&nft_all_info.info.token_uri).send().await {
                                Ok(nft_info)=>{
                                    let nft_info:Value=nft_info.json().await.unwrap();
                                    if let Some(attributes) = nft_info.get("attributes") {
                                        let attributes=serde_json::from_value::<Vec<NftAttribute>>(attributes.to_owned());
                                        let name=serde_json::from_value::<String>(nft_info.get("name").unwrap().to_owned());
                                        let image=serde_json::from_value::<String>(nft_info.get("image").unwrap().to_owned());
                                        if attributes.is_ok() && name.is_ok() && image.is_ok(){
                                            nfts_info.lock().await.push(
                                                nft_data_struct::NftInfo{
                                                    token_id:nft_id.clone(),
                                                    name:format!("{}#{}",name.unwrap(),nft_id.to_owned()),
                                                    key:format!("{}-{}",contract_address,nft_id),
                                                    image:image.unwrap(),
                                                    royalty_percentage:nft_all_info.info.extension.royalty_percentage,
                                                    attributes:attributes.unwrap()
                                                }
                                            );
                                        }
                                    }
                                },
                                _=>{},
                            }
                        }
                    }
                },
                _=>{}
            }
        });
        handles.push(handle);
    };

    futures::future::join_all(handles).await;
    let nfts_info=nfts_info.lock().await.to_vec();
    Ok(nfts_info)
}


pub async fn get_address_nfts_hold_by_contract<'apis>(
    rpc_url:Option<&'apis str>,
    wallet_address:&'apis str,
    contract_address:&'apis str
) -> Result<nft_data_struct::NftCollectHold> {

    let rpc_url=rpc_url.unwrap_or(ARCHIVE_RPC);
    let client=Client::new();
    // let client=Arc::new(Client::builder().proxy(reqwest::Proxy::all("http://127.0.0.1:9999").unwrap()).build().unwrap());

    let query=general_purpose::STANDARD.encode(json!({"tokens":{"owner":wallet_address}}).to_string());
    let url=format!("{}/cosmwasm/wasm/v1/contract/{}/smart/{}",rpc_url,contract_address,query);

    let rp:Value=client.get(url)
                    .send().await?
                    .json().await?;
    
    if rp.get("code").is_some(){
        let code=rp.get("code").unwrap().as_i64().unwrap();
        if code == 3{
            return Err(SeiClientErrs::NftCollectNotHaveAddressHold.into());
        }else {
            return Err(SeiClientErrs::Unkonw.into());
        }
    };

    let nfts_hold=serde_json::from_value::<nft_collect_contract_rp_struct::NftsHold>(rp.get("data").unwrap().to_owned()).unwrap();
    if let Ok((nft_collect_info,nfts_info))=tokio::try_join!(get_nft_collect_info_by_contract(Some(rpc_url), contract_address),get_nfts_info_by_contract(Some(rpc_url), contract_address, &nfts_hold.tokens)){
        Ok(nft_data_struct::NftCollectHold{
            collect_address:contract_address.to_owned(),
            collect_info:nft_collect_info,
            nfts_hold:nfts_info
        })
    }else {
        Err(SeiClientErrs::GetNftInfoErro.into())
    }
    
}


pub async fn get_address_balances<'apis>(
    wallet_address:&'apis str
) -> Result<Vec<token_contract_rp_struct::Token>>{

    let client=Client::new();
    let url=format!("https://celatone-api-prod.alleslabs.dev/v1/sei/pacific-1/accounts/{}/balances",wallet_address);
    
    let rp:Value=client.get(url)
                       .send().await?
                       .json().await?;

    
    Ok(serde_json::from_value::<Vec<token_contract_rp_struct::Token>>(rp)?)
}


pub async fn get_token_info_by_contract<'apis>(
    rpc_url:Option<&'apis str>,
    token_contract_address:&'apis str
) -> Result<token_data_struct::TokenInfo> {

    let rpc_url=rpc_url.unwrap_or(ARCHIVE_RPC);
    let querys=vec![
        general_purpose::STANDARD.encode(json!({"token_info":{}}).to_string()),
        general_purpose::STANDARD.encode(json!({"marketing_info":{}}).to_string()),
        general_purpose::STANDARD.encode(json!({"minter":{}}).to_string()),
    ];

    let client=Arc::new(Client::new());
    let marketing_info=Arc::new(Mutex::new(Vec::new()));
    let token_info=Arc::new(Mutex::new(Vec::new()));
    let token_minter=Arc::new(Mutex::new(Vec::new()));

    let mut handles=Vec::new();

    for query_index in 0..3{
        let client=Arc::clone(&client);
        let token_info=Arc::clone(&token_info);
        let token_minter=Arc::clone(&token_minter);
        let marketing_info=Arc::clone(&marketing_info);
        let query=querys[query_index].to_owned();

        
        
        match query_index {
            0_usize=>{
                let url=format!("{}/cosmwasm/wasm/v1/contract/{}/smart/{}",rpc_url,token_contract_address,query);
                
                let handle=tokio::spawn(async move{
                    match client.get(url).send().await{
                        Ok(rp)=>{
                            let rp:Value=rp.json().await.unwrap();
                            if rp.get("data").is_some(){
                                let data=serde_json::from_value::<token_contract_rp_struct::_TokenInfo>(rp.get("data").unwrap().to_owned()).unwrap();
                                token_info.lock().await.push(data);
                            }
                           
                        },
                        Err(_)=>{},
                    }
                });
                handles.push(handle);
            },
            1_usize=>{
                let url=format!("{}/cosmwasm/wasm/v1/contract/{}/smart/{}",rpc_url,token_contract_address,query);
                let handle=tokio::spawn(async move{
                    match client.get(url).send().await{
                        Ok(rp)=>{
                            let rp:Value=rp.json().await.unwrap();
                            if rp.get("data").is_some(){
                                let data=serde_json::from_value::<token_contract_rp_struct::TokenMarketingInfo>(rp.get("data").unwrap().to_owned()).unwrap();
                                marketing_info.lock().await.push(data);
                            }
                           
                        },
                        Err(_)=>{},
                    }
                });
                handles.push(handle);
            },
            2_usize=>{
                let url=format!("{}/cosmwasm/wasm/v1/contract/{}/smart/{}",rpc_url,token_contract_address,query);
                let handle=tokio::spawn(async move{
                    match client.get(url).send().await{
                        Ok(rp)=>{
                            let rp:Value=rp.json().await.unwrap();
                            if rp.get("data").is_some(){
                                let data=serde_json::from_value::<token_contract_rp_struct::TokenMinter>(rp.get("data").unwrap().to_owned()).unwrap();
                                token_minter.lock().await.push(data);
                            }
                           
                        },
                        Err(_)=>{},
                    }
                });
                handles.push(handle);
            },
            _=>{},
        };
    };

    futures::future::join_all(handles).await;
    let token_minter_info=token_minter.lock().await.to_vec();
    let token_marketing_info=marketing_info.lock().await.to_vec();
    let token_info=token_info.lock().await.to_vec();
    
    if token_minter_info.len()!=1{
        return Err(SeiClientErrs::GetTokenMinterInfoByContractErr.into());
    }else if token_marketing_info.len()!=1 {
        return Err(SeiClientErrs::GetTokenMarketingInfoByContractErr.into());
    }else if token_info.len()!=1 {
        return Err(SeiClientErrs::GetTokeninfoByContractErr.into());
    }else if token_marketing_info.len()==1 && token_info.len()==1 && token_minter_info.len()==1 {

        let token_marketing_info=token_marketing_info[0].to_owned();
        let token_info=token_info[0].to_owned();
        let token_minter_info=token_minter_info[0].to_owned();
        Ok(token_data_struct::TokenInfo{
            name:token_info.name,
            symbol:token_info.symbol,
            project:token_marketing_info.project,
            description:token_marketing_info.description,
            decimals:token_info.decimals,
            total_supply:token_info.total_supply,
            minter:token_minter_info.minter,
            market:token_marketing_info.marketing,
            logo_url:token_marketing_info.logo.url
        })
    }else {
        return Err(SeiClientErrs::GetTokeninfoByContractErr.into());
    }
}