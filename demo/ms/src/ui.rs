use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Stylize, Color, Style},
    symbols::border,
    text::{Line, Text},
    widgets::{block::{Position, Title}, Block, List, ListItem, Paragraph, Widget, Tabs},
};
use crate::app::App;

pub fn draw(frame: &mut ratatui::Frame, app: &App) {
    // Create tabs
    let titles = vec!["NETWORK [1]", "MANUSCRIPTS [2]"];
    let tabs = Tabs::new(titles)
        .block(Block::bordered().title("Tabs"))
        .select(app.current_tab)
        .style(Style::default())
        .highlight_style(Style::default().bold());

    // Create main layout with space for tabs
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Height for tabs
            Constraint::Min(0),     // Remaining space for content
        ])
        .split(frame.area());

    // Render tabs
    frame.render_widget(tabs, main_chunks[0]);

    match app.current_tab {
        0 => {
            // Original content
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(main_chunks[1]);  // Use main_chunks[1] instead of frame.area()

            // 左侧区域再次分割，用于显示链列表和表格列表
            let left_chunks = if app.show_tables {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(45),  // Reduced to make room for hints
                        Constraint::Percentage(45),
                        Constraint::Percentage(10),  // New space for key hints
                    ])
                    .split(chunks[0])
            } else {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(90),  // Reduced to make room for hints
                        Constraint::Percentage(10),  // Space for key hints
                    ])
                    .split(chunks[0])
            };

            // 渲染链列表
            let visible_height = left_chunks[0].height as usize - 2; // 减去边框占用的2行
            let chains_block = Block::bordered()
                .border_set(border::THICK)
                .title(Title::from(" Chains ").alignment(Alignment::Center));

            let chain_names: Vec<ListItem> = app.chains
                .iter()
                .skip(app.scroll_offset)
                .take(visible_height)
                .enumerate()
                .map(|(i, chain)| {
                    let index = i + app.scroll_offset + 1; // Calculate the 1-based index
                    let time_ago_style = if chain.time_ago.contains("min") && 
                        chain.time_ago.as_str().trim_end_matches(" min").parse::<u64>().unwrap_or(0) > 10 {
                        chain.time_ago.as_str().yellow()
                    } else {
                        let display_time = if chain.time_ago == "unknown" { "-" } else { &chain.time_ago };
                        display_time.white()
                    };

                    let content = if i + app.scroll_offset == app.selected_chain_index {
                        Line::from(vec![
                            format!("{:<3} {:<25}", index, chain.name).bold().white().into(),
                            format!("{:<20}", chain.status).bold().into(),
                            format!("{:<10}", time_ago_style).bold().into(),
                        ])
                    } else {
                        Line::from(vec![
                            format!("{:<3}. {:<25}", index, chain.name).bold()
                                .style(if chain.status == "Online" && chain.time_ago.contains("min") { 
                                    Style::default().fg(Color::Green)
                                } else if chain.status == "Offline" {
                                    Style::default().fg(Color::Red)
                                } else { 
                                    Style::default().fg(Color::Yellow) 
                                }).into(),
                            format!("{:<20}", chain.status).bold()
                                .style(if chain.status == "Online" && chain.time_ago.contains("min") { 
                                    Style::default().fg(Color::Green)
                                } else if chain.status == "Offline" {
                                    Style::default().fg(Color::Red)
                                } else { 
                                    Style::default().fg(Color::Yellow) 
                                }).into(),
                            format!("{:<10}", time_ago_style).bold()
                                .style(if chain.status == "Online" && chain.time_ago.contains("min") { 
                                    Style::default().fg(Color::Green)
                                } else if chain.status == "Offline" {
                                    Style::default().fg(Color::Red)
                                } else { 
                                    Style::default().fg(Color::Yellow) 
                                }).into(),
                        ])
                    };
                    ListItem::new(content)
                })
                .collect();

            let chain_list = List::new(chain_names).block(chains_block);
            frame.render_widget(chain_list, left_chunks[0]);

            // 如果显示表格列表，则渲染表格列表
            if app.show_tables {
                if let Some(selected_chain) = app.chains.get(app.selected_chain_index) {
                    let table_names: Vec<ListItem> = selected_chain.dataDictionary
                        .keys()
                        .enumerate()
                        .map(|(i, table_name)| {
                            let content = if Some(i) == app.selected_table_index {
                                Line::from(table_name.clone().bold().green())
                            } else {
                                Line::from(table_name.clone())
                            };
                            ListItem::new(content)
                        })
                        .collect();

                    let tables_block = Block::bordered()
                        .title(" Tables ")
                        .title_alignment(Alignment::Center)
                        .border_set(border::THICK);

                    let table_list = List::new(table_names).block(tables_block);
                    frame.render_widget(table_list, left_chunks[1]);
                }
            }

            // Add key hints at the bottom
            let hints = vec![
                "Enter: Select",
                "PageUp/Down: Navigate",
                "q: Quit",
            ];
            let hints_text = Text::from(hints.join(" | "));
            let hints_block = Block::bordered()
                .title(" Controls ")
                .title_alignment(Alignment::Center)
                .border_set(border::THICK);
            let hints_paragraph = Paragraph::new(hints_text)
                .block(hints_block)
                .alignment(Alignment::Center);
            
            // Render hints in the bottom section
            frame.render_widget(
                hints_paragraph,
                if app.show_tables { left_chunks[2] } else { left_chunks[1] }
            );

            // 右侧显示字段信息
            if let Some(selected_chain) = app.chains.get(app.selected_chain_index) {
                let data_lines = if app.show_tables && app.selected_table_index.is_some() {
                    // 获取选中的表名
                    let table_name = selected_chain.dataDictionary
                        .keys()
                        .nth(app.selected_table_index.unwrap())
                        .map(|s| s.as_str())
                        .unwrap_or("");

                    // 获取选中表的字段信息
                    if let Some(fields) = selected_chain.dataDictionary.get(table_name) {
                        fields.iter().map(|item| {
                            Line::from(format!(
                                "{}: {} - {}",
                                item.name, item.dataType, item.description
                            ))
                        }).collect()
                    } else {
                        vec![Line::from("No fields available")]
                    }
                } else {
                    vec![Line::from("Select a table to view fields")]
                };

                let right_block = Block::bordered()
                    .title(" Data Dictionary ")
                    .title_alignment(Alignment::Center)
                    .border_set(border::THICK);

                let data_paragraph = Paragraph::new(data_lines)
                    .block(right_block)
                    .wrap(ratatui::widgets::Wrap { trim: true });
                frame.render_widget(data_paragraph, chunks[1]);
            }
        }
        1 => {
            // Tab 2 content
            let tab2_text = Paragraph::new("tab2 text")
                .block(Block::bordered().title("Tab 2"))
                .alignment(Alignment::Center);
            frame.render_widget(tab2_text, main_chunks[1]);
        }
        _ => unreachable!(),
    }
}
