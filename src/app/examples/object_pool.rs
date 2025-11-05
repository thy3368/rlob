use std::marker::PhantomData;

// 世代句柄：安全地引用池中对象
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Handle<T> {
    index: u32,                  // 对象在records中的位置
    generation: u32,             // 世代号，用于验证句柄有效性
    type_marker: PhantomData<T>, // 类型安全标记
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Handle<T> {}

impl<T> Handle<T> {
    fn new(index: u32, generation: u32) -> Self {
        Self {
            index,
            generation,
            type_marker: PhantomData,
        }
    }
}

// 内存池记录
struct PoolRecord<T> {
    generation: u32, // 世代号
    data: Option<T>, // 存储的实际数据
}

// 主内存池结构
pub struct Pool<T> {
    records: Vec<PoolRecord<T>>, // 存储对象的连续内存块
    free_stack: Vec<u32>,        // 空闲位置索引栈
}

impl<T> Pool<T> {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            free_stack: Vec::new(),
        }
    }

    // 创建新对象
    pub fn spawn(&mut self, data: T) -> Handle<T> {
        if let Some(free_index) = self.free_stack.pop() {
            // 复用空闲位置
            let record = &mut self.records[free_index as usize];
            record.generation += 1; // 递增世代号使旧句柄失效
            record.data = Some(data);

            Handle::new(free_index, record.generation)
        } else {
            // 扩展records数组
            let index = self.records.len() as u32;
            self.records.push(PoolRecord {
                generation: 1,
                data: Some(data),
            });

            Handle::new(index, 1)
        }
    }

    // 销毁对象
    pub fn free(&mut self, handle: Handle<T>) -> Option<T> {
        if !self.is_handle_valid(&handle) {
            return None;
        }

        let record = &mut self.records[handle.index as usize];
        self.free_stack.push(handle.index); // 加入空闲栈等待复用
        record.generation += 1; // 使旧句柄失效
        record.data.take() // 取出数据
    }

    // 安全访问对象
    pub fn try_borrow(&self, handle: Handle<T>) -> Option<&T> {
        if self.is_handle_valid(&handle) {
            self.records[handle.index as usize].data.as_ref()
        } else {
            None
        }
    }

    pub fn try_borrow_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        if self.is_handle_valid(&handle) {
            self.records[handle.index as usize].data.as_mut()
        } else {
            None
        }
    }

    // 验证句柄有效性
    fn is_handle_valid(&self, handle: &Handle<T>) -> bool {
        if let Some(record) = self.records.get(handle.index as usize) {
            record.generation == handle.generation && record.data.is_some()
        } else {
            false
        }
    }
}

// 定义游戏对象
struct GameObject {
    name: String,
    position: (f32, f32),
}

fn main() {
    let mut pool = Pool::new();

    // 创建对象
    let player_handle = pool.spawn(GameObject {
        name: "Player".to_string(),
        position: (0.0, 0.0),
    });

    let enemy_handle = pool.spawn(GameObject {
        name: "Enemy".to_string(),
        position: (10.0, 5.0),
    });

    // 安全访问
    if let Some(player) = pool.try_borrow_mut(player_handle) {
        player.position.0 += 1.0;
    }

    // 销毁对象
    if let Some(_enemy) = pool.free(enemy_handle) {
        println!("Enemy destroyed");
    }

    // 尝试使用已失效的句柄
    assert!(pool.try_borrow(enemy_handle).is_none());
}
