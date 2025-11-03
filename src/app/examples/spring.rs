use anyhow::Context;
use spring::{auto_config, App};
use spring_sqlx::{
    sqlx::{self, Row}, ConnectPool,
    SqlxPlugin,
};
use spring_web::{
    axum::response::IntoResponse, error::Result,
    extractor::{Component, Path},
    WebConfigurator,
    WebPlugin
    ,
};
use spring_web::{get, route};

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(SqlxPlugin)
        .add_plugin(WebPlugin)
        .run()
        .await
}

#[get("/")]
async fn hello_world() -> impl IntoResponse {
    "hello world"
}

#[route("/hello/{name}", method = "GET", method = "POST")]
async fn hello(Path(name): Path<String>) -> impl IntoResponse {
    format!("hello {name}")
}

#[get("/version")]
async fn sqlx_request_handler(Component(pool): Component<ConnectPool>) -> Result<String> {
    let version = sqlx::query("select version() as version")
        .fetch_one(&pool)
        .await
        .context("sqlx query failed")?
        .get("version");
    Ok(version)
}
