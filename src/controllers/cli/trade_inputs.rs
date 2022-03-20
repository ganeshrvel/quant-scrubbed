use dialoguer::Input;
use ethers::abi::Address;
use std::str::FromStr;

#[derive(Debug)]
pub struct TradeInputs {
    pub token_in_contract: String,
    pub token_out_contract: String,
}

impl TradeInputs {
    fn get_cli_input(prompt_text: String) -> anyhow::Result<String> {
        let token_in: String = Input::new()
            .with_prompt(prompt_text)
            .validate_with(|input: &String| -> anyhow::Result<()> {
                let a = Address::from_str(input);

                match a {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        let fmt_e = format!("invalid token address, try again. '{:?}'", e);
                        log::error!("invalid token address, try again. '{:?}'", e);

                        Err(anyhow::Error::msg(fmt_e))
                    }
                }
            })
            .interact()?;

        Ok(token_in)
    }

    pub fn token_in_contract(token_in_symbol: String) -> anyhow::Result<String> {
        let text = format!("Enter token address for {}", token_in_symbol);
        Self::get_cli_input(text)
    }

    pub fn token_out_contract(token_out_symbol: String) -> anyhow::Result<String> {
        let text = format!("Enter token address for {}", token_out_symbol);

        Self::get_cli_input(text)
    }
}
