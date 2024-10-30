use std::{io, collections::HashMap};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::DefaultTerminal;
use serde::{Deserialize, Serialize};
use reqwest;

use crate::ui;

#[derive(Debug, Default)]
pub struct App {
    pub chains: Vec<Chain>,            // 保存所有链信息
    pub selected_chain_index: usize,    // 记录当前选中的链索引
    pub selected_table_index: Option<usize>,  // 新增：当前选中的表索引
    pub show_tables: bool,                    // 新增：是否显示表列表
    pub scroll_offset: usize,    // 新增：跟踪滚动位置
    pub exit: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Chain {
    pub name: String,
    pub dataDictionary: HashMap<String, Vec<DataDictionaryItem>>,  // 修改：使用 HashMap 存储表和字段
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
        }
    }

    async fn fetch_chains() -> Result<Vec<Chain>, reqwest::Error> {
        let url = "https://api.chainbase.com/api/v1/metadata/network_chains";
        println!("Sending request to {}", url);

        match reqwest::get(url).await?.json::<Response>().await {
            Ok(response) => {
                println!("Parsed response successfully, received {} chains", response.graphData.len());
                Ok(response.graphData.into_iter()
                    .map(|graph_data| {
                        let mut tables = HashMap::new();
                        // 将不同的表数据添加到 HashMap 中
                        tables.insert("blocks".to_string(), graph_data.chain.dataDictionary.blocks);
                        tables.insert("transactions".to_string(), graph_data.chain.dataDictionary.transactions);
                        tables.insert("transactionLogs".to_string(), graph_data.chain.dataDictionary.transactionLogs);
                        
                        Chain {
                            name: graph_data.chain.name,
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


    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            let visible_height = terminal.size()?.height as usize - 2; // 减去边框等装饰元素的高度
            terminal.draw(|frame| ui::draw(frame, self))?;
            
            // 处理事件时传入可见高度
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event, visible_height)
                }
                _ => {}
            };
        }
        Ok(())
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
                        }
                    }
                }
            }
            KeyCode::Enter => {
                if !self.show_tables {
                    self.show_tables = true;
                    self.selected_table_index = Some(0);
                }
            }
            KeyCode::Esc => {
                if self.show_tables {
                    self.show_tables = false;
                    self.selected_table_index = None;
                }
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
    dataDictionary: DataDictionary,
}

#[derive(Debug, Deserialize)]
struct DataDictionary {
    blocks: Vec<DataDictionaryItem>,
    transactions: Vec<DataDictionaryItem>,
    transactionLogs: Vec<DataDictionaryItem>,
}
