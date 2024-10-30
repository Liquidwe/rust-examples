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
    pub exit: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Chain {
    pub name: String,
    pub dataDictionary: Vec<DataDictionaryItem>,
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
            exit: false,
        }
    }

    async fn fetch_chains() -> Result<Vec<Chain>, reqwest::Error> {
        let url = "https://api.chainbase.com/api/v1/metadata/network_chains";
        println!("Sending request to {}", url);

        match reqwest::get(url).await {
            Ok(resp) => {
                println!("Request successful, processing response...");
                match resp.json::<Response>().await {
                    Ok(response) => {
                        println!("Parsed response successfully, received {} chains", response.graphData.len());
                        Ok(response.graphData.into_iter()
                            .map(|graph_data| Chain {
                                name: graph_data.chain.name,
                                dataDictionary: graph_data.chain.dataDictionary.blocks,
                            })
                            .collect())
                    }
                    Err(e) => {
                        println!("Failed to parse JSON response: {:?}", e);
                        Err(e)
                    }
                }
            }
            Err(e) => {
                println!("HTTP request failed: {:?}", e);
                Err(e)
            }
        }
    }


    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| ui::draw(frame, self))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Up => {
                if self.selected_chain_index > 0 {
                    self.selected_chain_index -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected_chain_index < self.chains.len() - 1 {
                    self.selected_chain_index += 1;
                }
            }
            _ => {}
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
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
}
