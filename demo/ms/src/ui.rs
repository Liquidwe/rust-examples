use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{block::{Position, Title}, Block, List, ListItem, Paragraph, Widget},
};
use crate::app::App;

pub fn draw(frame: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(frame.area());

    // 左侧区域再次分割，用于显示链列表和表格列表
    let left_chunks = if app.show_tables {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[0])
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)])
            .split(chunks[0])
    };

    // 渲染链列表
    let visible_height = left_chunks[0].height as usize - 2; // 减去边框占用的2行
    let chain_names: Vec<ListItem> = app.chains
        .iter()
        .skip(app.scroll_offset)
        .take(visible_height)
        .enumerate()
        .map(|(i, chain)| {
            let content = if i + app.scroll_offset == app.selected_chain_index {
                Line::from(chain.name.clone().bold().green())
            } else {
                Line::from(chain.name.clone())
            };
            ListItem::new(content)
        })
        .collect();

    let chains_block = Block::bordered()
        .title(" Chains ")
        .title_alignment(Alignment::Center)
        .border_set(border::THICK);

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
