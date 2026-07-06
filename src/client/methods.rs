//! Shared JSON-RPC read/write method bodies for sync and async clients.

#[macro_export]
macro_rules! impl_auroran_client_methods {
    (blocking, $client:ident) => {
        impl $client {
    /// Sign with master key and submit (`domain_chain_id` defaults to `chain_id`).
    pub fn submit_signed(
        &self,
        chain_id: u64,
        network_tag: &str,
        sk: &PrivateKeySigner,
        nonce: u64,
        action: Action,
    ) -> Result<TxReceiptResponse, ClientError> {
        self.submit_signed_with_domain(chain_id, chain_id, network_tag, sk, nonce, action)
    }

    /// Like [`Self::submit_signed`], but returns [`ClientError::TxRejected`] on `kept-reject`.
    pub fn submit_signed_accepted(
        &self,
        chain_id: u64,
        network_tag: &str,
        sk: &PrivateKeySigner,
        nonce: u64,
        action: Action,
    ) -> Result<TxReceiptResponse, ClientError> {
        self.submit_signed(chain_id, network_tag, sk, nonce, action)
            .and_then(|r| r.ensure_accepted())
    }

    /// Sign with master key and submit, with an explicit EIP-712 `domain_chain_id`.
    ///
    /// For User-Signed actions (`WithdrawRequest` / `RegisterAgent` / `RevokeAgent`),
    /// set `domain_chain_id` to the user's connected EVM network ID (e.g. BSC = 56).
    pub fn submit_signed_with_domain(
        &self,
        chain_id: u64,
        domain_chain_id: u64,
        network_tag: &str,
        sk: &PrivateKeySigner,
        nonce: u64,
        action: Action,
    ) -> Result<TxReceiptResponse, ClientError> {
        let env = master_signed_envelope(chain_id, domain_chain_id, network_tag, sk, nonce, action);
        self.submit_action(&env)
    }

    /// Like [`Self::submit_signed_with_domain`], but returns [`ClientError::TxRejected`] on `kept-reject`.
    pub fn submit_signed_with_domain_accepted(
        &self,
        chain_id: u64,
        domain_chain_id: u64,
        network_tag: &str,
        sk: &PrivateKeySigner,
        nonce: u64,
        action: Action,
    ) -> Result<TxReceiptResponse, ClientError> {
        self.submit_signed_with_domain(
            chain_id,
            domain_chain_id,
            network_tag,
            sk,
            nonce,
            action,
        )
        .and_then(|r| r.ensure_accepted())
    }

    /// Sign with agent key and submit (`domain_chain_id` defaults to `chain_id`).
    pub fn submit_agent_signed(
        &self,
        chain_id: u64,
        network_tag: &str,
        signer: Address20,
        agent_sk: &PrivateKeySigner,
        nonce: u64,
        action: Action,
    ) -> Result<TxReceiptResponse, ClientError> {
        self.submit_agent_signed_with_domain(
            chain_id,
            chain_id,
            network_tag,
            signer,
            agent_sk,
            nonce,
            action,
        )
    }

    /// Sign with agent key and submit, with an explicit EIP-712 `domain_chain_id`.
    pub fn submit_agent_signed_with_domain(
        &self,
        chain_id: u64,
        domain_chain_id: u64,
        network_tag: &str,
        signer: Address20,
        agent_sk: &PrivateKeySigner,
        nonce: u64,
        action: Action,
    ) -> Result<TxReceiptResponse, ClientError> {
        let env = agent_signed_envelope(
            chain_id,
            domain_chain_id,
            network_tag,
            signer,
            agent_sk,
            nonce,
            action,
        );
        self.submit_action(&env)
    }

    // ── Chain / health ─────────────────────────────────────────────────────

    pub fn health_rest(&self) -> Result<HealthResponse, ClientError> {
        self.get_json("/api/v1/health")
    }

    pub fn health(&self) -> Result<HealthResponse, ClientError> {
        let qr: QueryResult<Value> = self.rpc_query("getHealth", Value::Null)?;
        Ok(HealthResponse {
            status: qr
                .data
                .get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("ok")
                .into(),
            height: qr.height,
        })
    }

    // ── Blocks ─────────────────────────────────────────────────────────────

    pub fn block_latest(&self) -> Result<BlockResponse, ClientError> {
        self.rpc_query("getBlock", Value::Null)
            .map(|qr: QueryResult<BlockResponse>| qr.data)
    }

    pub fn block_by_height(&self, height: u64) -> Result<BlockResponse, ClientError> {
        self.rpc_query("getBlock", serde_json::json!({ "height": height }))
            .map(|qr: QueryResult<BlockResponse>| qr.data)
    }

    pub fn block_events(
        &self,
        height: u64,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<BlockEventsResponse, ClientError> {
        let mut params = serde_json::json!({ "height": height });
        if let Some(o) = offset {
            params["offset"] = serde_json::Value::from(o);
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        // 节点读骨架：`result.data` 是事件数组本身，分页在 `result.page`。
        // （旧实现误把 `data` 当作 `{height,offset,total,events}` 结构 → 解码失败。）
        let qr: QueryResult<Vec<serde_json::Value>> = self.rpc_query("getBlockEvents", params)?;
        let page = page_from_query(&qr, offset, limit, qr.data.len());
        Ok(BlockEventsResponse {
            height: qr.height,
            offset: page.offset,
            total: page.total_or(qr.data.len()),
            events: qr.data,
        })
    }

    pub fn get_tx(&self, hash: &str) -> Result<TxReceiptResponse, ClientError> {
        self.rpc_query("getTx", serde_json::json!({ "hash": hash }))
            .map(|qr: QueryResult<TxReceiptResponse>| qr.data)
    }

    // ── Markets / orderbook ────────────────────────────────────────────────

    pub fn orderbook(&self, symbol: &str) -> Result<OrderbookResponse, ClientError> {
        self.orderbook_with_depth(symbol, None)
    }

    pub fn orderbook_with_depth(
        &self,
        symbol: &str,
        depth: Option<usize>,
    ) -> Result<OrderbookResponse, ClientError> {
        let mut params = serde_json::json!({ "symbol": symbol });
        if let Some(d) = depth {
            params["depth"] = serde_json::Value::from(d);
        }
        self.rpc_query("getOrderbook", params)
            .map(|qr: QueryResult<OrderbookResponse>| qr.data)
    }

    pub fn list_markets(&self) -> Result<Vec<MarketListItem>, ClientError> {
        self.rpc_query("getMarkets", Value::Null)
            .map(|qr: QueryResult<Vec<MarketListItem>>| qr.data)
    }

    pub fn market(&self, symbol: &str) -> Result<MarketDetailResponse, ClientError> {
        self.rpc_query("getMarket", serde_json::json!({ "symbol": symbol }))
            .map(|qr: QueryResult<MarketDetailResponse>| qr.data)
    }

    // ── Account ────────────────────────────────────────────────────────────

    pub fn account(&self, address: &str) -> Result<AccountSummaryResponse, ClientError> {
        self.rpc_query("getAccount", serde_json::json!({ "address": address }))
            .map(|qr: QueryResult<AccountSummaryResponse>| qr.data)
    }

    pub fn account_orders(&self, address: &str) -> Result<AccountOrdersResponse, ClientError> {
        self.rpc_query(
            "getAccountOrders",
            serde_json::json!({ "address": address }),
        )
        .map(|qr: QueryResult<AccountOrdersResponse>| qr.data)
    }

    // ── Bridge ─────────────────────────────────────────────────────────────

    pub fn bridge_settlement(&self) -> Result<BridgeSettlementResponse, ClientError> {
        self.rpc_query("getBridgeSettlement", Value::Null)
            .map(|qr: QueryResult<BridgeSettlementResponse>| qr.data)
    }

    pub fn bridge_deposit(&self, seq: u64) -> Result<BridgeDepositResponse, ClientError> {
        self.rpc_query("getBridgeDeposit", serde_json::json!({ "seq": seq }))
            .map(|qr: QueryResult<BridgeDepositResponse>| qr.data)
    }

    /// 按外部链充值引用 `(chain, seq)` 查询充值单。未找到时返回 `Ok(None)`。
    pub fn bridge_deposit_by_external_ref(
        &self,
        chain: &str,
        seq: u64,
    ) -> Result<Option<BridgeDepositResponse>, ClientError> {
        self.rpc_query(
            "getBridgeDepositByExternalRef",
            serde_json::json!({ "chain": chain.to_lowercase(), "seq": seq }),
        )
        .map(|qr: QueryResult<Option<BridgeDepositResponse>>| qr.data)
    }

    pub fn bridge_deposits(
        &self,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<BridgeDepositsListResponse, ClientError> {
        let mut params = serde_json::json!({});
        if let Some(o) = offset {
            params["offset"] = serde_json::Value::from(o);
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        let qr: QueryResult<Vec<DepositRecord>> = self.rpc_query("listBridgeDeposits", params)?;
        let page = page_from_query(&qr, offset, limit, qr.data.len());
        Ok(BridgeDepositsListResponse {
            offset: page.offset,
            total: page.total_or(qr.data.len()),
            deposits: qr.data,
        })
    }

    pub fn bridge_withdrawal(&self, id: u64) -> Result<BridgeWithdrawalResponse, ClientError> {
        self.rpc_query("getBridgeWithdrawal", serde_json::json!({ "id": id }))
            .map(|qr: QueryResult<BridgeWithdrawalResponse>| qr.data)
    }

    pub fn bridge_withdrawals(
        &self,
        status: Option<&str>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<BridgeWithdrawalsListResponse, ClientError> {
        let mut params = serde_json::json!({});
        if let Some(s) = status {
            params["status"] = serde_json::Value::from(s);
        }
        if let Some(o) = offset {
            params["offset"] = serde_json::Value::from(o);
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        let qr: QueryResult<Vec<WithdrawRecord>> =
            self.rpc_query("listBridgeWithdrawals", params)?;
        let page = page_from_query(&qr, offset, limit, qr.data.len());
        Ok(BridgeWithdrawalsListResponse {
            offset: page.offset,
            total: page.total_or(qr.data.len()),
            withdrawals: qr.data,
        })
    }

    // ── Meta ───────────────────────────────────────────────────────────────

    pub fn actions_meta(&self) -> Result<ActionsMetaResponse, ClientError> {
        self.rpc_query("getActionsMeta", Value::Null)
            .map(|qr: QueryResult<ActionsMetaResponse>| qr.data)
    }

    pub fn bootstrap(
        &self,
        address: Option<&str>,
        symbols: Option<&[String]>,
    ) -> Result<Value, ClientError> {
        self.bootstrap_with_options(address, symbols, None)
    }

    pub fn bootstrap_typed(
        &self,
        address: Option<&str>,
        symbols: Option<&[String]>,
    ) -> Result<BootstrapResponse, ClientError> {
        self.bootstrap_typed_with_options(address, symbols, None)
    }

    pub fn bootstrap_with_options(
        &self,
        address: Option<&str>,
        symbols: Option<&[String]>,
        book_depth: Option<usize>,
    ) -> Result<Value, ClientError> {
        self.rpc_query(
            "getBootstrap",
            bootstrap_params(address, symbols, book_depth),
        )
        .map(|qr: QueryResult<Value>| qr.data)
    }

    pub fn bootstrap_typed_with_options(
        &self,
        address: Option<&str>,
        symbols: Option<&[String]>,
        book_depth: Option<usize>,
    ) -> Result<BootstrapResponse, ClientError> {
        self.rpc_query(
            "getBootstrap",
            bootstrap_params(address, symbols, book_depth),
        )
        .map(|qr: QueryResult<BootstrapResponse>| qr.data)
    }

    // ── History / layer-2 queries ──────────────────────────────────────────

    pub fn candles(
        &self,
        symbol: &str,
        interval_ms: u64,
        start_time_ms: Option<u64>,
        end_time_ms: Option<u64>,
        limit: Option<usize>,
    ) -> Result<Vec<CandleResponse>, ClientError> {
        let mut params = serde_json::json!({ "symbol": symbol, "interval_ms": interval_ms });
        if let Some(t) = start_time_ms {
            params["start_time_ms"] = serde_json::Value::from(t);
        }
        if let Some(t) = end_time_ms {
            params["end_time_ms"] = serde_json::Value::from(t);
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        self.rpc_query("getCandles", params)
            .map(|qr: QueryResult<Vec<CandleResponse>>| qr.data)
    }

    pub fn recent_trades(
        &self,
        symbol: &str,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<Vec<TradeResponse>, ClientError> {
        let mut params = serde_json::json!({ "symbol": symbol });
        if let Some(o) = offset {
            params["offset"] = serde_json::Value::from(o);
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        self.rpc_query("getRecentTrades", params)
            .map(|qr: QueryResult<Vec<TradeResponse>>| qr.data)
    }

    pub fn user_fills(
        &self,
        address: &str,
        symbol: Option<&str>,
        start_time_ms: Option<u64>,
        end_time_ms: Option<u64>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<Vec<UserFillResponse>, ClientError> {
        let mut params = serde_json::json!({ "address": address });
        if let Some(s) = symbol {
            params["symbol"] = serde_json::Value::from(s);
        }
        if let Some(t) = start_time_ms {
            params["start_time_ms"] = serde_json::Value::from(t);
        }
        if let Some(t) = end_time_ms {
            params["end_time_ms"] = serde_json::Value::from(t);
        }
        if let Some(o) = offset {
            params["offset"] = serde_json::Value::from(o);
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        self.rpc_query("getUserFills", params)
            .map(|qr: QueryResult<Vec<UserFillResponse>>| qr.data)
    }

    pub fn order_status(&self, order_id: u64) -> Result<OrderStatusResponse, ClientError> {
        self.rpc_query(
            "getOrderStatus",
            serde_json::json!({ "order_id": order_id }),
        )
        .map(|qr: QueryResult<OrderStatusResponse>| qr.data)
    }

    pub fn order_status_by_cloid(
        &self,
        address: &str,
        symbol: &str,
        client_order_id: &str,
    ) -> Result<OrderStatusResponse, ClientError> {
        self.rpc_query(
            "getOrderStatus",
            serde_json::json!({
                "address": address,
                "symbol": symbol,
                "client_order_id": client_order_id,
            }),
        )
        .map(|qr: QueryResult<OrderStatusResponse>| qr.data)
    }

    pub fn exchange_config(&self) -> Result<ExchangeConfigResponse, ClientError> {
        self.rpc_query("getExchangeConfig", Value::Null)
            .map(|qr: QueryResult<ExchangeConfigResponse>| qr.data)
    }

    pub fn liquidatable_positions(
        &self,
        symbol: Option<&str>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<Vec<LiquidatablePosition>, ClientError> {
        let mut params = serde_json::json!({});
        if let Some(s) = symbol {
            params["symbol"] = serde_json::Value::from(s);
        }
        if let Some(o) = offset {
            params["offset"] = serde_json::Value::from(o);
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        self.rpc_query("getLiquidatablePositions", params)
            .map(|qr: QueryResult<Vec<LiquidatablePosition>>| qr.data)
    }

    pub fn estimated_liquidation_price(
        &self,
        symbol: &str,
        size: &str,
        entry_price: Option<&str>,
        leverage: u32,
    ) -> Result<EstimatedLiquidationResponse, ClientError> {
        let mut params =
            serde_json::json!({ "symbol": symbol, "size": size, "leverage": leverage });
        if let Some(ep) = entry_price {
            params["entry_price"] = serde_json::Value::from(ep);
        }
        self.rpc_query("getEstimatedLiquidationPrice", params)
            .map(|qr: QueryResult<EstimatedLiquidationResponse>| qr.data)
    }

    pub fn user_rate_limit(&self, address: &str) -> Result<UserRateLimitResponse, ClientError> {
        self.rpc_query(
            "getUserRateLimit",
            serde_json::json!({ "address": address }),
        )
        .map(|qr: QueryResult<UserRateLimitResponse>| qr.data)
    }

    pub fn admin_audit_log(
        &self,
        signer: Option<&str>,
        start_time_ms: Option<u64>,
        end_time_ms: Option<u64>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<Vec<AdminAuditEntry>, ClientError> {
        let mut params = serde_json::json!({});
        if let Some(s) = signer {
            params["signer"] = serde_json::Value::from(s);
        }
        if let Some(t) = start_time_ms {
            params["start_time_ms"] = serde_json::Value::from(t);
        }
        if let Some(t) = end_time_ms {
            params["end_time_ms"] = serde_json::Value::from(t);
        }
        if let Some(o) = offset {
            params["offset"] = serde_json::Value::from(o);
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        self.rpc_query("getAdminAuditLog", params)
            .map(|qr: QueryResult<Vec<AdminAuditEntry>>| qr.data)
    }

    pub fn user_fees(&self, address: &str) -> Result<UserFeesResponse, ClientError> {
        self.rpc_query("getUserFees", serde_json::json!({ "address": address }))
            .map(|qr: QueryResult<UserFeesResponse>| qr.data)
    }

    pub fn referral(&self, address: &str) -> Result<ReferralResponse, ClientError> {
        self.rpc_query("getReferral", serde_json::json!({ "address": address }))
            .map(|qr: QueryResult<ReferralResponse>| qr.data)
    }

    // ── Extended index/query surface ───────────────────────────────────────

    pub fn list_accounts(
        &self,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<ListAccountsResponse, ClientError> {
        self.list_accounts_filtered(ListAccountsFilter {
            offset,
            limit,
            ..Default::default()
        })
    }

    /// Unified `listAccounts` with optional role / referral filters.
    pub fn list_accounts_filtered(
        &self,
        filter: ListAccountsFilter,
    ) -> Result<ListAccountsResponse, ClientError> {
        let mut params = serde_json::json!({});
        if let Some(o) = filter.offset {
            params["offset"] = serde_json::Value::from(o);
        }
        if let Some(l) = filter.limit {
            params["limit"] = serde_json::Value::from(l);
        }
        if let Some(role) = filter.role {
            params["role"] = serde_json::Value::from(role);
        }
        if let Some(code) = filter.referral_code {
            params["referral_code"] = serde_json::Value::from(code);
        }
        if let Some(code) = filter.referred_by_code {
            params["referred_by_code"] = serde_json::Value::from(code);
        }
        let qr: QueryResult<Vec<AccountListItem>> = self.rpc_query("listAccounts", params)?;
        let page = page_from_query(&qr, filter.offset, filter.limit, qr.data.len());
        Ok(ListAccountsResponse {
            offset: page.offset,
            total: page.total_or(qr.data.len()),
            accounts: qr.data,
        })
    }

    pub fn all_bbos(&self) -> Result<Vec<AllBboItem>, ClientError> {
        self.rpc_query("getAllBBOs", Value::Null)
            .map(|qr: QueryResult<Vec<AllBboItem>>| qr.data)
    }

    pub fn all_marks(&self) -> Result<std::collections::BTreeMap<String, String>, ClientError> {
        self.rpc_query("getAllMarks", Value::Null)
            .map(|qr: QueryResult<std::collections::BTreeMap<String, String>>| qr.data)
    }

    pub fn market_summary(&self, symbol: &str) -> Result<MarketSummaryResponse, ClientError> {
        self.rpc_query("getMarketSummary", serde_json::json!({ "symbol": symbol }))
            .map(|qr: QueryResult<MarketSummaryResponse>| qr.data)
    }

    // `getQuoteInBlock` is no longer a query method (WS only).

    pub fn global_stats(&self) -> Result<GlobalStatsResponse, ClientError> {
        self.rpc_query("getGlobalStats", Value::Null)
            .map(|qr: QueryResult<GlobalStatsResponse>| qr.data)
    }

    pub fn position(&self, address: &str, symbol: &str) -> Result<PositionRecord, ClientError> {
        self.rpc_query(
            "getPosition",
            serde_json::json!({ "address": address, "symbol": symbol }),
        )
        .map(|qr: QueryResult<PositionRecord>| qr.data)
    }

    /// Like [`Self::position`], but maps `-32004` (no position record for this market) to `Ok(None)`.
    ///
    /// Use this to read per-market settings (leverage, margin mode) after `SetLeverage` even when
    /// `size == 0`. Do **not** use `getAccount.positions` for leverage — that map only includes
    /// markets with open size.
    pub fn try_position(
        &self,
        address: &str,
        symbol: &str,
    ) -> Result<Option<PositionRecord>, ClientError> {
        match self.position(address, symbol) {
            Ok(pos) => Ok(Some(pos)),
            Err(e) if e.is_resource_not_found() => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn all_open_orders(
        &self,
        symbol: Option<&str>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<AllOpenOrdersResponse, ClientError> {
        let mut params = serde_json::json!({});
        if let Some(s) = symbol {
            params["symbol"] = serde_json::Value::from(s);
        }
        if let Some(o) = offset {
            params["offset"] = serde_json::Value::from(o);
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        let qr: QueryResult<Vec<RestingOrderSummary>> =
            self.rpc_query("getAllOpenOrders", params)?;
        let page = page_from_query(&qr, offset, limit, qr.data.len());
        Ok(AllOpenOrdersResponse {
            offset: page.offset,
            total: page.total_or(qr.data.len()),
            orders: qr.data,
        })
    }

    pub fn trigger_orders(
        &self,
        address: &str,
        symbol: Option<&str>,
    ) -> Result<TriggerOrdersResponse, ClientError> {
        let mut params = serde_json::json!({ "address": address });
        if let Some(s) = symbol {
            params["symbol"] = serde_json::Value::from(s);
        }
        self.rpc_query("getTriggerOrders", params)
            .map(|qr: QueryResult<TriggerOrdersResponse>| qr.data)
    }

    pub fn oco_pairs(
        &self,
        address: &str,
        symbol: Option<&str>,
    ) -> Result<OcoPairsResponse, ClientError> {
        let mut params = serde_json::json!({ "address": address });
        if let Some(s) = symbol {
            params["symbol"] = serde_json::Value::from(s);
        }
        self.rpc_query("getOcoPairs", params)
            .map(|qr: QueryResult<OcoPairsResponse>| qr.data)
    }

    pub fn trigger_order(&self, trigger_id: u64) -> Result<TriggerOrderResponse, ClientError> {
        self.rpc_query(
            "getTriggerOrder",
            serde_json::json!({ "trigger_id": trigger_id }),
        )
        .map(|qr: QueryResult<TriggerOrderResponse>| qr.data)
    }

    pub fn oco_pair(&self, pair_id: u64) -> Result<OcoPairResponse, ClientError> {
        self.rpc_query("getOcoPair", serde_json::json!({ "pair_id": pair_id }))
            .map(|qr: QueryResult<OcoPairResponse>| qr.data)
    }

    pub fn all_trigger_orders(
        &self,
        symbol: Option<&str>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<AllTriggerOrdersResponse, ClientError> {
        let mut params = serde_json::json!({});
        if let Some(s) = symbol {
            params["symbol"] = serde_json::Value::from(s);
        }
        if let Some(o) = offset {
            params["offset"] = serde_json::Value::from(o);
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        let qr: QueryResult<Vec<TriggerOrderResponse>> =
            self.rpc_query("getAllTriggerOrders", params)?;
        let page = page_from_query(&qr, offset, limit, qr.data.len());
        Ok(AllTriggerOrdersResponse {
            offset: page.offset,
            total: page.total_or(qr.data.len()),
            triggers: qr.data,
        })
    }

    pub fn all_oco_pairs(
        &self,
        symbol: Option<&str>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<AllOcoPairsResponse, ClientError> {
        let mut params = serde_json::json!({});
        if let Some(s) = symbol {
            params["symbol"] = serde_json::Value::from(s);
        }
        if let Some(o) = offset {
            params["offset"] = serde_json::Value::from(o);
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        let qr: QueryResult<Vec<OcoPairResponse>> = self.rpc_query("getAllOcoPairs", params)?;
        let page = page_from_query(&qr, offset, limit, qr.data.len());
        Ok(AllOcoPairsResponse {
            offset: page.offset,
            total: page.total_or(qr.data.len()),
            pairs: qr.data,
        })
    }

    pub fn deposits_by_owner(
        &self,
        owner: &str,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<BridgeDepositsListResponse, ClientError> {
        let mut params = serde_json::json!({ "owner": owner });
        if let Some(o) = offset {
            params["offset"] = serde_json::Value::from(o);
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        let qr: QueryResult<Vec<DepositRecord>> = self.rpc_query("listBridgeDeposits", params)?;
        let page = page_from_query(&qr, offset, limit, qr.data.len());
        Ok(BridgeDepositsListResponse {
            offset: page.offset,
            total: page.total_or(qr.data.len()),
            deposits: qr.data,
        })
    }

    pub fn withdrawals_by_owner(
        &self,
        owner: &str,
        status: Option<&str>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<BridgeWithdrawalsListResponse, ClientError> {
        let mut params = serde_json::json!({ "owner": owner });
        if let Some(s) = status {
            params["status"] = serde_json::Value::from(s);
        }
        if let Some(o) = offset {
            params["offset"] = serde_json::Value::from(o);
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        let qr: QueryResult<Vec<WithdrawRecord>> =
            self.rpc_query("listBridgeWithdrawals", params)?;
        let page = page_from_query(&qr, offset, limit, qr.data.len());
        Ok(BridgeWithdrawalsListResponse {
            offset: page.offset,
            total: page.total_or(qr.data.len()),
            withdrawals: qr.data,
        })
    }

    pub fn accounts_by_role(
        &self,
        role: &str,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<ListAccountsResponse, ClientError> {
        self.list_accounts_filtered(ListAccountsFilter {
            offset,
            limit,
            role: Some(role.to_string()),
            ..Default::default()
        })
    }

    pub fn search_accounts(
        &self,
        referral_code: Option<&str>,
        referred_by_code: Option<&str>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<ListAccountsResponse, ClientError> {
        self.list_accounts_filtered(ListAccountsFilter {
            offset,
            limit,
            referral_code: referral_code.map(str::to_string),
            referred_by_code: referred_by_code.map(str::to_string),
            ..Default::default()
        })
    }

    pub fn top_accounts(
        &self,
        sort_by: &str,
        limit: Option<usize>,
    ) -> Result<Vec<TopAccountItem>, ClientError> {
        let mut params = serde_json::json!({ "sort_by": sort_by });
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        self.rpc_query("getTopAccounts", params)
            .map(|qr: QueryResult<Vec<TopAccountItem>>| qr.data)
    }

    // ── Raw query (for generic CLI/scripting) ────────────────────────────────

    /// Low-level JSON-RPC query: returns the raw `result` field.
    /// Prefer typed methods when available; this is the escape hatch for ad-hoc queries.
    pub fn raw_query(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, ClientError> {
        let req = RpcRequest {
            jsonrpc: "2.0",
            id: self.next_id(),
            method,
            params,
        };
        let raw: serde_json::Value = self.post_json("/api/v1/query", &req)?;
        if let Some(err) = raw.get("error") {
            return Err(ClientError::from_rpc_value(err));
        }
        Ok(raw
            .get("result")
            .cloned()
            .unwrap_or(serde_json::Value::Null))
    }

    // ── REST aliases (cacheable GET) ───────────────────────────────────────

    pub fn markets_rest(&self) -> Result<Vec<MarketListItem>, ClientError> {
        self.get_query_data("/api/v1/markets")
    }

    pub fn market_rest(&self, symbol: &str) -> Result<MarketDetailResponse, ClientError> {
        self.get_query_data(&format!("/api/v1/markets/{symbol}"))
    }

    pub fn orderbook_rest(
        &self,
        symbol: &str,
        depth: Option<usize>,
    ) -> Result<OrderbookResponse, ClientError> {
        let path = if let Some(d) = depth {
            format!("/api/v1/orderbook/{symbol}?depth={d}")
        } else {
            format!("/api/v1/orderbook/{symbol}")
        };
        self.get_query_data(&path)
    }

    pub fn market_summary_rest(&self, symbol: &str) -> Result<MarketSummaryResponse, ClientError> {
        self.get_query_data(&format!("/api/v1/markets/{symbol}/summary"))
    }

    pub fn bbos_rest(&self) -> Result<Vec<AllBboItem>, ClientError> {
        self.get_query_data("/api/v1/bbos")
    }

    pub fn marks_rest(&self) -> Result<std::collections::BTreeMap<String, String>, ClientError> {
        self.get_query_data($crate::routes::GET_MARKS)
    }

    pub fn stats_rest(&self) -> Result<GlobalStatsResponse, ClientError> {
        self.get_query_data("/api/v1/stats")
    }

    /// WebSocket full URL (`ws://` / `wss://`).
    pub fn websocket_url(&self) -> Result<Url, ClientError> {
        let mut url = self.url("/api/v1/ws")?;
        let scheme: String = url.scheme().to_string();
        let ws_scheme = match scheme.as_str() {
            "http" => "ws",
            "https" => "wss",
            s @ ("ws" | "wss") => s,
            other => {
                return Err(ClientError::Api {
                    status: 0,
                    body: format!("unsupported scheme: {other}"),
                })
            }
        };
        url.set_scheme(ws_scheme).map_err(|_| ClientError::Api {
            status: 0,
            body: "failed to set ws scheme".into(),
        })?;
        Ok(url)
    }
        }
    };
    (async, $client:ident) => {
        impl $client {
    /// Sign with master key and submit (`domain_chain_id` defaults to `chain_id`).
    pub async fn submit_signed(
        &self,
        chain_id: u64,
        network_tag: &str,
        sk: &PrivateKeySigner,
        nonce: u64,
        action: Action,
    ) -> Result<TxReceiptResponse, ClientError> {
        self.submit_signed_with_domain(chain_id, chain_id, network_tag, sk, nonce, action).await
    }

    /// Like [`Self::submit_signed`], but returns [`ClientError::TxRejected`] on `kept-reject`.
    pub async fn submit_signed_accepted(
        &self,
        chain_id: u64,
        network_tag: &str,
        sk: &PrivateKeySigner,
        nonce: u64,
        action: Action,
    ) -> Result<TxReceiptResponse, ClientError> {
        self.submit_signed(chain_id, network_tag, sk, nonce, action).await
            .and_then(|r| r.ensure_accepted())
    }

    /// Sign with master key and submit, with an explicit EIP-712 `domain_chain_id`.
    ///
    /// For User-Signed actions (`WithdrawRequest` / `RegisterAgent` / `RevokeAgent`),
    /// set `domain_chain_id` to the user's connected EVM network ID (e.g. BSC = 56).
    pub async fn submit_signed_with_domain(
        &self,
        chain_id: u64,
        domain_chain_id: u64,
        network_tag: &str,
        sk: &PrivateKeySigner,
        nonce: u64,
        action: Action,
    ) -> Result<TxReceiptResponse, ClientError> {
        let env = master_signed_envelope(chain_id, domain_chain_id, network_tag, sk, nonce, action);
        self.submit_action(&env).await
    }

    /// Like [`Self::submit_signed_with_domain`], but returns [`ClientError::TxRejected`] on `kept-reject`.
    pub async fn submit_signed_with_domain_accepted(
        &self,
        chain_id: u64,
        domain_chain_id: u64,
        network_tag: &str,
        sk: &PrivateKeySigner,
        nonce: u64,
        action: Action,
    ) -> Result<TxReceiptResponse, ClientError> {
        self.submit_signed_with_domain(
            chain_id,
            domain_chain_id,
            network_tag,
            sk,
            nonce,
            action,
        ).await
        .and_then(|r| r.ensure_accepted())
    }

    /// Sign with agent key and submit (`domain_chain_id` defaults to `chain_id`).
    pub async fn submit_agent_signed(
        &self,
        chain_id: u64,
        network_tag: &str,
        signer: Address20,
        agent_sk: &PrivateKeySigner,
        nonce: u64,
        action: Action,
    ) -> Result<TxReceiptResponse, ClientError> {
        self.submit_agent_signed_with_domain(
            chain_id,
            chain_id,
            network_tag,
            signer,
            agent_sk,
            nonce,
            action,
        ).await
    }

    /// Sign with agent key and submit, with an explicit EIP-712 `domain_chain_id`.
    pub async fn submit_agent_signed_with_domain(
        &self,
        chain_id: u64,
        domain_chain_id: u64,
        network_tag: &str,
        signer: Address20,
        agent_sk: &PrivateKeySigner,
        nonce: u64,
        action: Action,
    ) -> Result<TxReceiptResponse, ClientError> {
        let env = agent_signed_envelope(
            chain_id,
            domain_chain_id,
            network_tag,
            signer,
            agent_sk,
            nonce,
            action,
        );
        self.submit_action(&env).await
    }

    // ── Chain / health ─────────────────────────────────────────────────────

    pub async fn health_rest(&self) -> Result<HealthResponse, ClientError> {
        self.get_json("/api/v1/health").await
    }

    pub async fn health(&self) -> Result<HealthResponse, ClientError> {
        let qr: QueryResult<Value> = self.rpc_query("getHealth", Value::Null).await?;
        Ok(HealthResponse {
            status: qr
                .data
                .get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("ok")
                .into(),
            height: qr.height,
        })
    }

    // ── Blocks ─────────────────────────────────────────────────────────────

    pub async fn block_latest(&self) -> Result<BlockResponse, ClientError> {
        self.rpc_query("getBlock", Value::Null).await
            .map(|qr: QueryResult<BlockResponse>| qr.data)
    }

    pub async fn block_by_height(&self, height: u64) -> Result<BlockResponse, ClientError> {
        self.rpc_query("getBlock", serde_json::json!({ "height": height })).await
            .map(|qr: QueryResult<BlockResponse>| qr.data)
    }

    pub async fn block_events(
        &self,
        height: u64,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<BlockEventsResponse, ClientError> {
        let mut params = serde_json::json!({ "height": height });
        if let Some(o) = offset {
            params["offset"] = serde_json::Value::from(o);
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        // 节点读骨架：`result.data` 是事件数组本身，分页在 `result.page`。
        // （旧实现误把 `data` 当作 `{height,offset,total,events}` 结构 → 解码失败。）
        let qr: QueryResult<Vec<serde_json::Value>> = self.rpc_query("getBlockEvents", params).await?;
        let page = page_from_query(&qr, offset, limit, qr.data.len());
        Ok(BlockEventsResponse {
            height: qr.height,
            offset: page.offset,
            total: page.total_or(qr.data.len()),
            events: qr.data,
        })
    }

    pub async fn get_tx(&self, hash: &str) -> Result<TxReceiptResponse, ClientError> {
        self.rpc_query("getTx", serde_json::json!({ "hash": hash })).await
            .map(|qr: QueryResult<TxReceiptResponse>| qr.data)
    }

    // ── Markets / orderbook ────────────────────────────────────────────────

    pub async fn orderbook(&self, symbol: &str) -> Result<OrderbookResponse, ClientError> {
        self.orderbook_with_depth(symbol, None).await
    }

    pub async fn orderbook_with_depth(
        &self,
        symbol: &str,
        depth: Option<usize>,
    ) -> Result<OrderbookResponse, ClientError> {
        let mut params = serde_json::json!({ "symbol": symbol });
        if let Some(d) = depth {
            params["depth"] = serde_json::Value::from(d);
        }
        self.rpc_query("getOrderbook", params).await
            .map(|qr: QueryResult<OrderbookResponse>| qr.data)
    }

    pub async fn list_markets(&self) -> Result<Vec<MarketListItem>, ClientError> {
        self.rpc_query("getMarkets", Value::Null).await
            .map(|qr: QueryResult<Vec<MarketListItem>>| qr.data)
    }

    pub async fn market(&self, symbol: &str) -> Result<MarketDetailResponse, ClientError> {
        self.rpc_query("getMarket", serde_json::json!({ "symbol": symbol })).await
            .map(|qr: QueryResult<MarketDetailResponse>| qr.data)
    }

    // ── Account ────────────────────────────────────────────────────────────

    pub async fn account(&self, address: &str) -> Result<AccountSummaryResponse, ClientError> {
        self.rpc_query("getAccount", serde_json::json!({ "address": address })).await
            .map(|qr: QueryResult<AccountSummaryResponse>| qr.data)
    }

    pub async fn account_orders(&self, address: &str) -> Result<AccountOrdersResponse, ClientError> {
        self.rpc_query(
            "getAccountOrders",
            serde_json::json!({ "address": address }),
        ).await
        .map(|qr: QueryResult<AccountOrdersResponse>| qr.data)
    }

    // ── Bridge ─────────────────────────────────────────────────────────────

    pub async fn bridge_settlement(&self) -> Result<BridgeSettlementResponse, ClientError> {
        self.rpc_query("getBridgeSettlement", Value::Null).await
            .map(|qr: QueryResult<BridgeSettlementResponse>| qr.data)
    }

    pub async fn bridge_deposit(&self, seq: u64) -> Result<BridgeDepositResponse, ClientError> {
        self.rpc_query("getBridgeDeposit", serde_json::json!({ "seq": seq })).await
            .map(|qr: QueryResult<BridgeDepositResponse>| qr.data)
    }

    /// 按外部链充值引用 `(chain, seq)` 查询充值单。未找到时返回 `Ok(None)`。
    pub async fn bridge_deposit_by_external_ref(
        &self,
        chain: &str,
        seq: u64,
    ) -> Result<Option<BridgeDepositResponse>, ClientError> {
        self.rpc_query(
            "getBridgeDepositByExternalRef",
            serde_json::json!({ "chain": chain.to_lowercase(), "seq": seq }),
        ).await
        .map(|qr: QueryResult<Option<BridgeDepositResponse>>| qr.data)
    }

    pub async fn bridge_deposits(
        &self,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<BridgeDepositsListResponse, ClientError> {
        let mut params = serde_json::json!({});
        if let Some(o) = offset {
            params["offset"] = serde_json::Value::from(o);
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        let qr: QueryResult<Vec<DepositRecord>> = self.rpc_query("listBridgeDeposits", params).await?;
        let page = page_from_query(&qr, offset, limit, qr.data.len());
        Ok(BridgeDepositsListResponse {
            offset: page.offset,
            total: page.total_or(qr.data.len()),
            deposits: qr.data,
        })
    }

    pub async fn bridge_withdrawal(&self, id: u64) -> Result<BridgeWithdrawalResponse, ClientError> {
        self.rpc_query("getBridgeWithdrawal", serde_json::json!({ "id": id })).await
            .map(|qr: QueryResult<BridgeWithdrawalResponse>| qr.data)
    }

    pub async fn bridge_withdrawals(
        &self,
        status: Option<&str>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<BridgeWithdrawalsListResponse, ClientError> {
        let mut params = serde_json::json!({});
        if let Some(s) = status {
            params["status"] = serde_json::Value::from(s);
        }
        if let Some(o) = offset {
            params["offset"] = serde_json::Value::from(o);
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        let qr: QueryResult<Vec<WithdrawRecord>> =
            self.rpc_query("listBridgeWithdrawals", params).await?;
        let page = page_from_query(&qr, offset, limit, qr.data.len());
        Ok(BridgeWithdrawalsListResponse {
            offset: page.offset,
            total: page.total_or(qr.data.len()),
            withdrawals: qr.data,
        })
    }

    // ── Meta ───────────────────────────────────────────────────────────────

    pub async fn actions_meta(&self) -> Result<ActionsMetaResponse, ClientError> {
        self.rpc_query("getActionsMeta", Value::Null).await
            .map(|qr: QueryResult<ActionsMetaResponse>| qr.data)
    }

    pub async fn bootstrap(
        &self,
        address: Option<&str>,
        symbols: Option<&[String]>,
    ) -> Result<Value, ClientError> {
        self.bootstrap_with_options(address, symbols, None).await
    }

    pub async fn bootstrap_typed(
        &self,
        address: Option<&str>,
        symbols: Option<&[String]>,
    ) -> Result<BootstrapResponse, ClientError> {
        self.bootstrap_typed_with_options(address, symbols, None).await
    }

    pub async fn bootstrap_with_options(
        &self,
        address: Option<&str>,
        symbols: Option<&[String]>,
        book_depth: Option<usize>,
    ) -> Result<Value, ClientError> {
        self.rpc_query(
            "getBootstrap",
            bootstrap_params(address, symbols, book_depth),
        ).await
        .map(|qr: QueryResult<Value>| qr.data)
    }

    pub async fn bootstrap_typed_with_options(
        &self,
        address: Option<&str>,
        symbols: Option<&[String]>,
        book_depth: Option<usize>,
    ) -> Result<BootstrapResponse, ClientError> {
        self.rpc_query(
            "getBootstrap",
            bootstrap_params(address, symbols, book_depth),
        ).await
        .map(|qr: QueryResult<BootstrapResponse>| qr.data)
    }

    // ── History / layer-2 queries ──────────────────────────────────────────

    pub async fn candles(
        &self,
        symbol: &str,
        interval_ms: u64,
        start_time_ms: Option<u64>,
        end_time_ms: Option<u64>,
        limit: Option<usize>,
    ) -> Result<Vec<CandleResponse>, ClientError> {
        let mut params = serde_json::json!({ "symbol": symbol, "interval_ms": interval_ms });
        if let Some(t) = start_time_ms {
            params["start_time_ms"] = serde_json::Value::from(t);
        }
        if let Some(t) = end_time_ms {
            params["end_time_ms"] = serde_json::Value::from(t);
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        self.rpc_query("getCandles", params).await
            .map(|qr: QueryResult<Vec<CandleResponse>>| qr.data)
    }

    pub async fn recent_trades(
        &self,
        symbol: &str,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<Vec<TradeResponse>, ClientError> {
        let mut params = serde_json::json!({ "symbol": symbol });
        if let Some(o) = offset {
            params["offset"] = serde_json::Value::from(o);
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        self.rpc_query("getRecentTrades", params).await
            .map(|qr: QueryResult<Vec<TradeResponse>>| qr.data)
    }

    pub async fn user_fills(
        &self,
        address: &str,
        symbol: Option<&str>,
        start_time_ms: Option<u64>,
        end_time_ms: Option<u64>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<Vec<UserFillResponse>, ClientError> {
        let mut params = serde_json::json!({ "address": address });
        if let Some(s) = symbol {
            params["symbol"] = serde_json::Value::from(s);
        }
        if let Some(t) = start_time_ms {
            params["start_time_ms"] = serde_json::Value::from(t);
        }
        if let Some(t) = end_time_ms {
            params["end_time_ms"] = serde_json::Value::from(t);
        }
        if let Some(o) = offset {
            params["offset"] = serde_json::Value::from(o);
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        self.rpc_query("getUserFills", params).await
            .map(|qr: QueryResult<Vec<UserFillResponse>>| qr.data)
    }

    pub async fn order_status(&self, order_id: u64) -> Result<OrderStatusResponse, ClientError> {
        self.rpc_query(
            "getOrderStatus",
            serde_json::json!({ "order_id": order_id }),
        ).await
        .map(|qr: QueryResult<OrderStatusResponse>| qr.data)
    }

    pub async fn order_status_by_cloid(
        &self,
        address: &str,
        symbol: &str,
        client_order_id: &str,
    ) -> Result<OrderStatusResponse, ClientError> {
        self.rpc_query(
            "getOrderStatus",
            serde_json::json!({
                "address": address,
                "symbol": symbol,
                "client_order_id": client_order_id,
            }),
        ).await
        .map(|qr: QueryResult<OrderStatusResponse>| qr.data)
    }

    pub async fn exchange_config(&self) -> Result<ExchangeConfigResponse, ClientError> {
        self.rpc_query("getExchangeConfig", Value::Null).await
            .map(|qr: QueryResult<ExchangeConfigResponse>| qr.data)
    }

    pub async fn liquidatable_positions(
        &self,
        symbol: Option<&str>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<Vec<LiquidatablePosition>, ClientError> {
        let mut params = serde_json::json!({});
        if let Some(s) = symbol {
            params["symbol"] = serde_json::Value::from(s);
        }
        if let Some(o) = offset {
            params["offset"] = serde_json::Value::from(o);
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        self.rpc_query("getLiquidatablePositions", params).await
            .map(|qr: QueryResult<Vec<LiquidatablePosition>>| qr.data)
    }

    pub async fn estimated_liquidation_price(
        &self,
        symbol: &str,
        size: &str,
        entry_price: Option<&str>,
        leverage: u32,
    ) -> Result<EstimatedLiquidationResponse, ClientError> {
        let mut params =
            serde_json::json!({ "symbol": symbol, "size": size, "leverage": leverage });
        if let Some(ep) = entry_price {
            params["entry_price"] = serde_json::Value::from(ep);
        }
        self.rpc_query("getEstimatedLiquidationPrice", params).await
            .map(|qr: QueryResult<EstimatedLiquidationResponse>| qr.data)
    }

    pub async fn user_rate_limit(&self, address: &str) -> Result<UserRateLimitResponse, ClientError> {
        self.rpc_query(
            "getUserRateLimit",
            serde_json::json!({ "address": address }),
        ).await
        .map(|qr: QueryResult<UserRateLimitResponse>| qr.data)
    }

    pub async fn admin_audit_log(
        &self,
        signer: Option<&str>,
        start_time_ms: Option<u64>,
        end_time_ms: Option<u64>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<Vec<AdminAuditEntry>, ClientError> {
        let mut params = serde_json::json!({});
        if let Some(s) = signer {
            params["signer"] = serde_json::Value::from(s);
        }
        if let Some(t) = start_time_ms {
            params["start_time_ms"] = serde_json::Value::from(t);
        }
        if let Some(t) = end_time_ms {
            params["end_time_ms"] = serde_json::Value::from(t);
        }
        if let Some(o) = offset {
            params["offset"] = serde_json::Value::from(o);
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        self.rpc_query("getAdminAuditLog", params).await
            .map(|qr: QueryResult<Vec<AdminAuditEntry>>| qr.data)
    }

    pub async fn user_fees(&self, address: &str) -> Result<UserFeesResponse, ClientError> {
        self.rpc_query("getUserFees", serde_json::json!({ "address": address })).await
            .map(|qr: QueryResult<UserFeesResponse>| qr.data)
    }

    pub async fn referral(&self, address: &str) -> Result<ReferralResponse, ClientError> {
        self.rpc_query("getReferral", serde_json::json!({ "address": address })).await
            .map(|qr: QueryResult<ReferralResponse>| qr.data)
    }

    // ── Extended index/query surface ───────────────────────────────────────

    pub async fn list_accounts(
        &self,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<ListAccountsResponse, ClientError> {
        self.list_accounts_filtered(ListAccountsFilter {
            offset,
            limit,
            ..Default::default()
        })
        .await
    }

    /// Unified `listAccounts` with optional role / referral filters.
    pub async fn list_accounts_filtered(
        &self,
        filter: ListAccountsFilter,
    ) -> Result<ListAccountsResponse, ClientError> {
        let mut params = serde_json::json!({});
        if let Some(o) = filter.offset {
            params["offset"] = serde_json::Value::from(o);
        }
        if let Some(l) = filter.limit {
            params["limit"] = serde_json::Value::from(l);
        }
        if let Some(role) = filter.role {
            params["role"] = serde_json::Value::from(role);
        }
        if let Some(code) = filter.referral_code {
            params["referral_code"] = serde_json::Value::from(code);
        }
        if let Some(code) = filter.referred_by_code {
            params["referred_by_code"] = serde_json::Value::from(code);
        }
        let qr: QueryResult<Vec<AccountListItem>> = self.rpc_query("listAccounts", params).await?;
        let page = page_from_query(&qr, filter.offset, filter.limit, qr.data.len());
        Ok(ListAccountsResponse {
            offset: page.offset,
            total: page.total_or(qr.data.len()),
            accounts: qr.data,
        })
    }

    pub async fn all_bbos(&self) -> Result<Vec<AllBboItem>, ClientError> {
        self.rpc_query("getAllBBOs", Value::Null).await
            .map(|qr: QueryResult<Vec<AllBboItem>>| qr.data)
    }

    pub async fn all_marks(&self) -> Result<std::collections::BTreeMap<String, String>, ClientError> {
        self.rpc_query("getAllMarks", Value::Null).await
            .map(|qr: QueryResult<std::collections::BTreeMap<String, String>>| qr.data)
    }

    pub async fn market_summary(&self, symbol: &str) -> Result<MarketSummaryResponse, ClientError> {
        self.rpc_query("getMarketSummary", serde_json::json!({ "symbol": symbol })).await
            .map(|qr: QueryResult<MarketSummaryResponse>| qr.data)
    }

    // `getQuoteInBlock` is no longer a query method (WS only).

    pub async fn global_stats(&self) -> Result<GlobalStatsResponse, ClientError> {
        self.rpc_query("getGlobalStats", Value::Null).await
            .map(|qr: QueryResult<GlobalStatsResponse>| qr.data)
    }

    pub async fn position(&self, address: &str, symbol: &str) -> Result<PositionRecord, ClientError> {
        self.rpc_query(
            "getPosition",
            serde_json::json!({ "address": address, "symbol": symbol }),
        ).await
        .map(|qr: QueryResult<PositionRecord>| qr.data)
    }

    /// Like [`Self::position`], but maps `-32004` (no position record for this market) to `Ok(None)`.
    ///
    /// Use this to read per-market settings (leverage, margin mode) after `SetLeverage` even when
    /// `size == 0`. Do **not** use `getAccount.positions` for leverage — that map only includes
    /// markets with open size.
    pub async fn try_position(
        &self,
        address: &str,
        symbol: &str,
    ) -> Result<Option<PositionRecord>, ClientError> {
        match self.position(address, symbol).await {
            Ok(pos) => Ok(Some(pos)),
            Err(e) if e.is_resource_not_found() => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn all_open_orders(
        &self,
        symbol: Option<&str>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<AllOpenOrdersResponse, ClientError> {
        let mut params = serde_json::json!({});
        if let Some(s) = symbol {
            params["symbol"] = serde_json::Value::from(s);
        }
        if let Some(o) = offset {
            params["offset"] = serde_json::Value::from(o);
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        let qr: QueryResult<Vec<RestingOrderSummary>> =
            self.rpc_query("getAllOpenOrders", params).await?;
        let page = page_from_query(&qr, offset, limit, qr.data.len());
        Ok(AllOpenOrdersResponse {
            offset: page.offset,
            total: page.total_or(qr.data.len()),
            orders: qr.data,
        })
    }

    pub async fn trigger_orders(
        &self,
        address: &str,
        symbol: Option<&str>,
    ) -> Result<TriggerOrdersResponse, ClientError> {
        let mut params = serde_json::json!({ "address": address });
        if let Some(s) = symbol {
            params["symbol"] = serde_json::Value::from(s);
        }
        self.rpc_query("getTriggerOrders", params).await
            .map(|qr: QueryResult<TriggerOrdersResponse>| qr.data)
    }

    pub async fn oco_pairs(
        &self,
        address: &str,
        symbol: Option<&str>,
    ) -> Result<OcoPairsResponse, ClientError> {
        let mut params = serde_json::json!({ "address": address });
        if let Some(s) = symbol {
            params["symbol"] = serde_json::Value::from(s);
        }
        self.rpc_query("getOcoPairs", params).await
            .map(|qr: QueryResult<OcoPairsResponse>| qr.data)
    }

    pub async fn trigger_order(&self, trigger_id: u64) -> Result<TriggerOrderResponse, ClientError> {
        self.rpc_query(
            "getTriggerOrder",
            serde_json::json!({ "trigger_id": trigger_id }),
        ).await
        .map(|qr: QueryResult<TriggerOrderResponse>| qr.data)
    }

    pub async fn oco_pair(&self, pair_id: u64) -> Result<OcoPairResponse, ClientError> {
        self.rpc_query("getOcoPair", serde_json::json!({ "pair_id": pair_id })).await
            .map(|qr: QueryResult<OcoPairResponse>| qr.data)
    }

    pub async fn all_trigger_orders(
        &self,
        symbol: Option<&str>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<AllTriggerOrdersResponse, ClientError> {
        let mut params = serde_json::json!({});
        if let Some(s) = symbol {
            params["symbol"] = serde_json::Value::from(s);
        }
        if let Some(o) = offset {
            params["offset"] = serde_json::Value::from(o);
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        let qr: QueryResult<Vec<TriggerOrderResponse>> =
            self.rpc_query("getAllTriggerOrders", params).await?;
        let page = page_from_query(&qr, offset, limit, qr.data.len());
        Ok(AllTriggerOrdersResponse {
            offset: page.offset,
            total: page.total_or(qr.data.len()),
            triggers: qr.data,
        })
    }

    pub async fn all_oco_pairs(
        &self,
        symbol: Option<&str>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<AllOcoPairsResponse, ClientError> {
        let mut params = serde_json::json!({});
        if let Some(s) = symbol {
            params["symbol"] = serde_json::Value::from(s);
        }
        if let Some(o) = offset {
            params["offset"] = serde_json::Value::from(o);
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        let qr: QueryResult<Vec<OcoPairResponse>> = self.rpc_query("getAllOcoPairs", params).await?;
        let page = page_from_query(&qr, offset, limit, qr.data.len());
        Ok(AllOcoPairsResponse {
            offset: page.offset,
            total: page.total_or(qr.data.len()),
            pairs: qr.data,
        })
    }

    pub async fn deposits_by_owner(
        &self,
        owner: &str,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<BridgeDepositsListResponse, ClientError> {
        let mut params = serde_json::json!({ "owner": owner });
        if let Some(o) = offset {
            params["offset"] = serde_json::Value::from(o);
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        let qr: QueryResult<Vec<DepositRecord>> = self.rpc_query("listBridgeDeposits", params).await?;
        let page = page_from_query(&qr, offset, limit, qr.data.len());
        Ok(BridgeDepositsListResponse {
            offset: page.offset,
            total: page.total_or(qr.data.len()),
            deposits: qr.data,
        })
    }

    pub async fn withdrawals_by_owner(
        &self,
        owner: &str,
        status: Option<&str>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<BridgeWithdrawalsListResponse, ClientError> {
        let mut params = serde_json::json!({ "owner": owner });
        if let Some(s) = status {
            params["status"] = serde_json::Value::from(s);
        }
        if let Some(o) = offset {
            params["offset"] = serde_json::Value::from(o);
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        let qr: QueryResult<Vec<WithdrawRecord>> =
            self.rpc_query("listBridgeWithdrawals", params).await?;
        let page = page_from_query(&qr, offset, limit, qr.data.len());
        Ok(BridgeWithdrawalsListResponse {
            offset: page.offset,
            total: page.total_or(qr.data.len()),
            withdrawals: qr.data,
        })
    }

    pub async fn accounts_by_role(
        &self,
        role: &str,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<ListAccountsResponse, ClientError> {
        self.list_accounts_filtered(ListAccountsFilter {
            offset,
            limit,
            role: Some(role.to_string()),
            ..Default::default()
        })
        .await
    }

    pub async fn search_accounts(
        &self,
        referral_code: Option<&str>,
        referred_by_code: Option<&str>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<ListAccountsResponse, ClientError> {
        self.list_accounts_filtered(ListAccountsFilter {
            offset,
            limit,
            referral_code: referral_code.map(str::to_string),
            referred_by_code: referred_by_code.map(str::to_string),
            ..Default::default()
        })
        .await
    }

    pub async fn top_accounts(
        &self,
        sort_by: &str,
        limit: Option<usize>,
    ) -> Result<Vec<TopAccountItem>, ClientError> {
        let mut params = serde_json::json!({ "sort_by": sort_by });
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::from(l);
        }
        self.rpc_query("getTopAccounts", params).await
            .map(|qr: QueryResult<Vec<TopAccountItem>>| qr.data)
    }

    // ── Raw query (for generic CLI/scripting) ────────────────────────────────

    /// Low-level JSON-RPC query: returns the raw `result` field.
    /// Prefer typed methods when available; this is the escape hatch for ad-hoc queries.
    pub async fn raw_query(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, ClientError> {
        let req = RpcRequest {
            jsonrpc: "2.0",
            id: self.next_id(),
            method,
            params,
        };
        let raw: serde_json::Value = self.post_json("/api/v1/query", &req).await?;
        if let Some(err) = raw.get("error") {
            return Err(ClientError::from_rpc_value(err));
        }
        Ok(raw
            .get("result")
            .cloned()
            .unwrap_or(serde_json::Value::Null))
    }

    // ── REST aliases (cacheable GET) ───────────────────────────────────────

    pub async fn markets_rest(&self) -> Result<Vec<MarketListItem>, ClientError> {
        self.get_query_data("/api/v1/markets").await
    }

    pub async fn market_rest(&self, symbol: &str) -> Result<MarketDetailResponse, ClientError> {
        self.get_query_data(&format!("/api/v1/markets/{symbol}")).await
    }

    pub async fn orderbook_rest(
        &self,
        symbol: &str,
        depth: Option<usize>,
    ) -> Result<OrderbookResponse, ClientError> {
        let path = if let Some(d) = depth {
            format!("/api/v1/orderbook/{symbol}?depth={d}")
        } else {
            format!("/api/v1/orderbook/{symbol}")
        };
        self.get_query_data(&path).await
    }

    pub async fn market_summary_rest(&self, symbol: &str) -> Result<MarketSummaryResponse, ClientError> {
        self.get_query_data(&format!("/api/v1/markets/{symbol}/summary")).await
    }

    pub async fn bbos_rest(&self) -> Result<Vec<AllBboItem>, ClientError> {
        self.get_query_data("/api/v1/bbos").await
    }

    pub async fn marks_rest(&self) -> Result<std::collections::BTreeMap<String, String>, ClientError> {
        self.get_query_data($crate::routes::GET_MARKS).await
    }

    pub async fn stats_rest(&self) -> Result<GlobalStatsResponse, ClientError> {
        self.get_query_data("/api/v1/stats").await
    }

    /// WebSocket full URL (`ws://` / `wss://`).
    pub async fn websocket_url(&self) -> Result<Url, ClientError> {
        let mut url = self.url("/api/v1/ws")?;
        let scheme: String = url.scheme().to_string();
        let ws_scheme = match scheme.as_str() {
            "http" => "ws",
            "https" => "wss",
            s @ ("ws" | "wss") => s,
            other => {
                return Err(ClientError::Api {
                    status: 0,
                    body: format!("unsupported scheme: {other}"),
                })
            }
        };
        url.set_scheme(ws_scheme).map_err(|_| ClientError::Api {
            status: 0,
            body: "failed to set ws scheme".into(),
        })?;
        Ok(url)
    }
        }
    };
}
