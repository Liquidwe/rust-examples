use std::io;
use ratatui::DefaultTerminal;

mod app;
mod ui;

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::init();  // 初始化日志

    let mut terminal = ratatui::init();
    let mut app = app::App::new().await;  // 等待异步数据加载完成
    let app_result = app.run(&mut terminal);
    ratatui::restore();
    app_result
}
