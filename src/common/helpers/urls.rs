use crate::common::constants::urls::Urls;
use crate::common::models::config::NetworkName;
use ethers::abi::ethereum_types::H256;

pub fn get_tx_hash_url(tx_hash: H256, network_name: NetworkName) -> String {
    let end_point = "tx/";

    match network_name {
        NetworkName::Mainnet => format!("{}{}{:?}", Urls::BSC_MAINNET, end_point, tx_hash),
        NetworkName::Testnet => format!("{}{}{:?}", Urls::BSC_TESTNET, end_point, tx_hash),
    }
}
