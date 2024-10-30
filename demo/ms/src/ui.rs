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
    let chain_names: Vec<ListItem> = app.chains
        .iter()
        .skip(app.scroll_offset)
        .take(visible_height)
        .enumerate()
        .map(|(i, chain)| {
            let status_style = if chain.status != "Online" {
                chain.status.clone().red()
            } else {
                chain.status.clone().white()
            };

            // 解析和检查最后更新时间
            let last_update_style = if should_highlight_time(&chain.lastUpdate) {
                chain.lastUpdate.clone().yellow()
            } else {
                chain.lastUpdate.clone().white()
            };

            let content = if i + app.scroll_offset == app.selected_chain_index {
                Line::from(vec![
                    chain.name.clone().bold().green().into(),
                    "  ".into(),
                    status_style.bold().into(),
                    "  ".into(),
                    last_update_style.bold().into(),
                ])
            } else {
                Line::from(vec![
                    chain.name.clone().into(),
                    "  ".into(),
                    status_style.into(),
                    "  ".into(),
                    last_update_style.into(),
                ])
            };
            ListItem::new(content)
        })
        .collect();

    let chains_block = Block::bordered()
        .title(" Chain     Status    Last Update ")  // 更新标题以对齐列
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

// 添加这个辅助函数来检查时间是否超过10分钟
fn should_highlight_time(time_str: &str) -> bool {
    // 这里需要根据实际的时间格式来实现解析逻辑
    // 示例实现，假设时间格式为"XM"，其中X是分钟数
    if let Some(minutes) = time_str.trim_end_matches('M').parse::<u64>().ok() {
        minutes > 10
    } else {
        false
    }
}
