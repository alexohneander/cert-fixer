use k8s_openapi::api::{networking::v1::Ingress, core::v1::{ConfigMap, Pod}};
use kube::{Client, Api, api::ListParams};

type Result<T, E = anyhow::Error> = std::result::Result<T, E>;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::try_default().await?;
    let ingress_api: Api<Ingress> = Api::all(client.to_owned());
    let config_api: Api<ConfigMap> = Api::namespaced(client.to_owned(), "kube-system");
    let pod_api: Api<Pod> = Api::namespaced(client.to_owned(), "kube-system");
  
    // Get all ingresses
    let ingresses = ingress_api.list(&Default::default()).await?;
    for ingress in ingresses.items {
        println!("Ingress {} in namespace {}", ingress.metadata.name.unwrap(), ingress.metadata.namespace.unwrap());
    }

    // Get config map in kube-system namespace with name coredns
    let config_map = config_api.get("coredns").await?;
    println!("Config map {} in namespace {}", config_map.metadata.name.unwrap(), config_map.metadata.namespace.unwrap());

    // Get pod in kube-system namespace with label k8s-app: kube-dns
    let pods = pod_api.list(&ListParams::default().labels("k8s-app=kube-dns")).await?;
    for pod in &pods.items {
        println!("Pod {} in namespace {}", pod.metadata.name.to_owned().unwrap(), pod.metadata.namespace.to_owned().unwrap());
    }

    // delete pods
    for pod in pods.items {
        pod_api.delete(&pod.metadata.name.unwrap(), &Default::default()).await?;
    }

    Ok(())
}
