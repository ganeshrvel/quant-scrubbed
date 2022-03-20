use ethers::abi::ethereum_types::Address;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::Provider;
use ethers::prelude::{abigen, LocalWallet, Ws};
use std::sync::Arc;

// 'abigen' generates the type-safe contract bindings by providing the ABI
// definition in human readable format
abigen!(
    FactoryContract,
    r#"[
        event PairCreated(address indexed token0, address indexed token1, address pair, uint)
        function getPair(address tokenA, address tokenB) external view returns (address pair)
    ]"#,
    event_derives(serde::Deserialize, serde::Serialize)
);

abigen!(
    RouterContract,
    r#"[
        function getAmountsOut(uint amountIn, address[] memory path) public view returns (uint[] memory amounts)
        function swapExactTokensForTokens(uint amountIn, uint amountOutMin, address[] calldata path, address to, uint deadline) external returns (uint[] memory amounts)
        function swapExactTokensForTokensSupportingFeeOnTransferTokens(uint amountIn, uint amountOutMin, address[] calldata path, address to, uint deadline) external returns (uint[] memory amounts)
        function approve(address _spender, uint256 value) external returns(bool)
    ]"#,
    event_derives(serde::Deserialize, serde::Serialize)
);

abigen!(
    Erc20Contract,
    r#"[{"constant":true,"inputs":[],"name":"name","outputs":[{"name":"name","type":"string"}],"payable":false,"type":"function"},{"constant":true,"inputs":[],"name":"symbol","outputs":[{"name":"symbol","type":"string"}],"payable":false,"type":"function"},{"constant":true,"inputs":[],"name":"decimals","outputs":[{"name":"decimals","type":"uint8"}],"payable":false,"type":"function"},{"constant":false,"inputs":[{"name":"spender","type":"address"},{"name":"value","type":"uint256"}],"name":"approve","outputs":[{"name":"success","type":"bool"}],"payable":false,"type":"function"},{"constant":true,"inputs":[],"name":"totalSupply","outputs":[{"name":"totalSupply","type":"uint256"}],"payable":false,"type":"function"},{"constant":false,"inputs":[{"name":"from","type":"address"},{"name":"to","type":"address"},{"name":"value","type":"uint256"}],"name":"transferFrom","outputs":[{"name":"success","type":"bool"}],"payable":false,"type":"function"},{"constant":true,"inputs":[{"name":"who","type":"address"}],"name":"balanceOf","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"type":"function"},{"constant":false,"inputs":[{"name":"to","type":"address"},{"name":"value","type":"uint256"}],"name":"transfer","outputs":[{"name":"success","type":"bool"}],"payable":false,"type":"function"},{"constant":true,"inputs":[{"name":"owner","type":"address"},{"name":"spender","type":"address"}],"name":"allowance","outputs":[{"name":"remaining","type":"uint256"}],"payable":false,"type":"function"},{"anonymous":false,"inputs":[{"indexed":true,"name":"owner","type":"address"},{"indexed":true,"name":"spender","type":"address"},{"indexed":false,"name":"value","type":"uint256"}],"name":"Approval","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"name":"from","type":"address"},{"indexed":true,"name":"to","type":"address"},{"indexed":false,"name":"value","type":"uint256"}],"name":"Transfer","type":"event"}]"#,
    event_derives(serde::Deserialize, serde::Serialize)
);

pub type FactoryContract =
    factorycontract_mod::FactoryContract<SignerMiddleware<Provider<Ws>, LocalWallet>>;

pub type Erc20Contract =
    erc20contract_mod::Erc20Contract<SignerMiddleware<Provider<Ws>, LocalWallet>>;

pub type RouterContract =
    routercontract_mod::RouterContract<SignerMiddleware<Provider<Ws>, LocalWallet>>;

#[derive(Debug)]
pub struct QuantContracts {
    pub factory: FactoryContract,
    pub token_in_erc20: Erc20Contract,
    pub native_token_erc20: Erc20Contract,
    pub router: RouterContract,
}

#[derive(Debug)]
pub struct QuantContractsArgs<'a> {
    pub client: &'a Arc<SignerMiddleware<Provider<Ws>, LocalWallet>>,
    pub factory_addr_h160: Address,
    pub token_in_h160: Address,
    pub router_in_h160: Address,
    pub native_token_h160: Address,
}

impl QuantContracts {
    fn factory_contract(
        client: &Arc<SignerMiddleware<Provider<Ws>, LocalWallet>>,
        factory_addr: Address,
    ) -> FactoryContract {
        FactoryContract::new(factory_addr, client.clone())
    }

    fn ecr20_contract(
        client: &Arc<SignerMiddleware<Provider<Ws>, LocalWallet>>,
        erc20_token_h160: Address,
    ) -> Erc20Contract {
        Erc20Contract::new(erc20_token_h160, client.clone())
    }

    fn router_contract(
        client: &Arc<SignerMiddleware<Provider<Ws>, LocalWallet>>,
        router_in: Address,
    ) -> RouterContract {
        RouterContract::new(router_in, client.clone())
    }

    pub fn new(args: QuantContractsArgs) -> QuantContracts {
        log::debug!("initializing contracts...");

        let client = args.client;
        let factory_addr_h160 = args.factory_addr_h160;
        let token_in_h160 = args.token_in_h160;
        let native_token_h160 = args.native_token_h160;
        let router_in_h160 = args.router_in_h160;

        let factory = QuantContracts::factory_contract(client, factory_addr_h160);
        let token_in_erc20 = QuantContracts::ecr20_contract(client, token_in_h160);
        let native_token_erc20 = QuantContracts::ecr20_contract(client, native_token_h160);
        let router = QuantContracts::router_contract(client, router_in_h160);

        QuantContracts {
            factory,
            token_in_erc20,
            native_token_erc20,
            router,
        }
    }
}
