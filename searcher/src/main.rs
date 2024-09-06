use anyhow::Result;
use searcher::run_wss;
use tokio::sync::mpsc::unbounded_channel;

#[tokio::main]
async fn main() -> Result<()> {
    
    let query="tm.event='Tx'";

    let (transaction_data_tx,mut transaction_data_rx)=unbounded_channel();
    // run wss and get tx_hash info tokio_thread
    let _run_wss_thead=tokio::spawn(async move{
        let _ = run_wss(query,transaction_data_tx).await;
    });

    for x in transaction_data_rx.recv().await{
        println!("{:#?}",x);
    }

    Ok(())
}