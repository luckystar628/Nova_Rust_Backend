
use actix_cors::Cors;
use actix_web::{web::{self, Data}, App, HttpServer};
use anyhow::Result;
use dotenv::dotenv;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use sqlx::PgPool;

#[actix_web::main]
async fn main()->Result<()> {

    // load conn poll and set ssl
    dotenv().ok();
    let nova_db_url=std::env::var("DATABASE_URL")?;
    let nova_db_pool=PgPool::connect(&nova_db_url).await?;

    let mut ssl_builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    ssl_builder.set_private_key_file("/etc/letsencrypt/live/api.nova-solutions.io/privkey.pem", SslFiletype::PEM).unwrap();
    ssl_builder.set_certificate_chain_file("/etc/letsencrypt/live/api.nova-solutions.io/fullchain.pem").unwrap();


    HttpServer::new(move ||{
        App::new()
            .app_data(Data::new(nova_db_pool.to_owned()))
            .wrap(  
                Cors::default()
                .allowed_origin("http://localhost:3000")
                .allowed_origin("https://localhost:3000")
                .allowed_origin("https://novafrontend-dev.vercel.app")
                .allowed_origin("https://dev.nova-solutions.io")
                .allow_any_header()
                .allow_any_method()
                .max_age(3600) 
            )
            .service(web::scope("/user"))
            .service(web::scope("/tools"))
    })
    .bind_openssl(("0.0.0.0", 19999),ssl_builder)?
    .run()
    .await?;
    
    Ok(())


}
