use std::{io, collections::HashMap, time::Duration, cell::RefCell};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::DefaultTerminal;
use serde::{Deserialize, Serialize};
use reqwest;
use serde_json::json;
use reqwest::header::{HeaderMap, HeaderValue};
use tokio::sync::mpsc;
use ratatui::style::{Style, Color,palette::tailwind, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::buffer::Buffer;
use ratatui::widgets::{Gauge, Widget,block::Title,Block,Borders, Padding, Paragraph};

use crate::ui;
use crate::docker::DockerManager;

#[derive(Debug)]
pub struct App {
    pub chains: Vec<Chain>,            // 保存所有链信息
    pub selected_chain_index: usize,    // 记录当前选中的链索引
    pub selected_table_index: Option<usize>,  // 新增：当前选中的表索引
    pub show_tables: bool,                    // 新增：是否显示表列表
    pub scroll_offset: usize,    // 新增：跟踪滚动位置
    pub exit: bool,
    pub current_tab: usize,  // Add this line
    pub example_data: Option<ExampleData>,  // Add this line
    pub sql_input: String,
    pub show_sql_window: bool,
    pub sql_cursor_position: usize,
    pub sql_result: Option<String>,  // To store the mock response
    pub saved_sql: Option<String>,  // Add this field to store saved SQL
    pub sql_executing: bool,
    pub sql_error: Option<String>,
    pub sql_columns: Vec<Column>,
    pub sql_data: Vec<Vec<serde_json::Value>>,
    sql_sender: Option<mpsc::Sender<Result<serde_json::Value, String>>>,
    sql_receiver: Option<mpsc::Receiver<Result<serde_json::Value, String>>>,
    pub sql_timer: u64,  // Add this field for the timer
    pub docker_manager: DockerManager,
    pub docker_status: Option<String>,
    pub docker_setup_in_progress: bool,
    pub docker_setup_timer: u64,  // Add this new field
    pub setup_progress: f64,  // Add this field
    pub setup_state: SetupState,  // Add this field
    state: AppState,
    progress_columns: u16,
    pub progress1: f64,
    pub progress_lines: RefCell<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SetupState {
    NotStarted,
    InProgress,
    Complete,
    Failed(String),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum AppState {
    #[default]
    Running,
    Started,
    Quitting,
}

impl Clone for App {
    fn clone(&self) -> Self {
        Self {
            chains: self.chains.clone(),
            selected_chain_index: self.selected_chain_index,
            selected_table_index: self.selected_table_index,
            show_tables: self.show_tables,
            scroll_offset: self.scroll_offset,
            exit: self.exit,
            current_tab: self.current_tab,
            example_data: self.example_data.clone(),
            sql_input: self.sql_input.clone(),
            show_sql_window: self.show_sql_window,
            sql_cursor_position: self.sql_cursor_position,
            sql_result: self.sql_result.clone(),
            saved_sql: self.saved_sql.clone(),
            sql_executing: self.sql_executing,
            sql_error: self.sql_error.clone(),
            sql_columns: self.sql_columns.clone(),
            sql_data: self.sql_data.clone(),
            sql_sender: self.sql_sender.clone(),
            sql_receiver: None,  // Don't clone the receiver
            sql_timer: self.sql_timer,
            docker_manager: self.docker_manager.clone(),
            docker_status: self.docker_status.clone(),
            docker_setup_in_progress: self.docker_setup_in_progress,
            docker_setup_timer: self.docker_setup_timer,  // Initialize the timer
            setup_progress: self.setup_progress,
            setup_state: self.setup_state.clone(),
            state: self.state,
            progress_columns: self.progress_columns,
            progress1: self.progress1,
            progress_lines: RefCell::new(Vec::new()),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ExampleData {
    pub columns: Vec<Column>,
    pub data: Vec<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub type_: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Chain {
    pub name: String,
    pub status: String,
    pub lastUpdate: String,
    pub time_ago: String,  // 新增字段存储计算好的时间差
    pub dataDictionary: HashMap<String, Vec<DataDictionaryItem>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DataDictionaryItem {
    pub name: String,
    pub dataType: String,
    pub description: String,
}

impl App {
    pub async fn new() -> Self {
        let (sql_sender, sql_receiver) = mpsc::channel(32);
        
        let chains = match App::fetch_chains().await {
            Ok(data) => data,
            Err(_) => Vec::new()
        };

        App {
            chains,
            selected_chain_index: 0,
            selected_table_index: None,
            show_tables: false,
            scroll_offset: 0,     // 初始化滚动位置
            exit: false,
            current_tab: 0,  // Add this line
            example_data: None,  // Changed: Initialize as None
            sql_input: String::new(),
            show_sql_window: false,
            sql_cursor_position: 0,
            sql_result: None,
            saved_sql: None,
            sql_executing: false,
            sql_error: None,
            sql_columns: Vec::new(),
            sql_data: Vec::new(),
            sql_sender: Some(sql_sender),
            sql_receiver: Some(sql_receiver),
            sql_timer: 0,  // Initialize timer
            docker_manager: DockerManager::new(),
            docker_status: None,
            docker_setup_in_progress: false,
            docker_setup_timer: 0,  // Initialize the timer
            setup_progress: 0.0,
            setup_state: SetupState::NotStarted,
            state: AppState::default(),
            progress_columns: 0,
            progress1: 0.0,
            progress_lines: RefCell::new(Vec::new()),
        }
    }

    async fn fetch_chains() -> Result<Vec<Chain>, reqwest::Error> {
        let url = "https://api.chainbase.com/api/v1/metadata/network_chains";

        match reqwest::get(url).await?.json::<Response>().await {
            Ok(response) => {
                Ok(response.graphData.into_iter()
                    .map(|graph_data| {
                        let mut tables = HashMap::new();
                        tables.insert("blocks".to_string(), graph_data.chain.dataDictionary.blocks);
                        tables.insert("transactions".to_string(), graph_data.chain.dataDictionary.transactions);
                        tables.insert("transactionLogs".to_string(), graph_data.chain.dataDictionary.transactionLogs);
                        
                        let time_ago = Self::calculate_time_diff(&graph_data.chain.lastUpdate);
                        
                        Chain {
                            name: graph_data.chain.name,
                            status: graph_data.chain.status,
                            lastUpdate: graph_data.chain.lastUpdate,
                            time_ago,
                            dataDictionary: tables,
                        }
                    })
                    .collect())
            }
            Err(e) => {
                println!("Failed to parse JSON response: {:?}", e);
                Err(e)
            }
        }
    }

    fn calculate_time_diff(time_str: &str) -> String {
        if let Ok(time) = chrono::DateTime::parse_from_rfc3339(time_str) {
            let now = chrono::Utc::now();
            let duration = now.signed_duration_since(time);
            
            if duration.num_hours() > 24 {
                format!("{} days", duration.num_days())
            } else if duration.num_hours() > 0 {
                format!("{} hrs", duration.num_hours())
            } else {
                format!("{} min", duration.num_minutes())
            }
        } else {
            "unknown".to_string()
        }
    }

    fn mock_blocks_data() -> ExampleData {
        ExampleData {
            columns: vec![
                Column { name: "block_number".to_string(), type_: "bigint".to_string() },
                Column { name: "hash".to_string(), type_: "varchar(66)".to_string() },
                Column { name: "parent_hash".to_string(), type_: "varchar(66)".to_string() },
                Column { name: "nonce".to_string(), type_: "varchar(78)".to_string() },
                Column { name: "sha3_uncles".to_string(), type_: "varchar(66)".to_string() },
                Column { name: "logs_bloom".to_string(), type_: "varchar".to_string() },
                Column { name: "transactions_root".to_string(), type_: "varchar(66)".to_string() },
                Column { name: "state_root".to_string(), type_: "varchar(66)".to_string() },
                Column { name: "receipts_root".to_string(), type_: "varchar(66)".to_string() },
                Column { name: "miner".to_string(), type_: "varchar(42)".to_string() },
                Column { name: "difficulty".to_string(), type_: "varchar(78)".to_string() },
                Column { name: "total_difficulty".to_string(), type_: "varchar(78)".to_string() },
                Column { name: "size".to_string(), type_: "bigint".to_string() },
                Column { name: "extra_data".to_string(), type_: "varchar".to_string() },
                Column { name: "gas_limit".to_string(), type_: "varchar(78)".to_string() },
                Column { name: "gas_used".to_string(), type_: "varchar(78)".to_string() },
                Column { name: "block_timestamp".to_string(), type_: "timestamp".to_string() },
                Column { name: "transaction_count".to_string(), type_: "bigint".to_string() },
                Column { name: "base_fee_per_gas".to_string(), type_: "varchar(78)".to_string() },
                Column { name: "withdrawals_root".to_string(), type_: "varchar(66)".to_string() },
                Column { name: "__pk".to_string(), type_: "integer".to_string() },
            ],
            data: vec![
                vec![
                    json!(0),
                    json!("0x81005434635456a16f74ff7023fbe0bf423abbc8a8deb093ffff455c0ad3b741"),
                    json!("0x0000000000000000000000000000000000000000000000000000000000000000"),
                    json!("0x0000000000000000"),
                    json!("0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347"),
                    json!("0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
                    json!("0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421"),
                    json!("0x3f86b09b43e3e49a41fc20a07579b79eba044253367817d5c241d23c0e2bc5c9"),
                    json!("0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421"),
                    json!("0x0000000000000000000000000000000000000000"),
                    json!("0"),
                    json!("0"),
                    json!(505),
                    json!("0x"),
                    json!("0"),
                    json!("0"),
                    json!("2023-03-24 10:19:23.000"),
                    json!(0),
                    json!(null),
                    json!(null),
                    json!(0)
                ],
            ],
        }
    }

    fn mock_transaction_logs_data() -> ExampleData {
        ExampleData {
            columns: vec![
                Column { name: "block_number".to_string(), type_: "bigint".to_string() },
                Column { name: "block_timestamp".to_string(), type_: "timestamp".to_string() },
                Column { name: "transaction_hash".to_string(), type_: "varchar".to_string() },
                Column { name: "transaction_index".to_string(), type_: "integer".to_string() },
                Column { name: "log_index".to_string(), type_: "integer".to_string() },
                Column { name: "address".to_string(), type_: "varchar".to_string() },
                Column { name: "data".to_string(), type_: "varbinary".to_string() },
                Column { name: "topic0".to_string(), type_: "varchar".to_string() },
                Column { name: "topic1".to_string(), type_: "varchar".to_string() },
                Column { name: "topic2".to_string(), type_: "varchar".to_string() },
                Column { name: "topic3".to_string(), type_: "varchar".to_string() },
                Column { name: "pk".to_string(), type_: "integer".to_string() },
            ],
            data: vec![
                vec![
                    json!(1),
                    json!("2023-03-24 17:30:15.000"),
                    json!("0x1b1cc77d663d9176b791e94124eecffe49d1c69837ee6e9ed09356f2c70a065d"),
                    json!(0),
                    json!(0),
                    json!("0x2a3dd3eb832af982ec71669e178424b10dca2ede"),
                    json!("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAHboRMQAGZLiEobojhGQVmJIlLToAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABY0V4XYoAAA=="),
                    json!("0x25308c93ceeed162da955b3f7ce3e3f93606579e40fb92029faa9efe27545983"),
                    json!(""),
                    json!(""),
                    json!(""),
                    json!(0)
                ],
            ],
        }
    }

    fn mock_transactions_data() -> ExampleData {
        ExampleData {
            columns: vec![
                Column { name: "hash".to_string(), type_: "varchar(66)".to_string() },
                Column { name: "nonce".to_string(), type_: "varchar(78)".to_string() },
                Column { name: "transaction_index".to_string(), type_: "integer".to_string() },
                Column { name: "from_address".to_string(), type_: "varchar(42)".to_string() },
                Column { name: "to_address".to_string(), type_: "varchar(42)".to_string() },
                Column { name: "value".to_string(), type_: "varchar(78)".to_string() },
                Column { name: "gas".to_string(), type_: "varchar(78)".to_string() },
                Column { name: "gas_price".to_string(), type_: "varchar(78)".to_string() },
                Column { name: "method_id".to_string(), type_: "varchar(10)".to_string() },
                Column { name: "input".to_string(), type_: "varbinary".to_string() },
                Column { name: "block_timestamp".to_string(), type_: "timestamp".to_string() },
                Column { name: "block_number".to_string(), type_: "bigint".to_string() },
                Column { name: "block_hash".to_string(), type_: "varchar(66)".to_string() },
                Column { name: "max_fee_per_gas".to_string(), type_: "varchar(78)".to_string() },
                Column { name: "max_priority_fee_per_gas".to_string(), type_: "varchar(78)".to_string() },
                Column { name: "transaction_type".to_string(), type_: "integer".to_string() },
                Column { name: "receipt_cumulative_gas_used".to_string(), type_: "varchar(78)".to_string() },
                Column { name: "receipt_gas_used".to_string(), type_: "varchar(78)".to_string() },
                Column { name: "receipt_contract_address".to_string(), type_: "varchar(42)".to_string() },
                Column { name: "receipt_status".to_string(), type_: "integer".to_string() },
                Column { name: "receipt_effective_gas_price".to_string(), type_: "varchar(78)".to_string() },
                Column { name: "__pk".to_string(), type_: "integer".to_string() },
            ],
            data: vec![
                vec![
                    json!("0x81005434635456a16f74ff7023fbe0bf423abbc8a8deb093ffff455c0ad3b741"),
                    json!("0x0"),
                    json!(0),
                    json!("0x742d35Cc6634C0532925a3b844Bc454e4438f44e"),
                    json!("0x1234567890123456789012345678901234567890"),
                    json!("1000000000000000000"),
                    json!("21000"),
                    json!("20000000000"),
                    json!("0x"),
                    json!("0x"),
                    json!("2023-10-31 03:52:35.000"),
                    json!(12345678),
                    json!("0x0000000000000000000000000000000000000000000000000000000000000000"),
                    json!("30000000000"),
                    json!("2000000000"),
                    json!(2),
                    json!("21000"),
                    json!("21000"),
                    json!(null),
                    json!(1),
                    json!("20000000000"),
                    json!(0),
                ],
            ],
        }
    }

    // run 方法是用程序的主循环
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        // 当 self.exit 为 false 时持续运行
        while !self.exit {
            // 计算可见区域高度(终端高度减去边框占用的2行)
            let visible_height = terminal.size()?.height as usize - 2;
            
            // 调用 ui::draw 函数绘制用户界面
            terminal.draw(|frame| ui::draw(frame, self))?;
            
            // 等待事件,超时时间为100毫秒
            if event::poll(Duration::from_millis(100))? {
                // 如有事件发生
                if let Event::Key(key_event) = event::read()? {
                    // 只处理按键按下事件
                    if key_event.kind == KeyEventKind::Press {
                        // 调用 handle_key_event 处理按键事件
                        self.handle_key_event(key_event, visible_height);
                    }
                }
            }

            // Check for SQL execution results
            // 检查是否正在执行SQL查询
            if self.sql_executing {
                // 如果有接收器,尝试从通道中接收SQL执行结果
                if let Some(receiver) = &mut self.sql_receiver {
                    // 非阻塞地尝试接收结果
                    match receiver.try_recv() {
                        // 成功接收到结果
                        Ok(result) => match result {
                            // SQL执行成功,处理返回的JSON数据
                            Ok(json) => self.handle_sql_response(json),
                            // SQL执行出错,记录错误信息并停止执行状态
                            Err(error) => {
                                self.sql_error = Some(error);
                                self.sql_executing = false;
                            }
                        },
                        // 通道为空,继续等待
                        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {},
                        // 通道关闭,记录错误并停止执行状态
                        Err(_) => {
                            self.sql_executing = false;
                            self.sql_error = Some("Channel closed".to_string());
                        }
                    }
                }
            }

            // Update timer if Docker setup is in progress
            if self.docker_setup_in_progress {
                self.docker_setup_timer = self.docker_setup_timer.saturating_add(1);
            }

            self.update(terminal.size()?.width);
        }
        Ok(())
    }

    fn update(&mut self, terminal_width: u16) {
        // 只有在应用程序状态为Started时才更新
        if self.state != AppState::Started {
            return;
        }

        // 更新progress1和progress2来展示不同数值类型的进度条效果
        // progress_columns: 当前进度的列数,每次加1,不超过终端宽度
        self.progress_columns = (self.progress_columns + 1).clamp(0, terminal_width);
        // progress1: 使用整数计算百分比(0-100)
        self.progress1 = f64::from(self.progress_columns) * 100.0 / f64::from(terminal_width);

    }

    pub fn render_gauge1(&self, area: Rect, buf: &mut Buffer) {
        let title = title_block("Gauge with ratio and custom label");
        let label = Span::styled(
            format!("{:.1}/100", self.progress1),
            Style::new().italic().bold().fg(CUSTOM_LABEL_COLOR),
        );
        Gauge::default()
            .block(title)
            .gauge_style(GAUGE2_COLOR)
            .ratio(self.progress1 / 100.0)
            .label(label)
            .render(area, buf);
    }

    pub fn update_example_data(&mut self) {
        if let Some(selected_chain) = self.chains.get(self.selected_chain_index) {
            // Check if chain is offline
            if selected_chain.status == "Offline" {
                self.example_data = None;
                return;
            }

            if let Some(table_index) = self.selected_table_index {
                let table_name = selected_chain.dataDictionary
                    .keys()
                    .nth(table_index)
                    .map(|s| s.as_str());

                self.example_data = match table_name {
                    Some("blocks") => Some(Self::mock_blocks_data()),
                    Some("transactions") => Some(Self::mock_transactions_data()),
                    Some("transactionLogs") => Some(Self::mock_transaction_logs_data()),
                    _ => None,
                };
            }
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent, visible_height: usize) {
        if self.show_sql_window {
            match key_event.code {
                KeyCode::Esc => {
                    // Save the SQL when closing the window
                    if !self.sql_input.trim().is_empty() {
                        self.saved_sql = Some(self.sql_input.clone());
                    }
                    // Reset SQL window state
                    self.show_sql_window = false;
                    self.sql_result = None;
                    // Don't clear the selected table index anymore
                }
                KeyCode::Enter => {
                    // Execute SQL when Ctrl+Enter is pressed
                    if key_event.modifiers.contains(event::KeyModifiers::CONTROL) {
                        // tokio::spawn({
                        //     let mut app = self.clone();
                        //     async move {
                        //         app.execute_sql().await;
                        //         app
                        //     }
                        // });
                    } else {
                        // Insert newline at cursor position
                        self.sql_input.insert(self.sql_cursor_position, '\n');
                        self.sql_cursor_position += 1;
                    }
                }
                KeyCode::Char(c) => {
                    self.sql_input.insert(self.sql_cursor_position, c);
                    self.sql_cursor_position += 1;
                }
                KeyCode::Backspace => {
                    if self.sql_cursor_position > 0 {
                        self.sql_input.remove(self.sql_cursor_position - 1);
                        self.sql_cursor_position -= 1;
                    }
                }
                KeyCode::Left => {
                    if self.sql_cursor_position > 0 {
                        self.sql_cursor_position -= 1;
                    }
                }
                KeyCode::Right => {
                    if self.sql_cursor_position < self.sql_input.len() {
                        self.sql_cursor_position += 1;
                    }
                }
                KeyCode::Up => {
                    // Find the previous newline before cursor
                    let before_cursor = &self.sql_input[..self.sql_cursor_position];
                    if let Some(current_line_start) = before_cursor.rfind('\n') {
                        // Get the previous line's start
                        if let Some(prev_line_start) = before_cursor[..current_line_start].rfind('\n') {
                            let current_col = self.sql_cursor_position - current_line_start - 1;
                            let prev_line_length = current_line_start - prev_line_start - 1;
                            let new_col = current_col.min(prev_line_length);
                            self.sql_cursor_position = prev_line_start + 1 + new_col;
                        } else {
                            // Move to the first line
                            let current_col = self.sql_cursor_position - current_line_start - 1;
                            let first_line_length = current_line_start;
                            let new_col = current_col.min(first_line_length);
                            self.sql_cursor_position = new_col;
                        }
                    }
                }
                KeyCode::Down => {
                    // Find the next newline after cursor
                    if let Some(current_line_end) = self.sql_input[self.sql_cursor_position..].find('\n') {
                        let current_line_end = current_line_end + self.sql_cursor_position;
                        // Find the current line start to calculate column position
                        let before_cursor = &self.sql_input[..self.sql_cursor_position];
                        let current_line_start = before_cursor.rfind('\n')
                            .map(|pos| pos + 1)
                            .unwrap_or(0);
                        let current_col = self.sql_cursor_position - current_line_start;

                        // Find the next line's end
                        if let Some(next_line_end) = self.sql_input[current_line_end + 1..].find('\n') {
                            let next_line_end = next_line_end + current_line_end + 1;
                            let next_line_length = next_line_end - (current_line_end + 1);
                            let new_col = current_col.min(next_line_length);
                            self.sql_cursor_position = current_line_end + 1 + new_col;
                        } else {
                            // Move to the last line
                            let next_line_length = self.sql_input.len() - (current_line_end + 1);
                            let new_col = current_col.min(next_line_length);
                            self.sql_cursor_position = current_line_end + 1 + new_col;
                        }
                    }
                }
                _ => {}
            }
        } else {
            match key_event.code {
                KeyCode::Char('q') => self.exit = true,
                KeyCode::Up => {
                    if !self.show_tables {
                        if self.selected_chain_index > 0 {
                            self.selected_chain_index -= 1;
                            // 更新滚动位置
                            if self.selected_chain_index < self.scroll_offset {
                                self.scroll_offset = self.selected_chain_index;
                            }
                        }
                    } else {
                        if let Some(index) = self.selected_table_index {
                            if index > 0 {
                                self.selected_table_index = Some(index - 1);
                                self.update_example_data();
                            }
                        }
                    }
                }
                KeyCode::Down => {
                    if !self.show_tables {
                        if self.selected_chain_index < self.chains.len() - 1 {
                            self.selected_chain_index += 1;
                            // 使用实际可见高度计算滚动位置
                            if self.selected_chain_index >= self.scroll_offset + visible_height {
                                self.scroll_offset = self.selected_chain_index - visible_height + 1;
                            }
                        }
                    } else {
                        if let Some(index) = self.selected_table_index {
                            let tables_len = self.chains[self.selected_chain_index].dataDictionary.len();
                            if index < tables_len - 1 {
                                self.selected_table_index = Some(index + 1);
                                self.update_example_data();
                            }
                        }
                    }
                }
                KeyCode::Enter => {
                    if !self.show_tables {
                        self.show_tables = true;
                        self.selected_table_index = Some(0);
                        self.update_example_data();
                    } else {
                        // When table is selected, show SQL window
                        self.show_sql_window = true;
                        self.sql_input = self.generate_initial_sql();
                        self.sql_cursor_position = self.sql_input.len();
                    }
                }
                KeyCode::Esc => {
                    if self.show_tables && self.saved_sql.is_some() {
                        // Clear saved SQL and return to table view
                        self.saved_sql = None;
                    } else if self.show_tables {
                        // No saved SQL, exit table view completely
                        self.show_tables = false;
                        self.selected_table_index = None;
                    }
                }
                KeyCode::PageUp => {
                    if !self.show_tables {
                        // 向上一页
                        if self.selected_chain_index > visible_height {
                            self.selected_chain_index -= visible_height;
                        } else {
                            self.selected_chain_index = 0;
                        }
                        // 更新滚动位置
                        if self.selected_chain_index < self.scroll_offset {
                            self.scroll_offset = self.selected_chain_index;
                        }
                    }
                }
                KeyCode::PageDown => {
                    if !self.show_tables {
                        // 向下翻一页
                        let new_index = self.selected_chain_index + visible_height;
                        if new_index < self.chains.len() {
                            self.selected_chain_index = new_index;
                        } else {
                            self.selected_chain_index = self.chains.len() - 1;
                        }
                        // 更新滚动位置
                        if self.selected_chain_index >= self.scroll_offset + visible_height {
                            self.scroll_offset = self.selected_chain_index - visible_height + 1;
                        }
                    }
                }
                KeyCode::Tab => {
                    self.current_tab = (self.current_tab + 1) % 2;
                }
                KeyCode::Char('1') => {
                    self.current_tab = 0;
                }
                KeyCode::Char('2') => {
                    self.current_tab = 1;
                }
                KeyCode::Char('e') => {
                    // 如果保存的 SQL 并且正在显示表格，允许重新编辑
                    if self.show_tables && self.saved_sql.is_some() {
                        self.show_sql_window = true;
                        self.sql_input = self.saved_sql.clone().unwrap_or_default();
                        self.sql_cursor_position = self.sql_input.len();
                    }
                }
                KeyCode::Char('r') => {
                    self.state = AppState::Started;
                    println!("{}", self.docker_setup_in_progress);
                    if !self.docker_setup_in_progress {
                        tokio::spawn({
                            let mut app = self.clone();
                            async move {
                                app.setup_docker().await;
                            }
                        });
                    }
                },
                _ => {}
            }
        }
    }

    // Add new method to generate initial SQL
    fn generate_initial_sql(&self) -> String {
        if let Some(chain) = self.chains.get(self.selected_chain_index) {
            if let Some(table_index) = self.selected_table_index {
                if let Some(table_name) = chain.dataDictionary.keys().nth(table_index) {
                    return format!("SELECT *\nFROM {}.{}\nLIMIT 10", chain.name.to_lowercase(), table_name);
                }
            }
        }
        String::new()
    }

    // Add a new method to handle SQL response
    pub fn handle_sql_response(&mut self, json: serde_json::Value) {

        if let Some(error) = json.get("error") {
            self.sql_error = Some(error.to_string());
            self.sql_executing = false;
            return;
        }

        // Process columns if available
        if let Some(columns) = json.get("columns").and_then(|c| c.as_array()) {
            self.sql_columns = columns.iter()
                .filter_map(|col| {
                    Some(Column {
                        name: col.get("name")?.as_str()?.to_string(),
                        type_: col.get("type")?.as_str()?.to_string(),
                    })
                })
                .collect();
        }

        // Process data if available
        if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
            self.sql_data = data.iter()
                .filter_map(|row| row.as_array().cloned())
                .collect();
        }

        // Update status
        if let Some(stats) = json.get("stats") {
            if let Some(state) = stats.get("state").and_then(|s| s.as_str()) {
                self.sql_result = Some(format!("Query status: {}", state));
                if state == "FINISHED" {
                    println!("Debug: {}", self.sql_data.len());
                    self.sql_executing = false;
                    self.sql_result = Some(format!("Query completed: {} rows returned", self.sql_data.len()));
                }
            }
        }
    }

    pub async fn setup_docker(&mut self) {
        self.docker_setup_in_progress = true;
        self.docker_setup_timer = 0;  // Reset timer when starting setup
        self.docker_status = Some("Setting up Docker environment...".to_string());
        
        match self.docker_manager.setup().await {
            Ok(msg) => {
                self.docker_status = Some(msg);
            },
            Err(e) => {
                self.docker_status = Some(format!("Error: {}", e));
            }
        }
        
        self.docker_setup_in_progress = false;
    }

    // Add new method to get formatted setup progress
    pub fn get_setup_progress_lines(&self) -> Text {
        let mut lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("Setting up Docker environment... ({:.1}s)", 
                    self.docker_setup_timer as f64 / 10.0),
                Style::default().fg(Color::Yellow)
            )),
            Line::from("")
        ];

        // Get progress steps from DockerManager
        let progress_steps = self.docker_manager.get_setup_progress(self.docker_setup_timer);
        let all_steps_completed = progress_steps.iter().all(|(_, completed)| *completed);
        
        for (step, completed) in progress_steps {
            let style = if completed {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            
            let prefix = if completed { "✓" } else { "○" };
            lines.push(Line::from(Span::styled(
                format!("{} {}", prefix, step),
                style
            )));
        }

        // Add status message if available
        if let Some(status) = &self.docker_status {
            lines.push(Line::from(""));
            lines.push(Line::from(status.clone()));
        }

        // If all steps are completed, show completion message
        if all_steps_completed {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "✨ Setup completed successfully!",
                Style::default().fg(Color::Green).bold()
            )));
        }

        Text::from(lines)
    }

    pub fn progress1(&self) -> f64 {
        self.progress1
    }
}

const GAUGE1_COLOR: Color = tailwind::RED.c800;
const CUSTOM_LABEL_COLOR: Color = tailwind::SLATE.c200;
const GAUGE2_COLOR: Color = tailwind::GREEN.c800;

#[derive(Debug, Deserialize)]
struct Response {
    graphData: Vec<GraphData>,
}

#[derive(Debug, Deserialize)]
struct GraphData {
    chain: ChainData,
}

#[derive(Debug, Deserialize)]
struct ChainData {
    name: String,
    status: String,
    lastUpdate: String,
    dataDictionary: DataDictionary,
}

#[derive(Debug, Deserialize)]
struct DataDictionary {
    blocks: Vec<DataDictionaryItem>,
    transactions: Vec<DataDictionaryItem>,
    transactionLogs: Vec<DataDictionaryItem>,
}

pub fn title_block(title: &str) -> Block {
    let title = Title::from(title).alignment(Alignment::Center);
    Block::new()
        .borders(Borders::NONE)
        .padding(Padding::vertical(1))
        .title(title)
        .style(Style::default().fg(CUSTOM_LABEL_COLOR))
}
