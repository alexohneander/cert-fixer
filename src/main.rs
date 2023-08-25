use k8s_openapi::api::{networking::v1::Ingress, core::v1::{ConfigMap, Pod}};
use kube::{Client, Api, api::ListParams};

type Result<T, E = anyhow::Error> = std::result::Result<T, E>;

const INGRESSCONTROLLER: &str = "ingress-contour-envoy.projectcontour.svc.cluster.local";

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::try_default().await?;
    let config_api: Api<ConfigMap> = Api::namespaced(client.to_owned(), "kube-system");
  
    // Save ingress hostnames in a vector
    let hostnames = get_hostnames_from_ingresses(client.to_owned()).await?;
    println!("Hostnames: {:?}", hostnames);

    // Get config map in kube-system namespace with name coredns
    let config_map = config_api.get("coredns").await?;

    // Patch config map and add hostnames to coredns config map
    let mut data = config_map.data.unwrap();
    let corefile = data.get_mut("Corefile").unwrap();
    print!("Corefile: {}", &corefile);

    // add hostnames to corefile after ".:53 {" line
    let mut corefile_lines: Vec<String> = corefile.split("\n").map(|s| s.to_string()).collect();
    let mut index = 0;
    for line in corefile_lines.iter() {
        if line.contains(".:53 {") {
            break;
        } else {
            index += 1;
        }
    }
    for hostname in hostnames {
        let rewrite_rule = format!("    rewrite name {} {}", hostname, INGRESSCONTROLLER);
        corefile_lines.insert(index + 1, rewrite_rule);

        index += 1;
    }

    // convert corefile_lines back to string
    let mut corefile_new = String::new();
    for line in corefile_lines {
        corefile_new.push_str(line.to_owned().as_str());
        corefile_new.push_str("\n");
    }

    println!("Corefile new: {}", &corefile_new);
    

    // Restart CoreDNS after patching config map
    restart_coredns(client.to_owned()).await?;

    Ok(())
}

async fn get_hostnames_from_ingresses(client: Client) -> Result<Vec<String>> {
    let ingress_api: Api<Ingress> = Api::all(client.to_owned());
    let ingresses = ingress_api.list(&Default::default()).await?;

    let mut hostnames: Vec<String> = Vec::new();

    for ingress in ingresses.items {
        let hostnames_rules = ingress.spec.unwrap().rules.unwrap();
        for hostname in hostnames_rules {
            let host = hostname.host.unwrap();
            hostnames.push(host);
        }
    }

    Ok(hostnames)
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

