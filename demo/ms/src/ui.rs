use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{block::{Position, Title}, Block, Paragraph, Widget},
};
use crate::app::App;

pub fn draw(frame: &mut ratatui::Frame, app: &App) {
    // 创建左右两部分的布局
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]) // 左右各占50%
        .split(frame.area());

    // 左侧显示计数器信息
    frame.render_widget(app, chunks[0]);

    // 右侧显示默认内容 "chainbase"
    let right_text = Text::from(vec![Line::from("chainbase".to_string().yellow())]);
    let right_block = Block::bordered()
        .title(Title::from(" Right Panel ").alignment(Alignment::Center))
        .border_set(border::THICK);

    let right_paragraph = Paragraph::new(right_text)
        .centered()
        .block(right_block);
    frame.render_widget(right_paragraph, chunks[1]);
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(" Counter App Tutorial ".bold());
        let instructions = Title::from(Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]));
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            self.counter.to_string().yellow(),
        ])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
