use k8s_openapi::api::{networking::v1::Ingress, core::v1::{ConfigMap, Pod}};
use kube::{Client, Api, api::ListParams};

type Result<T, E = anyhow::Error> = std::result::Result<T, E>;

const INGRESSCONTROLLER: &str = "ingress-contour-envoy.projectcontour.svc.cluster.local";

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::try_default().await?;
    let ingress_api: Api<Ingress> = Api::all(client.to_owned());
    let config_api: Api<ConfigMap> = Api::namespaced(client.to_owned(), "kube-system");
  
    // Get all ingresses
    let ingresses = ingress_api.list(&Default::default()).await?;

    // Save ingress hostnames in a vector
    let mut hostnames: Vec<String> = Vec::new();
    for ingress in ingresses.items {
        let hostnames_rules = ingress.spec.unwrap().rules.unwrap();
        for hostname in hostnames_rules {
            let host = hostname.host.unwrap();
            hostnames.push(host);
        }
    }
    println!("Hostnames: {:?}", hostnames);

    // Get config map in kube-system namespace with name coredns
    let config_map = config_api.get("coredns").await?;
    println!("Config map {} in namespace {}", config_map.metadata.name.unwrap(), config_map.metadata.namespace.unwrap());

    // Restart CoreDNS after patching config map
    restart_coredns(client.to_owned()).await?;

    Ok(())
}

async fn restart_coredns(client: Client) -> Result<()> {
    let pod_api: Api<Pod> = Api::namespaced(client.to_owned(), "kube-system");
    let pods = pod_api.list(&ListParams::default().labels("k8s-app=kube-dns")).await?;
    for pod in pods.items {
        println!("Restarting CoreDNS");
        pod_api.delete(&pod.metadata.name.unwrap(), &Default::default()).await?;
    }
    Ok(())
}