/* module responsible for throwing manifests against a ku8s api and authenticating
with the given method in the kubeconfig */

use crate::error_exit;
use kube::{config::Kubeconfig, Client, Config};
use std::env;
use std::path::PathBuf;
use tower::ServiceBuilder;

pub async fn get_client(path: &str) -> Result<Client, kube::Error> {
    let kubeconfig = if path.is_empty() {
        // Use the kubeconfig file at the default location
        Config::infer().await?
    } else {
        // Use the kubeconfig file at the provided path
        let path = PathBuf::from(path);
        Config::from_custom_kubeconfig(
            kube::config::Kubeconfig::read_from(&path)?,
            &Default::default(),
        )
        .await?
    };
    let client = Client::try_from(kubeconfig)?;
    Ok(client)
}
