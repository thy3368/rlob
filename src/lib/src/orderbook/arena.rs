/// 订单簿条目的内存池分配器
///
/// 提供快速、缓存友好的分配，无堆开销。
/// 订单从预分配池中使用bump-pointer分配。

use super::types::OrderEntry;

/// 固定大小的订单条目内存池
pub struct OrderArena {
    entries: Vec<OrderEntry>,  // 订单条目数组
    next_free: usize,          // 下一个空闲位置
}

impl OrderArena {
    /// 创建指定容量的新内存池
    #[inline]
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            next_free: 0,
        }
    }

    /// 分配新的订单条目，返回其索引
    #[inline]
    pub fn allocate(&mut self, entry: OrderEntry) -> Option<usize> {
        if self.next_free >= self.entries.capacity() {
            return None; // 内存池已满
        }

        let idx = self.next_free;
        self.entries.push(entry);
        self.next_free += 1;
        Some(idx)
    }

    /// 通过索引获取条目的引用
    #[inline]
    pub fn get(&self, idx: usize) -> Option<&OrderEntry> {
        self.entries.get(idx)
    }

    /// 通过索引获取条目的可变引用
    #[inline]
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut OrderEntry> {
        self.entries.get_mut(idx)
    }

    /// 获取已分配条目的数量
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 检查内存池是否为空
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 获取内存池容量
    #[inline]
    pub fn capacity(&self) -> usize {
        self.entries.capacity()
    }

    /// 获取剩余容量
    #[inline]
    pub fn remaining_capacity(&self) -> usize {
        self.entries.capacity() - self.entries.len()
    }

    /// 清空内存池（用于重置）
    #[inline]
    pub fn clear(&mut self) {
        self.entries.clear();
        self.next_free = 0;
    }

    /// 预留额外容量
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.entries.reserve(additional);
    }
}

impl Default for OrderArena {
    fn default() -> Self {
        Self::new(1_000_000) // 默认容量：100万订单
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orderbook::types::TraderId;

    #[test]
    fn test_arena_allocation() {
        let mut arena = OrderArena::new(10);

        let entry = OrderEntry::new(1, TraderId::from_str("TRADER1"), 100);
        let idx = arena.allocate(entry).unwrap();

        assert_eq!(idx, 0);
        assert_eq!(arena.len(), 1);
        assert_eq!(arena.get(idx).unwrap().quantity, 100);
    }

    #[test]
    fn test_arena_full() {
        let mut arena = OrderArena::new(2);

        let entry1 = OrderEntry::new(1, TraderId::from_str("T1"), 100);
        let entry2 = OrderEntry::new(2, TraderId::from_str("T2"), 200);
        let entry3 = OrderEntry::new(3, TraderId::from_str("T3"), 300);

        assert!(arena.allocate(entry1).is_some());
        assert!(arena.allocate(entry2).is_some());
        assert!(arena.allocate(entry3).is_none()); // Full
    }

    #[test]
    fn test_arena_clear() {
        let mut arena = OrderArena::new(10);

        arena.allocate(OrderEntry::new(1, TraderId::from_str("T1"), 100));
        assert_eq!(arena.len(), 1);

        arena.clear();
        assert_eq!(arena.len(), 0);
        assert_eq!(arena.remaining_capacity(), 10);
    }
}
