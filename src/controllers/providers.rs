use crate::common::constants::default_values::DefaultValues;
use crate::common::models::config::ProviderEntity;
use crate::common::utils::network_request::{wss_request, RestfulAuthorization};
use ethers::providers::{Provider, Ws};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct QuantProvider {
    pub ws: Provider<Ws>,
}

impl QuantProvider {
    async fn ws(
        provider_api: String,
        authorization: &Option<RestfulAuthorization>,
    ) -> anyhow::Result<Provider<Ws>> {
        let request = wss_request(&*provider_api, authorization)?;
        let (ws, _) = tokio_tungstenite::connect_async(request).await?;
        let p = Provider::new(Ws::new(ws))
            .interval(Duration::from_millis(DefaultValues::PROVIDER_TIMEOUT));

        Ok(p)
    }

    pub async fn new(p: &ProviderEntity) -> anyhow::Result<QuantProvider> {
        let provider_api = &p.api;

        log::info!("initializing providers for the api: '{}'", provider_api);

        let provider_authorization = &p.get_authorization();

        let ws: Provider<Ws> =
            QuantProvider::ws(provider_api.to_owned(), provider_authorization).await?;
        let providers_struct = QuantProvider { ws };

        Ok(providers_struct)
    }
}
