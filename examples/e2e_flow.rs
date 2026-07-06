//! 端到端流程冒烟测试（生产 RPC）。
//!
//! 需要环境变量 `AURORAN_PRIVATE_KEY`（hex secp256k1），地址由私钥推导。
//!
//! ```bash
//! AURORAN_PRIVATE_KEY=0x... make e2e
//! ```

use auroran_sdk_rust::{
    address_from_verifying_key, close_position_market, find_leverage_updated, format_decimal,
    flatten_account, has_symbol_position, open_position_symbols, parse_decimal, place_order,
    poll_config_from_env, secp256k1_from_hex, set_leverage, submit_accepted, symbol_leverage,
    wait_for_account, wait_for_leverage, AccountOrdersResponse, AccountSummaryResponse,
    AuroranClient, ClientError, MarketListItem, PollConfig, Side, SigningConfig, TimeInForce,
    TxReceiptResponse,
};

const RPC_URL: &str = "https://rpc.auroran.io";
const CHAIN_ID: u64 = 42;
const NETWORK_TAG: &str = "zepto-dev";
const SYMBOL: &str = "BTC";

const ORDER_QTY: &str = "0.00100";
const MARK_TICKS_ABOVE: i128 = 5;
const DEFAULT_LEVERAGE: u32 = 25;

fn env(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_u32(key: &str, default: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn load_signer() -> Result<alloy::signers::local::PrivateKeySigner, String> {
    let key_hex = std::env::var("AURORAN_PRIVATE_KEY")
        .map_err(|_| "请设置环境变量 AURORAN_PRIVATE_KEY（32 字节 secp256k1 十六进制私钥）".to_string())?;
    secp256k1_from_hex(&key_hex)
}

fn owner_hex(owner: &auroran_sdk_rust::Address20) -> String {
    format!("0x{}", hex::encode(owner.as_bytes()))
}

fn bump_price(price: &str, price_decimals: u32, tick_steps: i128) -> Option<String> {
    let raw = parse_decimal(price, price_decimals)?;
    let tick = 10i128.checked_pow(price_decimals)?;
    let bumped = raw.checked_add(tick.checked_mul(tick_steps)?)?;
    Some(format_decimal(bumped, price_decimals))
}

fn side_zh(side: Side) -> &'static str {
    match side {
        Side::Bid => "买",
        Side::Ask => "卖",
    }
}

fn tif_zh(tif: TimeInForce) -> &'static str {
    match tif {
        TimeInForce::Gtc => "直到取消",
        TimeInForce::Ioc => "立即成交或取消",
        TimeInForce::Fok => "全部成交或取消",
        TimeInForce::PostOnly => "只做 Maker",
    }
}

fn signing_config() -> SigningConfig {
    SigningConfig::new(CHAIN_ID, NETWORK_TAG)
}

fn print_positions(label: &str, acct: &AccountSummaryResponse) {
    println!("--- {label} ---");
    println!("余额= {}", acct.balance);
    println!("账户价值= {}", acct.account_value);
    println!("可提现= {}", acct.withdrawable);
    println!(
        "全仓可用(cash)= {} 全仓交易可用= {}",
        acct.cross_cash_available, acct.cross_trading_available
    );
    if acct.positions.is_empty() {
        println!("持仓：无");
        return;
    }
    println!("持仓：");
    for (market_id, pos) in &acct.positions {
        println!(
            "  市场ID= {market_id:?} 交易对= {} 数量= {} 杠杆= {}x 开仓均价= {} 标记价= {} 未实现盈亏= {}",
            pos.symbol, pos.size, pos.leverage, pos.entry_vwap, pos.mark_price, pos.unrealized_pnl
        );
    }
}

fn print_symbol_leverage(label: &str, symbol: &str, leverage: Option<u32>, max_leverage: u32) {
    match leverage {
        Some(0) => println!(
            "{label}：{symbol} 当前杠杆= 0（链上默认，使用 tier 最高杠杆；市场最大 {max_leverage}x）"
        ),
        Some(lev) => println!("{label}：{symbol} 当前杠杆= {lev}x（市场最大 {max_leverage}x）"),
        None => println!(
            "{label}：{symbol} 当前杠杆= 未设置（无市场配置记录；市场最大 {max_leverage}x）"
        ),
    }
}

fn query_and_print_leverage(
    client: &AuroranClient,
    address: &str,
    symbol: &str,
    max_leverage: u32,
    label: &str,
) -> Result<Option<u32>, ClientError> {
    let lev = symbol_leverage(client, address, symbol)?;
    print_symbol_leverage(label, symbol, lev, max_leverage);
    Ok(lev)
}

fn print_open_orders(orders: &AccountOrdersResponse) {
    if orders.orders.is_empty() {
        println!("挂单：无");
        return;
    }
    println!("挂单（{} 笔）：", orders.orders.len());
    for o in &orders.orders {
        println!(
            "  {} {} {} 剩余= {} @ {} 订单ID= {}",
            o.symbol,
            side_zh(o.side),
            tif_zh(o.tif),
            o.remaining,
            o.price,
            o.order_id
        );
    }
}

fn print_receipt(label: &str, receipt: &TxReceiptResponse) {
    println!(
        "{label}：状态= {} 交易哈希= {} 区块高度= {}",
        receipt.status, receipt.tx_hash, receipt.height
    );
    if let Some(ev) = find_leverage_updated(receipt) {
        println!("  事件 LeverageUpdated= {}x (market_id= {:?})", ev.leverage, ev.market_id);
    }
}

fn run_flatten(
    client: &AuroranClient,
    sk: &alloy::signers::local::PrivateKeySigner,
    owner: auroran_sdk_rust::Address20,
    address: &str,
    markets: &[MarketListItem],
    phase: &str,
    sign: &SigningConfig,
    poll: &PollConfig,
) -> Result<(), ClientError> {
    let result = flatten_account(client, sign, sk, owner, address, markets, poll)?;
    if result.skipped {
        println!("{phase}：无需清理（无挂单、无持仓）");
    } else {
        if !result.cancelled_symbols.is_empty() {
            println!("{phase}：已撤单 {:?}", result.cancelled_symbols);
        }
        if !result.closed_symbols.is_empty() {
            println!("{phase}：已平仓 {:?}", result.closed_symbols);
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc = env("AURORAN_RPC_URL", RPC_URL);
    let symbol = env("AURORAN_SYMBOL", SYMBOL);
    let sign = signing_config();
    let poll = poll_config_from_env();

    let sk = load_signer()?;
    let owner = address_from_verifying_key(&sk);
    let address = owner_hex(&owner);
    println!("签名地址= {address}");

    let client = AuroranClient::new(&rpc)?;

    println!("\n== 0. 启动清理（幂等） ==");
    let markets = client.list_markets()?;
    run_flatten(
        &client,
        &sk,
        owner,
        &address,
        &markets,
        "启动清理",
        &sign,
        &poll,
    )?;

    println!("\n== 1. 查询账户余额 ==");
    print_positions("交易前", &client.account(&address)?);

    println!("\n== 2. 查询市场列表 ==");
    for m in &markets {
        println!(
            "  {} 状态= {} 标记价= {} 价格精度= {} 数量精度= {}",
            m.symbol, m.lifecycle, m.mark_price, m.price_decimals, m.size_decimals
        );
    }

    let market = markets
        .iter()
        .find(|m| m.symbol == symbol)
        .ok_or_else(|| format!("未找到市场：{symbol}"))?;

    println!("\n== 3. 查询 {symbol} 盘口 ==");
    let book = client.orderbook_with_depth(&symbol, Some(5))?;
    let best_bid = book.bids.first().map(|l| l.price.as_str());
    let best_ask = book.asks.first().map(|l| l.price.as_str());
    println!(
        "区块高度= {} 数据源= {} 最优买价= {best_bid:?} 最优卖价= {best_ask:?} 标记价= {}",
        book.height, book.source, market.mark_price
    );

    let target_leverage = env_u32("AURORAN_LEVERAGE", DEFAULT_LEVERAGE).min(market.max_leverage);

    println!("\n== 4. 查询杠杆（设置前，getPosition） ==");
    query_and_print_leverage(
        &client,
        &address,
        &symbol,
        market.max_leverage,
        "设置前",
    )?;

    let nonce = client.account(&address)?.nonce;
    println!("\n== 5. 设置杠杆 ==");
    println!("目标杠杆= {target_leverage}x nonce= {nonce}");
    let (receipt, mut nonce) = submit_accepted(
        &client,
        &sign,
        &sk,
        nonce,
        set_leverage(owner, &symbol, target_leverage),
    )?;
    print_receipt("设置杠杆", &receipt);

    println!("\n== 6. 查询杠杆（设置后，getPosition） ==");
    if wait_for_leverage(&client, &address, &symbol, target_leverage, &poll)? {
        println!("杠杆生效已就绪");
    } else {
        println!("杠杆生效在轮询窗口内未观察到目标状态");
    }
    query_and_print_leverage(
        &client,
        &address,
        &symbol,
        market.max_leverage,
        "设置后",
    )?;

    let buy_price = bump_price(market.mark_price.as_str(), market.price_decimals, MARK_TICKS_ABOVE)
        .ok_or_else(|| format!("无法基于标记价 {} 计算买单价格", market.mark_price))?;

    println!("\n== 7. 提交买单 ==");
    println!(
        "nonce= {nonce} 方向= 买 数量= {ORDER_QTY} 价格= {buy_price} 有效期= 立即成交或取消"
    );
    let (receipt, _) = submit_accepted(
        &client,
        &sign,
        &sk,
        nonce,
        place_order(
            owner,
            &symbol,
            Side::Bid,
            buy_price.as_str(),
            ORDER_QTY,
            TimeInForce::Ioc,
        ),
    )?;
    print_receipt("提交", &receipt);

    println!("\n== 8. 查询持仓（等待成交） ==");
    let size_decimals = market.size_decimals;
    let acct_after = wait_for_account(
        &client,
        &address,
        &poll,
        |acct| has_symbol_position(acct, &symbol, size_decimals),
    )?;
    print_positions("成交后", &acct_after);
    query_and_print_leverage(
        &client,
        &address,
        &symbol,
        market.max_leverage,
        "成交后",
    )?;
    print_open_orders(&client.account_orders(&address)?);

    println!("\n== 9. 平仓 ==");
    nonce = client.account(&address)?.nonce;
    let close_targets = open_position_symbols(&client.account(&address)?, &markets);
    if close_targets.is_empty() {
        println!("无可平持仓");
    } else {
        for sym in &close_targets {
            println!("市价平仓 {sym}（nonce= {nonce}）");
            let (receipt, next) = submit_accepted(
                &client,
                &sign,
                &sk,
                nonce,
                close_position_market(owner, sym),
            )?;
            nonce = next;
            print_receipt(&format!("平仓[{sym}]"), &receipt);
        }
    }

    println!("\n== 10. 查询持仓（等待平仓） ==");
    let acct_closed = wait_for_account(
        &client,
        &address,
        &poll,
        |acct| !has_symbol_position(acct, &symbol, size_decimals),
    )?;
    print_positions("平仓后", &acct_closed);
    print_open_orders(&client.account_orders(&address)?);

    println!("\n== 11. 收尾清理（幂等） ==");
    run_flatten(
        &client,
        &sk,
        owner,
        &address,
        &markets,
        "收尾清理",
        &sign,
        &poll,
    )?;

    println!("\n流程结束。");
    Ok(())
}
