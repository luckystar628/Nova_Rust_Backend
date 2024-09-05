use serde::{Deserialize, Serialize};
use serde_json::Value;


// nft collection 信息
#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq,sqlx::Type)]
pub struct NftCollectionInfo{
    pub name:String,
    pub symbol:String,
    pub creator:String,
    pub nft_nums :String,
}



// nft 的详细info
#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct NFtAllInfo{
    pub access:Access,
    pub info:NftInfo,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Access{
        approvals:Value,
    pub owner:String
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct NftInfo{
    pub extension:Extension,
    pub token_uri:String,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Extension{
        animation_url:Value,
        attributes:Value,
        background_color:Value,
        description:Value,
        external_url:Value,
        image:Value,
        image_data:Value,
        name:Value,
        royalty_payment_address:Value,
    pub royalty_percentage:u64,
        youtube_url :Value
}



// 请求 nft collect中的address 持有
#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct NftsHold{
    pub tokens:Vec<String>
}
