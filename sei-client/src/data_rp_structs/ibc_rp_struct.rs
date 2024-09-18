use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub struct  IbcInfo{
    pub path:String,
    pub base_denom:String,
}
