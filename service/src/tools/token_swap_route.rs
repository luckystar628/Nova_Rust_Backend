use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub struct TokenRouteData{
        id:String,
    pub swaps:Vec<SwapData>,
    pub denom_in:String,
    pub decimals_in:u8,
    pub price_in:Value,
    pub value_in:Value,
    pub amount_in:Value,
    pub denom_out:Value,
    pub decimals_out:u8,
    pub price_out:Value,
    pub value_out:Value,
    pub amount_out:String,
    pub price_difference:Value,
    pub price_impact :Value,
}

#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub struct SwapData{
    contract_addr:String,
    from:String,
    to:String,
    #[serde(rename = "type")]
    _type:String,
    illiquid:bool,
}



pub async fn token_routes(source_token:&str,target_source:&str,amount:usize)->Result<Vec<TokenRouteData>> {
    
    let url=format!("https://sei.astroport.fi/api/routes?start={}&end={}&amount={}&chainId=pacific-1&limit=1",source_token,target_source,amount);
    let client=Client::new();
    let data_rp:Value=client.get(url)
        .send().await?
        .json().await?;


    let data=serde_json::from_value::<Vec<TokenRouteData>>(data_rp);
    Ok(data?)
}