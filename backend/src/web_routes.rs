use actix_web::{get, http::header, post, web::{self, Data}, HttpRequest, HttpResponse, Responder, Result};
use serde::{Deserialize, Serialize};
use service::services;
use sqlx::PgPool;



#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Wallet{
    pub wallet_address:String,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct DontHaveData{
    pub wallet_address:String,
    pub result:Option<String>
}



#[get("/user/get_holding_nfts/{wallet_address}")]
pub async fn get_user_nfts_hold(
    path:web::Path<String>,
    conn_pool:Data<PgPool>,
)  -> Result<impl Responder> {
    let wallet_address=path.into_inner();
    
    if let Ok(user_nfts_hold) =services::get_user_nfts_hold::take(&wallet_address, &conn_pool).await {
        Ok(HttpResponse::Ok().json(user_nfts_hold))
    }else {
        Ok(HttpResponse::Ok().json(DontHaveData{wallet_address:wallet_address.clone(),result:None}))
    }
}


#[get("/user/get_income_nfts/{wallet_address}")]
pub async fn get_user_income_nfts(
    path:web::Path<String>,
    conn_pool:Data<PgPool>,
)  -> Result<impl Responder> {
    let wallet_address=path.into_inner();
    if let Ok(user_income_nfts) =services::get_user_income_nfts_hold::take(&wallet_address, &conn_pool).await {
        Ok(HttpResponse::Ok().json(user_income_nfts))
    }else {
        Ok(HttpResponse::Ok().json(DontHaveData{wallet_address:wallet_address.clone(),result:None}))
    }
}



#[get("/user/get_holding_nfts_top/{wallet_address}")]
pub async fn get_user_nfts_hold_top(
    path:web::Path<String>,
    conn_pool:Data<PgPool>,
) -> Result<impl Responder> {
    let wallet_address=path.into_inner();
    if let Ok(data) =services::get_user_nfts_hold_top::take(&wallet_address, &conn_pool).await {
        Ok(HttpResponse::Ok().json(data))
    }else {
        Ok(HttpResponse::Ok().json(DontHaveData{wallet_address:wallet_address.clone(),result:None}))
    }
}


#[get("/user/get_nfts_trade_info/{wallet_address}")]
pub async fn get_user_nfts_trade_info(
    path:web::Path<String>,
    conn_pool:Data<PgPool>,
) -> Result<impl Responder> {
    let wallet_address=path.into_inner();
    if let Ok(data) =services::get_user_nfts_trade_info::take(&wallet_address, &conn_pool).await {
        Ok(HttpResponse::Ok().json(data))
    }else {
        Ok(HttpResponse::Ok().json(DontHaveData{wallet_address:wallet_address.clone(),result:None}))
    }
}



#[get("/user/get_holding_tokens/{wallet_address}")]
pub async fn get_user_tokens_hold(
    path:web::Path<String>,
    conn_pool:Data<PgPool>,
) -> Result<impl Responder> {
    let wallet_address=path.into_inner();
    if let Ok(data) =services::get_user_tokens_hold::take(&wallet_address, &conn_pool).await {
        Ok(HttpResponse::Ok().json(data))
    }else {
        Ok(HttpResponse::Ok().json(DontHaveData{wallet_address:wallet_address.clone(),result:None}))
    }
}


#[get("/user/get_holding_tokens_top/{wallet_address}")]
pub async fn get_user_tokens_hold_top(
    path:web::Path<String>,
    conn_pool:Data<PgPool>,
) -> Result<impl Responder> {
    let wallet_address=path.into_inner();
    if let Ok(data) =services::get_user_tokens_hold_top::take(&wallet_address, &conn_pool).await {
        Ok(HttpResponse::Ok().json(data))
    }else {
        Ok(HttpResponse::Ok().json(DontHaveData{wallet_address:wallet_address.clone(),result:None}))
    }
}




// tools
#[post("/wallet_transactions_async")]
pub async fn sync_wallet_transactions(
    body: web::Json<Wallet>,
    conn_pool:Data<PgPool>,
) -> impl Responder {

    let wallet_address = body.wallet_address.clone();
    tokio::task::spawn(async move{
        services::sync_user_trade_histroy_data::take(&wallet_address, &conn_pool).await;
    });
    HttpResponse::Ok().body("Processing started. Please check back later.")
}
