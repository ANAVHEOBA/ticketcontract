use mongodb::{Client, options::ClientOptions};

pub async fn connect_mongo(database_url: &str) -> anyhow::Result<Client> {
    let options = ClientOptions::parse(database_url).await?;
    Ok(Client::with_options(options)?)
}
