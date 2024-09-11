use anyhow::Result;
use nova_db::create_db_conn_pool;
use searcher::{run_wss, save_to_db};
use tokio::sync::mpsc::unbounded_channel;

#[tokio::main]
async fn main() -> Result<()> {
    
    let query="tm.event='Tx'";

    let (transaction_data_tx,transaction_data_rx)=unbounded_channel();
    let conn_pool=create_db_conn_pool().await?;

    // run wss and get tx_hash info tokio_thread
    let _run_wss_thead=tokio::spawn(async move{
        let _ = run_wss(query,transaction_data_tx).await;
    });

    let _run_searcher_db=tokio::spawn(async move{
        let _=save_to_db(transaction_data_rx, conn_pool).await;
    }).await;
    Ok(())
}
