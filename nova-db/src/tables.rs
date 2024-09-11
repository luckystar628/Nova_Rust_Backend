use sei_client::data_feild_structs::{nft_data_struct, stake_data_sturct, token_data_struct};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, Row};


#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub struct  WalletInfo{
    pub wallet_address:String,
    pub nft_hold:Vec<nft_data_struct::NftCollectHold>,
    pub nft_transactions:Vec<nft_data_struct::NftTransaction>,
    pub token_transactions:Vec<token_data_struct::TokenTransaction>,
    pub stake_transactions:Vec<stake_data_sturct::Stake>
}

impl<'r> sqlx::FromRow<'r, PgRow> for WalletInfo {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        let wallet_address: String = row.try_get("wallet_address")?;
        let nft_hold_json: Option<serde_json::Value> = row.try_get("nft_hold")?;
        let nft_transactions_json: Option<serde_json::Value> = row.try_get("nft_transactions")?;
        let token_transactions_json: Option<serde_json::Value> = row.try_get("token_transactions")?;
        let stake_transactions_json: Option<serde_json::Value> = row.try_get("stake_transactions")?;
        
        let nft_hold = match nft_hold_json {
            Some(json) => serde_json::from_value(json).unwrap_or_default(),
            None => vec![], // 默认空 Vec
        };
        let nft_transactions = match nft_transactions_json {
            Some(json) => serde_json::from_value(json).unwrap_or_default(),
            None => vec![], // 默认空 Vec
        };
        let token_transactions = match token_transactions_json {
            Some(json) => serde_json::from_value(json).unwrap_or_default(),
            None => vec![], // 默认空 Vec
        };
        let stake_transactions = match stake_transactions_json {
            Some(json) => serde_json::from_value(json).unwrap_or_default(),
            None => vec![], // 默认空 Vec
        };
        Ok(Self {
            wallet_address,
            nft_hold,
            nft_transactions,
            token_transactions,
            stake_transactions,
        })
    }
}

#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub struct NftContract{
    pub collection_address:String,
    pub collection_floor_price:Option<CollectionFloorPrice>,
    pub nfts_floor_price:Vec<NftFloorPrice>,
}

impl<'r> sqlx::FromRow<'r, PgRow> for NftContract {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        let collection_address:String=row.try_get("collection_address")?;
        let collection_floor_price:Option<serde_json::Value>=row.try_get("collection_floor_price")?;
        let nfts_floor_price:Option<serde_json::Value>=row.try_get("nfts_floor_price")?;

        let collection_floor_price=match collection_floor_price {
            Some(json)=>Some(serde_json::from_value::<CollectionFloorPrice>(json).unwrap_or_default()),
            None=>None,
        };

        let nfts_floor_price=match nfts_floor_price {
            Some(json)=>serde_json::from_value::<Vec<NftFloorPrice>>(json).unwrap_or_default(),
            None=>vec![],
        };

        Ok(Self {
            collection_address,
            collection_floor_price,
            nfts_floor_price
        })
    }
}


#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq,Default)]
pub struct CollectionFloorPrice{
    pub floor_price:String,
    pub ts:String,
}

#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq,Default)]
pub struct NftFloorPrice{
    pub nft_key:String,
    pub floor_price:String,
    pub ts:String,
}