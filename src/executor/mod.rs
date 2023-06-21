/* module responsible for throwing manifests against a ku8s api and authenticating
with the given method in the kubeconfig */

use crate::error_exit;
use kube::{
    api::{DynamicObject, GroupVersionKind},
    config::{KubeConfigOptions, Kubeconfig},
    discovery::ApiResource,
    Api, Client, Config,
};
use serde_json::Value;

pub async fn get_client(kconf_path: Option<&str>) -> Client {
    /*
    This function returns a kube client, which is used to authenticate with the
    ku8s api. If Kubeconfig cannot be infered from environment, use provided path
    */

    if kconf_path.is_some() {
        info!("using kubeconfig from path {}", kconf_path.unwrap());
        let kconfig = Kubeconfig::read_from(kconf_path.unwrap())
            .map_err(|e| {
                error_exit!(
                    "could not read kubeconfig from path {} with error: {}",
                    kconf_path.unwrap(),
                    e
                );
            })
            .unwrap();

        let config = Config::from_custom_kubeconfig(kconfig, &KubeConfigOptions::default())
            .await
            .expect("could not create config from kubeconfig");
        Client::try_from(config).unwrap()
    } else {
        info!("using default kubeconfig from environment or default config location.");
        return match Client::try_default().await {
            Err(e) => error_exit!("could not infer kubeconfig from environment: {}", e),
            Ok(cl) => cl,
        };
    }
}

pub async fn deploy_resource(
    resource_raw: &Value,
    client: Client,
    namespace: &str,
) -> Result<DynamicObject, kube::Error> {
    let gv = resource_raw
        .get("apiVersion")
        .expect("missing apiVersion in resource!")
        .as_str()
        .unwrap();

    let kind = resource_raw
        .get("kind")
        .expect("missing kind in resource!")
        .as_str()
        .unwrap();

    let (group, version) = match gv.split_once("/") {
        Some((g, v)) => (g, v),
        None => ("", gv),
    };

    let gvk = GroupVersionKind::gvk(group, version, &kind);

    let apiresource = ApiResource::from_gvk(&gvk);

    let dynobj = DynamicObject::new(
        resource_raw["metadata"]["name"].to_string().as_str(),
        &apiresource,
    )
    .data(resource_raw.clone());

    let api = Api::<DynamicObject>::namespaced_with(client, namespace, &apiresource);

    let postparams = kube::api::PostParams {
        dry_run: true,
        field_manager: None,
    };

    api.create(&postparams, &dynobj).await
}
