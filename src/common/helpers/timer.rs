use crate::common::models::trade_scheme::TradeName;
use crate::common::strings::trim_newline;
use cancellable_timer::{Canceller, Timer};
use tokio::time::sleep;
use tokio::time::Duration;

pub async fn tokio_sleep(time_ms: u64) {
    sleep(Duration::from_millis(time_ms)).await
}

async fn read_from_stdin(
    interrupter_keyword: String,
    trade_name: TradeName,
    timer_canceller: Canceller,
) -> async_std::io::Result<()> {
    'read_loop: loop {
        log::info!(
            r#"type "{}" and press enter to '{}' immediately: "#,
            interrupter_keyword,
            trade_name
        );

        let stdin = async_std::io::stdin();
        let mut input_line = String::new();

        let r = stdin.read_line(&mut input_line);
        let n = r.await?;

        input_line = trim_newline(input_line);

        log::debug!("received the input line: {}", input_line);

        if input_line.eq(&interrupter_keyword) {
            log::debug!(
                "interrupter keyword received, cancelling the timer (if not cancelled already)..."
            );

            timer_canceller.cancel()?;

            break 'read_loop;
        } else {
            log::debug!(
                r#"the received interrupter keyword ({}) does NOT match, ({}). trying again..."#,
                input_line,
                interrupter_keyword
            );
        }
    }

    Ok(())
}

pub async fn interruptable_sleep(
    interrupter_keyword: String,
    trade_name: TradeName,
    sleep_for_ms: u64,
) -> anyhow::Result<()> {
    let (mut timer, timer_canceller) = Timer::new2()?;

    let c1 = async_std::task::spawn(read_from_stdin(
        interrupter_keyword,
        trade_name,
        timer_canceller,
    ));

    let t = timer.sleep(Duration::from_millis(sleep_for_ms));

    if t.is_err() {
        // do nothing there
    }

    async_std::task::block_on(c1.cancel());

    Ok(())
}
