use std::io;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::Constraint;
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Row, Table};
use tui::Terminal;

struct Runner {
    id: usize,
    description: String,
    ip_address: String,
    active: bool,
    is_shared: bool,
    name: String,
    online: bool,
    status: String,
}

fn main() -> Result<(), io::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    let runners: Vec<Runner> = vec![];
    terminal.draw(|f| {
        f.render_widget(create_table(&runners), f.size());
    })?;

    Ok(())
}

fn create_table(runners: &Vec<Runner>) -> Table {
    let headers_style = Style::default()
        .fg(Color::Black)
        .add_modifier(Modifier::BOLD);
    let headers = [
        "ID",
        "NAME",
        "DESCRIPTION",
        "IP",
        "SHARED",
        "ACTIVE",
        "ONLINE",
        "STATUS",
    ];

    let headers = Row::new(headers).style(headers_style);

    let rows: Vec<Row> = runners
        .iter()
        .map(|r| {
            let row = Row::new(vec![
                r.id.to_string(),
                r.name.to_string(),
                r.description.to_string(),
                r.ip_address.to_string(),
                r.is_shared.to_string(),
                r.active.to_string(),
                r.online.to_string(),
                r.status.to_string(),
            ]);

            if r.active {
                row.style(Style::default().fg(Color::Green))
            } else {
                row
            }
        })
        .collect();

    Table::new(rows)
        .header(headers)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Runners ")
                .title_alignment(tui::layout::Alignment::Center),
        )
        .highlight_style(Style::default().bg(Color::LightCyan))
        .widths(&[
            Constraint::Length(5),
            Constraint::Length(15),
            Constraint::Length(40),
            Constraint::Length(15),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(10),
        ])
}
