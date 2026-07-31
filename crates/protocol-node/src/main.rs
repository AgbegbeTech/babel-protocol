#[tokio::main]
async fn main() -> anyhow::Result<()> {
    protocol_node::run().await
}
