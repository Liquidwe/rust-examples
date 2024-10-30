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
    // 创建左右布局
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)]) // 左30%，右70%
        .split(frame.area());

    // 左侧链名称列表
    let chain_names: Vec<ListItem> = app.chains.iter().enumerate().map(|(i, chain)| {
        let content = if i == app.selected_chain_index {
            Line::from(chain.name.clone().bold().green()) // 选中的链用绿色加粗显示
        } else {
            Line::from(chain.name.clone())
        };
        ListItem::new(content)
    }).collect();

    let left_block = Block::bordered()
        .title(" Chains ")
        .title_alignment(Alignment::Center)
        .border_set(border::THICK);

    let chain_list = List::new(chain_names).block(left_block);
    frame.render_widget(chain_list, chunks[0]);

    // 右侧详细信息显示
    if let Some(selected_chain) = app.chains.get(app.selected_chain_index) {
        let data_lines: Vec<Line> = selected_chain.dataDictionary.iter().map(|item| {
            Line::from(format!(
                "{}: {} - {}",
                item.name, item.dataType, item.description
            ))
        }).collect();

        let data_text = Text::from(data_lines);

        let right_block = Block::bordered()
            .title(Title::from(" Data Dictionary ").alignment(Alignment::Center))
            .border_set(border::THICK);

        let data_paragraph = Paragraph::new(data_text).block(right_block);
        frame.render_widget(data_paragraph, chunks[1]);
    }
}
