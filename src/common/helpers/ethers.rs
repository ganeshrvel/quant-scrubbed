use ethers::middleware::SignerMiddleware;
use ethers::prelude::{LocalWallet, Middleware, Provider, Ws, U256};
use ethers::types::Address;
use std::sync::Arc;

pub async fn get_account_balance(
    client: &Arc<SignerMiddleware<Provider<Ws>, LocalWallet>>,
    account_address_h160: &Address,
) -> anyhow::Result<U256> {
    let balance = client.get_balance(*account_address_h160, None).await?;

    Ok(balance)
}

pub async fn get_network_gas_price(
    client: &Arc<SignerMiddleware<Provider<Ws>, LocalWallet>>,
) -> anyhow::Result<U256> {
    let network_gas_price = client.get_gas_price().await?;

    Ok(network_gas_price)
}
