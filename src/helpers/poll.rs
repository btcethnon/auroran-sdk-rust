//! Poll / flatten helpers shared by sync and async (`impl_trading_poll_helpers!`).

#[macro_export]
macro_rules! impl_trading_poll_helpers {
    (sync, $client:ty) => {
        $crate::impl_trading_poll_helpers!(@sync $client);
    };
    (async, $client:ty) => {
        $crate::impl_trading_poll_helpers!(@async $client);
    };
    (@sync $client:ty) => {
        pub fn symbol_leverage(
            client: &$client,
            address: &str,
            symbol: &str,
        ) -> Result<Option<u32>, $crate::ClientError> {
            Ok(client
                .try_position(address, symbol)?
                .map(|pos| pos.leverage))
        }

        pub fn wait_for_leverage(
            client: &$client,
            address: &str,
            symbol: &str,
            target: u32,
            poll: &$crate::PollConfig,
        ) -> Result<bool, $crate::ClientError> {
            $crate::helpers::sync_sleep(poll.interval);
            for attempt in 1..=poll.attempts {
                if symbol_leverage(client, address, symbol)? == Some(target) {
                    return Ok(true);
                }
                if attempt < poll.attempts {
                    $crate::helpers::sync_sleep(poll.interval);
                }
            }
            Ok(false)
        }

        pub fn wait_for_account<F>(
            client: &$client,
            address: &str,
            poll: &$crate::PollConfig,
            mut ready: F,
        ) -> Result<$crate::AccountSummaryResponse, $crate::ClientError>
        where
            F: FnMut(&$crate::AccountSummaryResponse) -> bool,
        {
            $crate::helpers::sync_sleep(poll.interval);
            for attempt in 1..=poll.attempts {
                let acct = client.account(address)?;
                if ready(&acct) {
                    return Ok(acct);
                }
                if attempt < poll.attempts {
                    $crate::helpers::sync_sleep(poll.interval);
                }
            }
            client.account(address)
        }

        pub fn wait_for_flat_orders(
            client: &$client,
            address: &str,
            poll: &$crate::PollConfig,
        ) -> Result<bool, $crate::ClientError> {
            $crate::helpers::sync_sleep(poll.interval);
            for attempt in 1..=poll.attempts {
                if client.account_orders(address)?.orders.is_empty() {
                    return Ok(true);
                }
                if attempt < poll.attempts {
                    $crate::helpers::sync_sleep(poll.interval);
                }
            }
            Ok(false)
        }

        pub fn wait_for_flat_positions(
            client: &$client,
            address: &str,
            markets: &[$crate::MarketListItem],
            poll: &$crate::PollConfig,
        ) -> Result<bool, $crate::ClientError> {
            $crate::helpers::sync_sleep(poll.interval);
            for attempt in 1..=poll.attempts {
                let acct = client.account(address)?;
                if $crate::open_position_symbols(&acct, markets).is_empty() {
                    return Ok(true);
                }
                if attempt < poll.attempts {
                    $crate::helpers::sync_sleep(poll.interval);
                }
            }
            Ok(false)
        }

        pub fn set_leverage_if_needed(
            client: &$client,
            config: &$crate::SigningConfig,
            sk: &alloy::signers::local::PrivateKeySigner,
            owner: $crate::Address20,
            address: &str,
            symbol: &str,
            target: u32,
            poll: &$crate::PollConfig,
        ) -> Result<(Option<$crate::TxReceiptResponse>, u64), $crate::ClientError> {
            let nonce = client.account(address)?.nonce;
            let current = symbol_leverage(client, address, symbol)?;
            if current == Some(target) {
                return Ok((None, nonce));
            }
            let action = $crate::builders::set_leverage(owner, symbol, target);
            let (receipt, next) =
                $crate::helpers::submit_accepted(client, config, sk, nonce, action)?;
            wait_for_leverage(client, address, symbol, target, poll)?;
            Ok((Some(receipt), next))
        }

        pub fn flatten_account(
            client: &$client,
            config: &$crate::SigningConfig,
            sk: &alloy::signers::local::PrivateKeySigner,
            owner: $crate::Address20,
            address: &str,
            markets: &[$crate::MarketListItem],
            poll: &$crate::PollConfig,
        ) -> Result<$crate::FlattenResult, $crate::ClientError> {
            let mut nonce = client.account(address)?.nonce;
            let orders = client.account_orders(address)?;
            let acct = client.account(address)?;
            let pos_symbols = $crate::open_position_symbols(&acct, markets);
            let order_syms = $crate::order_symbols(&orders);

            if order_syms.is_empty() && pos_symbols.is_empty() {
                return Ok($crate::FlattenResult {
                    skipped: true,
                    ..Default::default()
                });
            }

            let mut result = $crate::FlattenResult::default();

            if !order_syms.is_empty() {
                for sym in &order_syms {
                    let action = $crate::builders::mass_cancel_owner(owner, sym);
                    let (_, next) =
                        $crate::helpers::submit_accepted(client, config, sk, nonce, action)?;
                    nonce = next;
                    result.cancelled_symbols.push(sym.clone());
                }
                let _ = wait_for_flat_orders(client, address, poll)?;
            }

            nonce = client.account(address)?.nonce;
            let acct = client.account(address)?;
            let pos_symbols = $crate::open_position_symbols(&acct, markets);
            if !pos_symbols.is_empty() {
                for sym in &pos_symbols {
                    let action = $crate::builders::close_position_market(owner, sym);
                    let (_, next) =
                        $crate::helpers::submit_accepted(client, config, sk, nonce, action)?;
                    nonce = next;
                    result.closed_symbols.push(sym.clone());
                }
                let _ = wait_for_flat_positions(client, address, markets, poll)?;
            }

            Ok(result)
        }
    };
    (@async $client:ty) => {
        pub async fn symbol_leverage(
            client: &$client,
            address: &str,
            symbol: &str,
        ) -> Result<Option<u32>, $crate::ClientError> {
            Ok(client
                .try_position(address, symbol)
                .await?
                .map(|pos| pos.leverage))
        }

        pub async fn wait_for_leverage(
            client: &$client,
            address: &str,
            symbol: &str,
            target: u32,
            poll: &$crate::PollConfig,
        ) -> Result<bool, $crate::ClientError> {
            $crate::async_helpers::async_sleep(poll.interval).await;
            for attempt in 1..=poll.attempts {
                if symbol_leverage(client, address, symbol).await? == Some(target) {
                    return Ok(true);
                }
                if attempt < poll.attempts {
                    $crate::async_helpers::async_sleep(poll.interval).await;
                }
            }
            Ok(false)
        }

        pub async fn wait_for_account<F>(
            client: &$client,
            address: &str,
            poll: &$crate::PollConfig,
            mut ready: F,
        ) -> Result<$crate::AccountSummaryResponse, $crate::ClientError>
        where
            F: FnMut(&$crate::AccountSummaryResponse) -> bool,
        {
            $crate::async_helpers::async_sleep(poll.interval).await;
            for attempt in 1..=poll.attempts {
                let acct = client.account(address).await?;
                if ready(&acct) {
                    return Ok(acct);
                }
                if attempt < poll.attempts {
                    $crate::async_helpers::async_sleep(poll.interval).await;
                }
            }
            client.account(address).await
        }

        pub async fn wait_for_flat_orders(
            client: &$client,
            address: &str,
            poll: &$crate::PollConfig,
        ) -> Result<bool, $crate::ClientError> {
            $crate::async_helpers::async_sleep(poll.interval).await;
            for attempt in 1..=poll.attempts {
                if client.account_orders(address).await?.orders.is_empty() {
                    return Ok(true);
                }
                if attempt < poll.attempts {
                    $crate::async_helpers::async_sleep(poll.interval).await;
                }
            }
            Ok(false)
        }

        pub async fn wait_for_flat_positions(
            client: &$client,
            address: &str,
            markets: &[$crate::MarketListItem],
            poll: &$crate::PollConfig,
        ) -> Result<bool, $crate::ClientError> {
            $crate::async_helpers::async_sleep(poll.interval).await;
            for attempt in 1..=poll.attempts {
                let acct = client.account(address).await?;
                if $crate::open_position_symbols(&acct, markets).is_empty() {
                    return Ok(true);
                }
                if attempt < poll.attempts {
                    $crate::async_helpers::async_sleep(poll.interval).await;
                }
            }
            Ok(false)
        }

        pub async fn set_leverage_if_needed(
            client: &$client,
            config: &$crate::SigningConfig,
            sk: &alloy::signers::local::PrivateKeySigner,
            owner: $crate::Address20,
            address: &str,
            symbol: &str,
            target: u32,
            poll: &$crate::PollConfig,
        ) -> Result<(Option<$crate::TxReceiptResponse>, u64), $crate::ClientError> {
            let nonce = client.account(address).await?.nonce;
            let current = symbol_leverage(client, address, symbol).await?;
            if current == Some(target) {
                return Ok((None, nonce));
            }
            let action = $crate::builders::set_leverage(owner, symbol, target);
            let (receipt, next) =
                $crate::async_helpers::submit_accepted(client, config, sk, nonce, action).await?;
            wait_for_leverage(client, address, symbol, target, poll).await?;
            Ok((Some(receipt), next))
        }

        pub async fn flatten_account(
            client: &$client,
            config: &$crate::SigningConfig,
            sk: &alloy::signers::local::PrivateKeySigner,
            owner: $crate::Address20,
            address: &str,
            markets: &[$crate::MarketListItem],
            poll: &$crate::PollConfig,
        ) -> Result<$crate::FlattenResult, $crate::ClientError> {
            let mut nonce = client.account(address).await?.nonce;
            let orders = client.account_orders(address).await?;
            let acct = client.account(address).await?;
            let pos_symbols = $crate::open_position_symbols(&acct, markets);
            let order_syms = $crate::order_symbols(&orders);

            if order_syms.is_empty() && pos_symbols.is_empty() {
                return Ok($crate::FlattenResult {
                    skipped: true,
                    ..Default::default()
                });
            }

            let mut result = $crate::FlattenResult::default();

            if !order_syms.is_empty() {
                for sym in &order_syms {
                    let action = $crate::builders::mass_cancel_owner(owner, sym);
                    let (_, next) = $crate::async_helpers::submit_accepted(
                        client, config, sk, nonce, action,
                    )
                    .await?;
                    nonce = next;
                    result.cancelled_symbols.push(sym.clone());
                }
                let _ = wait_for_flat_orders(client, address, poll).await?;
            }

            nonce = client.account(address).await?.nonce;
            let acct = client.account(address).await?;
            let pos_symbols = $crate::open_position_symbols(&acct, markets);
            if !pos_symbols.is_empty() {
                for sym in &pos_symbols {
                    let action = $crate::builders::close_position_market(owner, sym);
                    let (_, next) = $crate::async_helpers::submit_accepted(
                        client, config, sk, nonce, action,
                    )
                    .await?;
                    nonce = next;
                    result.closed_symbols.push(sym.clone());
                }
                let _ = wait_for_flat_positions(client, address, markets, poll).await?;
            }

            Ok(result)
        }
    };
}
