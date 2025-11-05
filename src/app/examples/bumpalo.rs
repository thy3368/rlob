use bumpalo::Bump;

// 定义数据结构
struct Node<'a> {
    value: i32,
    children: Vec<&'a Node<'a>>, // 子节点引用
}

fn main() {
    // 创建内存池
    let arena = Bump::new();

    // 在池中分配对象
    let leaf1 = arena.alloc(Node {
        value: 1,
        children: vec![],
    });

    let leaf2 = arena.alloc(Node {
        value: 2,
        children: vec![],
    });

    let root = arena.alloc(Node {
        value: 0,
        children: vec![leaf1, leaf2],
    });

    // 使用对象...
    println!("Root value: {}", root.value);

    // arena析构时自动释放所有内存
}
