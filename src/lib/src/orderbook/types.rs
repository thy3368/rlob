/// 订单簿类型和数据结构
///
/// 本模块提供高性能订单簿基础类型，
/// 针对低时延交易系统进行优化。

use std::fmt;

/// 交易员标识符（8字节固定长度）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(align(8))]
pub struct TraderId([u8; 8]);

impl TraderId {
    /// 从字节数组创建交易员ID
    #[inline]
    pub fn new(bytes: [u8; 8]) -> Self {
        Self(bytes)
    }

    /// 从字符串创建交易员ID（最多8字节）
    #[inline]
    pub fn from_str(s: &str) -> Self {
        let mut bytes = [0u8; 8];
        let len = s.len().min(8);
        bytes[..len].copy_from_slice(&s.as_bytes()[..len]);
        Self(bytes)
    }

    /// 获取底层字节数组的引用
    #[inline]
    pub fn as_bytes(&self) -> &[u8; 8] {
        &self.0
    }
}

impl fmt::Display for TraderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = std::str::from_utf8(&self.0)
            .unwrap_or("INVALID")
            .trim_end_matches('\0');
        write!(f, "{}", s)
    }
}

/// 订单方向（买入或卖出）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Side {
    Buy = b'B',   // 买入
    Sell = b'S',  // 卖出
}

impl Side {
    /// 获取相反方向
    #[inline]
    pub fn opposite(&self) -> Side {
        match self {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        }
    }
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Side::Buy => write!(f, "BUY"),
            Side::Sell => write!(f, "SELL"),
        }
    }
}

/// 订单标识符
pub type OrderId = u64;

/// 价格（以分为单位，避免浮点运算）
pub type Price = u32;

/// 数量/规模
pub type Quantity = u32;

/// 交易执行记录
#[derive(Debug, Clone, Copy)]
pub struct Trade {
    pub buyer: TraderId,      // 买方
    pub seller: TraderId,     // 卖方
    pub price: Price,         // 成交价格
    pub quantity: Quantity,   // 成交数量
}

impl Trade {
    /// 创建新的交易记录
    #[inline]
    pub fn new(buyer: TraderId, seller: TraderId, price: Price, quantity: Quantity) -> Self {
        Self {
            buyer,
            seller,
            price,
            quantity,
        }
    }
}

impl fmt::Display for Trade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TRADE: {} <- {} @ {} x {}",
            self.buyer, self.seller, self.price, self.quantity
        )
    }
}

/// 订单簿条目（64字节缓存行对齐以提升性能）
#[derive(Debug, Clone, Copy)]
#[repr(align(64))]
pub struct OrderEntry {
    pub order_id: OrderId,           // 订单ID
    pub trader: TraderId,            // 交易员ID
    pub quantity: Quantity,          // 数量
    pub next_idx: Option<usize>,     // 链表中下一个订单的索引
}

impl OrderEntry {
    /// 创建新的订单条目
    #[inline]
    pub fn new(order_id: OrderId, trader: TraderId, quantity: Quantity) -> Self {
        Self {
            order_id,
            trader,
            quantity,
            next_idx: None,
        }
    }

    /// 检查订单是否仍然有效（数量>0）
    #[inline]
    pub fn is_active(&self) -> bool {
        self.quantity > 0
    }

    /// 取消订单（通过将数量置零，单次内存写入，速度快）
    #[inline]
    pub fn cancel(&mut self) {
        self.quantity = 0;
    }
}

/// 订单簿中的价格点（链表头）
#[derive(Debug, Clone, Copy)]
pub struct PricePoint {
    pub first_order_idx: Option<usize>,  // 该价格的第一个订单索引
    pub last_order_idx: Option<usize>,   // 该价格的最后一个订单索引
}

impl Default for PricePoint {
    fn default() -> Self {
        Self {
            first_order_idx: None,
            last_order_idx: None,
        }
    }
}

impl PricePoint {
    /// 检查该价格是否没有订单
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.first_order_idx.is_none()
    }

    /// 在链表尾部添加订单
    #[inline]
    pub fn push_back(&mut self, idx: usize) {
        match self.last_order_idx {
            None => {
                // 空链表
                self.first_order_idx = Some(idx);
                self.last_order_idx = Some(idx);
            }
            Some(_last_idx) => {
                // 追加到末尾（调用方需要更新条目的next_idx）
                self.last_order_idx = Some(idx);
            }
        }
    }
}
