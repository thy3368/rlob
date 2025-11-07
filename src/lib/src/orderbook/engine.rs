/// 高性能订单匹配引擎
///
/// 实现价格-时间优先的限价订单簿，具有O(1)订单放置
/// 和使用线性价格点数组的高效匹配。

use super::arena::OrderArena;
use super::types::{OrderEntry, OrderId, Price, PricePoint, Quantity, Side, Trade, TraderId};
use std::collections::HashMap;

/// 最大价格级别（以分为单位）- 根据预期价格范围调整
const MAX_PRICE: usize = 10_000_000; // 最高价格 $100,000

/// 订单簿匹配引擎
pub struct OrderBook {
    /// 买单价格点（出价）
    bids: Vec<PricePoint>,
    /// 卖单价格点（要价）
    asks: Vec<PricePoint>,
    /// 订单条目的内存池
    arena: OrderArena,
    /// 订单ID到内存池索引的映射（用于快速取消）
    order_index: HashMap<OrderId, usize>,
    /// 最佳买价（最高买入价）
    bid_max: Option<Price>,
    /// 最佳卖价（最低卖出价）
    ask_min: Option<Price>,
    /// 下一个订单ID
    next_order_id: OrderId,
    /// 交易执行历史
    trades: Vec<Trade>,
}

impl OrderBook {
    /// 创建新的订单簿
    pub fn new() -> Self {
        Self::with_capacity(MAX_PRICE, 1_000_000)
    }

    /// 创建指定容量的新订单簿
    pub fn with_capacity(max_price: usize, max_orders: usize) -> Self {
        Self {
            bids: vec![PricePoint::default(); max_price],
            asks: vec![PricePoint::default(); max_price],
            arena: OrderArena::new(max_orders),
            order_index: HashMap::with_capacity(max_orders),
            bid_max: None,
            ask_min: None,
            next_order_id: 1,
            trades: Vec::new(),
        }
    }

    /// 获取下一个订单ID
    #[inline]
    pub fn next_order_id(&self) -> OrderId {
        self.next_order_id
    }

    /// 设置下一个订单ID（用于状态恢复）
    #[inline]
    pub fn set_next_order_id(&mut self, id: OrderId) {
        self.next_order_id = id;
    }

    /// 获取最佳买价
    #[inline]
    pub fn best_bid(&self) -> Option<Price> {
        self.bid_max
    }

    /// 获取最佳卖价
    #[inline]
    pub fn best_ask(&self) -> Option<Price> {
        self.ask_min
    }

    /// 获取买卖价差（卖价 - 买价）
    #[inline]
    pub fn spread(&self) -> Option<Price> {
        match (self.ask_min, self.bid_max) {
            (Some(ask), Some(bid)) if ask > bid => Some(ask - bid),
            _ => None,
        }
    }

    /// 获取中间价
    #[inline]
    pub fn mid_price(&self) -> Option<Price> {
        match (self.ask_min, self.bid_max) {
            (Some(ask), Some(bid)) => Some((ask + bid) / 2),
            _ => None,
        }
    }

    /// 提交新的限价订单
    ///
    /// 返回 (订单ID, 成交列表)
    pub fn limit_order(
        &mut self,
        trader: TraderId,
        side: Side,
        price: Price,
        quantity: Quantity,
    ) -> (OrderId, Vec<Trade>) {
        let order_id = self.next_order_id;
        self.next_order_id += 1;

        let mut remaining = quantity;  // 剩余未成交数量
        let mut trades = Vec::new();   // 成交记录

        // 尝试与对手方匹配
        match side {
            Side::Buy => {
                // 从最佳（最低）卖价开始匹配卖单
                if let Some(mut ask_price) = self.ask_min {
                    while remaining > 0 && ask_price <= price {
                        let fills = self.match_at_price(
                            order_id,
                            trader,
                            side,
                            ask_price,
                            &mut remaining,
                        );
                        trades.extend(fills);

                        // 移动到下一个卖价级别
                        ask_price = self.find_next_ask(ask_price).unwrap_or(price + 1);
                    }
                    // 更新最佳卖价
                    self.ask_min = self.find_next_ask(0);
                }

                // 如果未完全成交，将剩余部分添加到买单侧
                if remaining > 0 {
                    self.add_order(order_id, trader, side, price, remaining);
                    // 更新最佳买价
                    if self.bid_max.map_or(true, |max| price > max) {
                        self.bid_max = Some(price);
                    }
                }
            }
            Side::Sell => {
                // 从最佳（最高）买价开始匹配买单
                if let Some(mut bid_price) = self.bid_max {
                    while remaining > 0 && bid_price >= price {
                        let fills = self.match_at_price(
                            order_id,
                            trader,
                            side,
                            bid_price,
                            &mut remaining,
                        );
                        trades.extend(fills);

                        // 移动到下一个买价级别
                        bid_price = self.find_prev_bid(bid_price).unwrap_or(0);
                    }
                    // 更新最佳买价
                    self.bid_max = self.find_prev_bid(u32::MAX);
                }

                // 如果未完全成交，将剩余部分添加到卖单侧
                if remaining > 0 {
                    self.add_order(order_id, trader, side, price, remaining);
                    // 更新最佳卖价
                    if self.ask_min.map_or(true, |min| price < min) {
                        self.ask_min = Some(price);
                    }
                }
            }
        }

        // 存储交易记录
        self.trades.extend(&trades);

        (order_id, trades)
    }

    /// 在特定价格级别匹配订单
    fn match_at_price(
        &mut self,
        _order_id: OrderId,
        trader: TraderId,
        side: Side,
        price: Price,
        remaining: &mut Quantity,
    ) -> Vec<Trade> {
        let mut trades = Vec::new();
        let price_idx = price as usize;

        let price_point = match side {
            Side::Buy => &mut self.asks[price_idx],
            Side::Sell => &mut self.bids[price_idx],
        };

        let mut current_idx = price_point.first_order_idx;
        let mut first_active_idx = None;

        while *remaining > 0 && current_idx.is_some() {
            let idx = current_idx.unwrap();
            let entry = self.arena.get_mut(idx).unwrap();

            if entry.is_active() {
                // Track first active order for price point update
                if first_active_idx.is_none() {
                    first_active_idx = Some(idx);
                }

                let fill_qty = (*remaining).min(entry.quantity);

                // Create trade record
                let trade = match side {
                    Side::Buy => Trade::new(trader, entry.trader, price, fill_qty),
                    Side::Sell => Trade::new(entry.trader, trader, price, fill_qty),
                };
                trades.push(trade);

                // Update quantities
                *remaining -= fill_qty;
                entry.quantity -= fill_qty;

                // If order fully filled, mark as inactive
                if entry.quantity == 0 {
                    self.order_index.remove(&entry.order_id);
                    // Update first active if this was it
                    if first_active_idx == Some(idx) {
                        first_active_idx = None;
                    }
                }
            }

            current_idx = self.arena.get(idx).unwrap().next_idx;

            // Update first_active_idx if we haven't found one yet
            if first_active_idx.is_none() && current_idx.is_some() {
                let next_entry = self.arena.get(current_idx.unwrap()).unwrap();
                if next_entry.is_active() {
                    first_active_idx = current_idx;
                }
            }
        }

        // Update price point to reflect first active order
        if first_active_idx.is_none() && current_idx.is_none() {
            // All orders consumed, clear price level
            price_point.first_order_idx = None;
            price_point.last_order_idx = None;
        } else if first_active_idx.is_some() {
            // Update to first active order
            price_point.first_order_idx = first_active_idx;
        }

        trades
    }

    /// 将新订单添加到订单簿
    fn add_order(
        &mut self,
        order_id: OrderId,
        trader: TraderId,
        side: Side,
        price: Price,
        quantity: Quantity,
    ) {
        let entry = OrderEntry::new(order_id, trader, quantity);
        let idx = self
            .arena
            .allocate(entry)
            .expect("Order arena capacity exceeded");

        self.order_index.insert(order_id, idx);

        let price_idx = price as usize;
        let price_point = match side {
            Side::Buy => &mut self.bids[price_idx],
            Side::Sell => &mut self.asks[price_idx],
        };

        // Link to existing orders at this price level
        if let Some(last_idx) = price_point.last_order_idx {
            self.arena.get_mut(last_idx).unwrap().next_idx = Some(idx);
        }

        price_point.push_back(idx);
    }

    /// 取消订单
    pub fn cancel_order(&mut self, order_id: OrderId) -> bool {
        if let Some(&idx) = self.order_index.get(&order_id) {
            if let Some(entry) = self.arena.get_mut(idx) {
                entry.cancel();
                self.order_index.remove(&order_id);
                return true;
            }
        }
        false
    }

    /// 查找下一个非空的卖价级别
    fn find_next_ask(&self, start_price: Price) -> Option<Price> {
        for price in (start_price as usize)..self.asks.len() {
            if !self.asks[price].is_empty() {
                return Some(price as Price);
            }
        }
        None
    }

    /// 查找上一个非空的买价级别
    fn find_prev_bid(&self, start_price: Price) -> Option<Price> {
        let max_price = start_price.min((self.bids.len() - 1) as u32);
        for price in (0..=max_price as usize).rev() {
            if !self.bids[price].is_empty() {
                return Some(price as Price);
            }
        }
        None
    }

    /// 获取交易历史
    pub fn trades(&self) -> &[Trade] {
        &self.trades
    }

    /// 清空交易历史
    pub fn clear_trades(&mut self) {
        self.trades.clear();
    }

    /// 获取订单簿状态快照
    pub fn snapshot(&self) -> OrderBookSnapshot {
        OrderBookSnapshot {
            next_order_id: self.next_order_id,
            bid_max: self.bid_max,
            ask_min: self.ask_min,
            active_orders: self.order_index.len(),
            total_trades: self.trades.len(),
        }
    }
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}

/// 订单簿状态快照
#[derive(Debug, Clone, Copy)]
pub struct OrderBookSnapshot {
    pub next_order_id: OrderId,       // 下一个订单ID
    pub bid_max: Option<Price>,       // 最佳买价
    pub ask_min: Option<Price>,       // 最佳卖价
    pub active_orders: usize,         // 活跃订单数
    pub total_trades: usize,          // 总交易数
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_buy_order() {
        let mut book = OrderBook::new();
        let trader = TraderId::from_str("TRADER1");

        let (order_id, trades) = book.limit_order(trader, Side::Buy, 10000, 100);

        assert_eq!(order_id, 1);
        assert_eq!(trades.len(), 0); // No matches
        assert_eq!(book.best_bid(), Some(10000));
        assert_eq!(book.best_ask(), None);
    }

    #[test]
    fn test_simple_match() {
        let mut book = OrderBook::new();
        let buyer = TraderId::from_str("BUYER");
        let seller = TraderId::from_str("SELLER");

        // Place sell order
        book.limit_order(seller, Side::Sell, 10000, 100);

        // Place matching buy order
        let (_order_id, trades) = book.limit_order(buyer, Side::Buy, 10000, 100);

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, 100);
        assert_eq!(trades[0].price, 10000);
    }

    #[test]
    fn test_partial_fill() {
        let mut book = OrderBook::new();
        let buyer = TraderId::from_str("BUYER");
        let seller = TraderId::from_str("SELLER");

        // Place large sell order
        book.limit_order(seller, Side::Sell, 10000, 200);

        // Place smaller buy order
        let (_order_id, trades) = book.limit_order(buyer, Side::Buy, 10000, 50);

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, 50);
        assert_eq!(book.best_ask(), Some(10000)); // Still has remaining
    }

    #[test]
    fn test_price_improvement() {
        let mut book = OrderBook::new();
        let buyer = TraderId::from_str("BUYER");
        let seller = TraderId::from_str("SELLER");

        // Place sell order at 10000
        book.limit_order(seller, Side::Sell, 10000, 100);

        // Place buy order at higher price (11000)
        let (_order_id, trades) = book.limit_order(buyer, Side::Buy, 11000, 100);

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].price, 10000); // Matched at seller's price
    }

    #[test]
    fn test_cancel_order() {
        let mut book = OrderBook::new();
        let trader = TraderId::from_str("TRADER1");

        let (order_id, _) = book.limit_order(trader, Side::Buy, 10000, 100);
        assert!(book.cancel_order(order_id));
        assert!(!book.cancel_order(order_id)); // Already cancelled
    }

    #[test]
    fn test_spread() {
        let mut book = OrderBook::new();

        book.limit_order(TraderId::from_str("B"), Side::Buy, 9900, 100);
        book.limit_order(TraderId::from_str("S"), Side::Sell, 10100, 100);

        assert_eq!(book.best_bid(), Some(9900));
        assert_eq!(book.best_ask(), Some(10100));
        assert_eq!(book.spread(), Some(200));
        assert_eq!(book.mid_price(), Some(10000));
    }
}
