use k8s_openapi::api::networking::v1::Ingress;
use kube::{Client, Api};

type Result<T, E = anyhow::Error> = std::result::Result<T, E>;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::try_default().await?;
    let api: Api<Ingress> = Api::all(client);

    let ingresses = api.list(&Default::default()).await?;
    for ingress in ingresses.items {
        println!("Ingress {} in namespace {}", ingress.metadata.name.unwrap(), ingress.metadata.namespace.unwrap());
    }

    Ok(())
}
