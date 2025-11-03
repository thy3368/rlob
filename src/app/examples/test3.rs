use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    #[serde(rename = "userID")] // 序列化后字段名为 "userID"
    id: u64,
    name: String,
    #[serde(skip_serializing)] // 序列化时跳过此字段（如密码哈希）
    password_hash: String,
    #[serde(skip_deserializing)] // 反序列化时跳过（如服务器生成的时间戳）
    created_at: String,
}

fn main() {
    let user = User {
        id: 101,
        name: "Charlie".to_string(),
        password_hash: "hashed_value".to_string(),
        created_at: "2023-10-01".to_string(),
    };

    let json = serde_json::to_string_pretty(&user).unwrap();
    println!("{}", json);
    // 输出：
    // {
    //   "userID": 101,
    //   "name": "Charlie"
    // }
}