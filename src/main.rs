mod sites;
use dotenv::dotenv;

#[async_std::main]
async fn main() -> tide::Result<()> {
    dotenv().ok();

    let mut app = tide::new();
    app.at("/").get(sites::get_sites);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
