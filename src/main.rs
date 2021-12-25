use reqwest::{header, Client, Url};
use serde::Deserialize;
use std::io::{self};
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::Constraint;
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Row, Table, TableState};
use tui::Terminal;

#[derive(Deserialize, Debug)]
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

struct App {
    state: TableState,
    runners: Vec<Runner>,
    client: Client,
    host: Url,
}

impl App {
    fn new(host: String, token: String) -> App {
        let mut headers = header::HeaderMap::new();
        headers.append(
            "PRIVATE-TOKEN",
            header::HeaderValue::from_str(&token).unwrap(),
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to build HTTP client");

        App {
            state: TableState::default(),
            runners: vec![],
            host: Url::parse(&format!("https://{host}/api/v4/runners/", host = host)).unwrap(),
            client: client,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.runners.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.runners.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub async fn update_runners(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.runners = self
            .client
            .get(self.host.as_ref())
            .query(&[("per_page", "100")]) // TODO: add pagination
            .send()
            .await?
            .json()
            .await?;

        let i = match self.state.selected() {
            Some(i) => {
                if i > self.runners.len() - 1 {
                    self.runners.len()
                } else {
                    i
                }
            }
            None => 0,
        };
        self.state.select(Some(i));

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args();
    args.next();

    let host = std::env::var("ISOLA_HOST")
        .unwrap_or_else(|_| args.next().expect("first argument must be host"));

    let token = std::env::var("ISOLA_TOKEN")
        .unwrap_or_else(|_| args.next().expect("second argument must be token"));

    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(String::from(host), String::from(token));
    app.update_runners().await?;
    println!("{:?}", app.runners);
    let mut events = io::stdin().events();

    loop {
        terminal.draw(|f| {
            f.render_stateful_widget(create_table(&app.runners), f.size(), &mut app.state);
        })?;

        let c = &events.next().unwrap()?;
        match c {
            Event::Key(Key::Char('q')) => break,
            Event::Key(Key::Down) | Event::Key(Key::Char('j')) => app.next(),
            Event::Key(Key::Up) | Event::Key(Key::Char('k')) => app.previous(),
            _ => {}
        }
    }
    terminal.clear()?;

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
            let shared = if r.is_shared { "✓" } else { "" };
            let active = if r.active { "✔" } else { "" };
            let online = if r.online { "✔" } else { "" };

            let row = Row::new(vec![
                r.id.to_string(),
                r.name.to_string(),
                r.description.to_string(),
                r.ip_address.to_string(),
                shared.to_string(),
                active.to_string(),
                online.to_string(),
                r.status.to_string(),
            ]);

            match &r.status[..] {
                "active" => row.style(Style::default().fg(Color::Green)),
                "online" => row,
                "offline" => row.style(Style::default().fg(Color::Red)),
                "paused" => row.style(Style::default().fg(Color::Rgb(255, 175, 0))),
                _ => panic!("Unknown status: {:?}", r),
            }
        })
        .collect();

    Table::new(rows)
        .header(headers)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Runners [{}] ", runners.len()))
                .title_alignment(tui::layout::Alignment::Center),
        )
        .highlight_style(Style::default().bg(Color::DarkGray))
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
