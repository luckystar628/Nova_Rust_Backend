mod responses;
mod tools;
mod erros;
pub mod services;

#[cfg(test)]
mod tests{
    use anyhow::Result;
    #[tokio::test]
    async fn test_get_user_nfts_hold() -> Result<()> {
        println!("11");
        Ok(())
    }
}
