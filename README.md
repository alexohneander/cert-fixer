# cert-fixer

## Why does this project exist?
If you run a Kubernetes cluster behind a NAT and want to create certificates with the **Cert-Manager**, you quickly run into the problem that the Self Propagation check fails. This issue can be fixed with a simple line of code in the **CoreDNS** config. But now you don't want to adjust the **CoreDNS** Config for every Ingress and restart the Pod afterwards. The **cert-fixer** is supposed to be a quick remedy for this.