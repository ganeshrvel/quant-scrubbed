use ethers::middleware::SignerMiddleware;
use ethers::prelude::{Ws, Signer};
use ethers::providers::Provider;
use ethers::signers::LocalWallet;
use std::sync::Arc;

#[derive(Debug)]
pub struct QuantMiddleware {
    pub wallet: LocalWallet,
    pub client: Arc<SignerMiddleware<Provider<Ws>, LocalWallet>>,
}

impl QuantMiddleware {
    fn wallet(chain_id: u64, mnemonic: &str) -> anyhow::Result<LocalWallet> {
        Ok(mnemonic.parse::<LocalWallet>()?.with_chain_id(chain_id))
    }

    fn client(
        provider: Provider<Ws>,
        wallet: &LocalWallet,
    ) -> anyhow::Result<Arc<SignerMiddleware<Provider<Ws>, LocalWallet>>> {
        let client: SignerMiddleware<Provider<Ws>, LocalWallet> =
            SignerMiddleware::new(provider, wallet.clone());
        let client: Arc<SignerMiddleware<Provider<Ws>, LocalWallet>> = Arc::new(client);

        Ok(client)
    }

    pub fn new(
        provider: Provider<Ws>,
        chain_id: u64,
        mnemonic: &str,
    ) -> anyhow::Result<QuantMiddleware> {
        log::debug!("initializing contracts...");

        let wallet = QuantMiddleware::wallet(chain_id, mnemonic)?;
        let client = QuantMiddleware::client(provider, &wallet)?;

        Ok(QuantMiddleware { wallet, client })
    }
}
