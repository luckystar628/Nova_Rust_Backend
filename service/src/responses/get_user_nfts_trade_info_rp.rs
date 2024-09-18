use std::collections::HashMap;
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use super::get_user_nfts_hold_rp::UserNFTHold;




#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub struct AgeOfNftAssets{
    pub level1: Vec<UserNFTHold>,
    pub level2:Vec<UserNFTHold>,
    pub level3:Vec<UserNFTHold>,
    pub level4:Vec<UserNFTHold>,
    pub level5:Vec<UserNFTHold>,
    pub level6:Vec<UserNFTHold>,
}

#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub struct NFTTransactionStatistic{
    pub transaction_amount:usize,
    pub total_volume:String,
}


#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub struct NFTTradeVolume{
    pub buy_volume:NFTTransactions,
    pub sell_volume:NFTTransactions,
}

#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub struct TradeInfo{
    pub sale_price:String,
    pub ts:String,
}

#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub struct NFTTransactions{
    day:HashMap<String,NFTTransactionStatistic>,
    week:HashMap<String,NFTTransactionStatistic>,
    month:HashMap<String,NFTTransactionStatistic>,  //(yaer month)
}impl NFTTransactions {
    pub fn new()->Self {
        NFTTransactions{
            day:HashMap::new(),
            week:HashMap::new(),
            month:HashMap::new(),
        }
    }

    pub fn add_data(&mut self,trades_info:Vec<TradeInfo>) {

        // 天分组 
        let mut day_trades:HashMap<NaiveDate,Vec<TradeInfo>>=HashMap::new();
        //周分组
        let mut week_trades:HashMap<(i32,u32),Vec<TradeInfo>>=HashMap::new();
        //月分组
        let mut month_trades:HashMap<(i32,u32),Vec<TradeInfo>>=HashMap::new();


        // 分类填充
        trades_info.iter().for_each(|trade_info|{
            let date:NaiveDateTime=DateTime::parse_from_rfc3339(&trade_info.ts).unwrap().with_timezone(&Utc).naive_utc();
            
            day_trades.entry(date.clone().date()).or_insert_with(Vec::new).push(trade_info.to_owned());

            let iso_week=date.iso_week();
            week_trades.entry((iso_week.year(),iso_week.week())).or_insert_with(Vec::new).push(trade_info.to_owned());

            month_trades.entry((date.year(),date.month())).or_insert_with(Vec::new).push(trade_info.to_owned());
        });

        day_trades.into_iter().for_each(|(key,value)|{
            let transaction_amount=value.len();
            let mut sale_price:Vec<usize>=vec![];
            value.iter().for_each(|trade|{
                let volume = trade.sale_price.clone().get(0..trade.sale_price.clone().len()-4).unwrap().parse::<usize>().unwrap();
                sale_price.push(volume);
            });

            if sale_price.len()>0{
                let volume:usize=sale_price.iter().sum();
                let total_volume=format!("{}usei",volume.to_string());
                self.day.entry(key.clone().to_string()).or_insert_with(||NFTTransactionStatistic{transaction_amount:transaction_amount.clone(),total_volume:total_volume});
            }else {
                let total_volume="0usei".to_string();
                self.day.entry(key.clone().to_string()).or_insert_with(||NFTTransactionStatistic{transaction_amount:transaction_amount.clone(),total_volume:total_volume});
            }
            
        });

        week_trades.into_iter().for_each(|(key,value)|{
            let transaction_amount=value.len();
            let mut sale_price:Vec<usize>=vec![];
            value.iter().for_each(|trade|{
                let volume = trade.sale_price.clone().get(0..trade.sale_price.clone().len()-4).unwrap().parse::<usize>().unwrap();
                sale_price.push(volume);
            });

            if sale_price.len()>0{
                let volume:usize=sale_price.iter().sum();
                let total_volume=format!("{}usei",volume.to_string());
                self.week.entry(format!("{}-{}",key.clone().0.to_string(),key.clone().1.to_string())).or_insert_with(||NFTTransactionStatistic{transaction_amount:transaction_amount.clone(),total_volume:total_volume});
            }else {
                let total_volume="0usei".to_string();
                self.week.entry(format!("{}-{}",key.clone().0.to_string(),key.clone().1.to_string())).or_insert_with(||NFTTransactionStatistic{transaction_amount:transaction_amount.clone(),total_volume:total_volume});
            }
            
        });

        month_trades.into_iter().for_each(|(key,value)|{
            let transaction_amount=value.len();
            let mut sale_price:Vec<usize>=vec![];
            value.iter().for_each(|trade|{
                let volume = trade.sale_price.clone().get(0..trade.sale_price.clone().len()-4).unwrap().parse::<usize>().unwrap();
                sale_price.push(volume);
            });

            if sale_price.len()>0{
                let volume:usize=sale_price.iter().sum();
                let total_volume=format!("{}usei",volume.to_string());
                self.month.entry(format!("{}-{}",key.clone().0.to_string(),key.clone().1.to_string())).or_insert_with(||NFTTransactionStatistic{transaction_amount:transaction_amount.clone(),total_volume:total_volume});
            }else {
                let total_volume="0usei".to_string();
                self.month.entry(format!("{}-{}",key.clone().0.to_string(),key.clone().1.to_string())).or_insert_with(||NFTTransactionStatistic{transaction_amount:transaction_amount.clone(),total_volume:total_volume});
            }
            
        });
    }
}




#[derive(Serialize, Deserialize,Clone,Debug,PartialEq, Eq)]
pub struct NFTTradeInfo{
    pub age_of_nft_assets:Option<AgeOfNftAssets>,
    pub transaction:NFTTransactions,
    pub volume:NFTTradeVolume,

}