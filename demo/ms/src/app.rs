use std::{io, collections::HashMap, time::Duration};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::DefaultTerminal;
use serde::{Deserialize, Serialize};
use reqwest;
use serde_json::json;

use crate::ui;

#[derive(Debug, Default)]
pub struct App {
    pub chains: Vec<Chain>,            // 保存所有链信息
    pub selected_chain_index: usize,    // 记录当前选中的链索引
    pub selected_table_index: Option<usize>,  // 新增：当前选中的表索引
    pub show_tables: bool,                    // 新增：是否显示表列表
    pub scroll_offset: usize,    // 新增：跟踪滚动位置
    pub exit: bool,
    pub current_tab: usize,  // Add this line
    pub example_data: Option<ExampleData>,  // Add this line
}

#[derive(Debug, Default)]
pub struct ExampleData {
    pub columns: Vec<Column>,
    pub data: Vec<Vec<serde_json::Value>>,
}

#[derive(Debug)]
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
        let chains = match App::fetch_chains().await {
            Ok(data) => data,
            Err(_) => {
                println!("Failed to load chain data. Initializing with empty data.");
                Vec::new() // 如果加载失败，返回空向量
            }
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
                // Add more columns as needed
            ],
            data: vec![
                vec![
                    json!(0),
                    json!("0x81005434635456a16f74ff7023fbe0bf423abbc8a8deb093ffff455c0ad3b741"),
                    json!("0x0000000000000000000000000000000000000000000000000000000000000000"),
                    // Add more field values
                ],
                // Add more rows as needed
            ],
        }
    }

    fn mock_transaction_logs_data() -> ExampleData {
        ExampleData {
            columns: vec![
                Column { name: "block_number".to_string(), type_: "bigint".to_string() },
                Column { name: "hash".to_string(), type_: "varchar(66)".to_string() },
                Column { name: "parent_hash".to_string(), type_: "varchar(66)".to_string() },
                // Add more columns as needed
            ],
            data: vec![
                vec![
                    json!(0),
                    json!("0x81005434635456a16f74ff7023fbe0bf423abbc8a8deb093ffff455c0ad3b741"),
                    json!("0x0000000000000000000000000000000000000000000000000000000000000000"),
                    // Add more field values
                ],
                // Add more rows as needed
            ],
        }
    }

    fn mock_transactions_data() -> ExampleData {
        ExampleData {
            columns: vec![
                Column { name: "block_number".to_string(), type_: "bigint".to_string() },
                Column { name: "hash".to_string(), type_: "varchar(66)".to_string() },
                Column { name: "parent_hash".to_string(), type_: "varchar(66)".to_string() },
                // Add more columns as needed
            ],
            data: vec![
                vec![
                    json!(0),
                    json!("0x81005434635456a16f74ff7023fbe0bf423abbc8a8deb093ffff455c0ad3b741"),
                    json!("0x0000000000000000000000000000000000000000000000000000000000000000"),
                    // Add more field values
                ],
                // Add more rows as needed
            ],
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            let visible_height = terminal.size()?.height as usize - 2;
            
            // Draw the UI
            terminal.draw(|frame| ui::draw(frame, self))?;
            
            // Wait for event with timeout (100ms)
            if event::poll(Duration::from_millis(1000))? {
                // Handle events if any
                if let Event::Key(key_event) = event::read()? {
                    if key_event.kind == KeyEventKind::Press {
                        self.handle_key_event(key_event, visible_height);
                    }
                }
            }
            // If no event, the timeout will trigger a redraw anyway
        }
        Ok(())
    }

    pub fn update_example_data(&mut self) {
        if let Some(selected_chain) = self.chains.get(self.selected_chain_index) {
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
                }
            }
            KeyCode::Esc => {
                if self.show_tables {
                    self.show_tables = false;
                    self.selected_table_index = None;
                }
            }
            KeyCode::PageUp => {
                if !self.show_tables {
                    // 向上翻一页
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
            _ => {}
        }
    }
}

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
