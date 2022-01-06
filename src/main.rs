use futures::stream::FuturesUnordered;
use futures::{stream, StreamExt};
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

enum RunnerInfo {
    Short(Runner),
    Details(RunnerDetails),
}

#[derive(Deserialize, Debug, Clone)]
struct Runner {
    id: usize,
    description: String,
    ip_address: String,
    active: bool,
    is_shared: bool,
    name: Option<String>,
    online: bool,
    status: String,
}

#[derive(Deserialize, Debug, Clone)]
struct RunnerDetails {
    id: usize,
    description: String,
    ip_address: String,
    active: bool,
    is_shared: bool,
    name: Option<String>,
    online: bool,
    status: String,

    architecture: Option<String>,
    runner_type: Option<String>,
    contacted_at: String,
    platform: Option<String>,
    projects: Vec<Project>,
    revision: Option<String>,
    tag_list: Vec<String>,
    version: Option<String>,
    access_level: String,
    maximum_timeout: Option<i64>,
}

#[derive(Deserialize, Debug, Clone)]
struct Project {
    id: i64,
    name: String,
    name_with_namespace: String,
    path: String,
    path_with_namespace: String,
}

struct App {
    state: TableState,
    runners: Vec<RunnerDetails>,
}

impl App {
    fn new() -> App {
        App {
            state: TableState::default(),
            runners: vec![],
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

    pub fn push_runner(&mut self, runner: RunnerDetails) {
        self.runners.push(runner);
    }
}

pub async fn get_runners(
    client: Client,
    host: Url,
) -> Result<Vec<Runner>, Box<dyn std::error::Error>> {
    let runner = client
        .get(host.as_ref())
        .query(&[("per_page", "100")]) // TODO: add pagination
        .send()
        .await?
        .json::<Vec<Runner>>()
        .await?;
    Ok(runner)
}

pub async fn get_runner_details(
    client: Client,
    host: Url,
    id: &usize,
) -> Result<RunnerDetails, Box<dyn std::error::Error>> {
    let details_url = host.join(&id.to_string())?;
    let details = client.get(details_url).send().await?.json().await?;
    Ok(details)
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

    let mut app = App::new();

    let mut headers = header::HeaderMap::new();
    headers.append(
        "PRIVATE-TOKEN",
        header::HeaderValue::from_str(&token).unwrap(),
    );

    let client = Client::builder()
        .default_headers(headers)
        .build()
        .expect("Failed to build HTTP client");
    let host = Url::parse(&format!("https://{host}/api/v4/runners/", host = host)).unwrap();

    let runners = get_runners(client.clone(), host.clone()).await?;
    let mut group = runners
        .iter()
        .map(|runner| get_runner_details(client.clone(), host.clone(), &runner.id))
        .into_iter()
        .collect::<FuturesUnordered<_>>();

    while let Some(item) = group.next().await {
        app.push_runner(item?);
    }

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

fn create_table(runners: &Vec<RunnerDetails>) -> Table {
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
        "TAGS",
    ];

    let headers = Row::new(headers).style(headers_style);
    let rows: Vec<Row> = runners.iter().map(|rd| bulid_detailed_row(&rd)).collect();

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
            Constraint::Length(40),
        ])
}

fn bulid_detailed_row(runner: &RunnerDetails) -> Row<'static> {
    let name = match &runner.name {
        Some(n) => n.to_owned(),
        None => "<unknown>".to_string(),
    };
    let row = Row::new(vec![
        runner.id.to_string(),
        name,
        runner.description.to_string(),
        runner.ip_address.to_string(),
        convert_str_flag(&runner.is_shared).to_string(),
        convert_str_flag(&runner.active).to_string(),
        convert_str_flag(&runner.online).to_string(),
        runner.status.to_string(),
        runner.tag_list.join(",")
    ]);

    match &runner.status[..] {
        "active" => row.style(Style::default().fg(Color::Green)),
        "online" => row,
        "offline" => row.style(Style::default().fg(Color::Red)),
        "paused" => row.style(Style::default().fg(Color::Rgb(255, 175, 0))),
        _ => panic!("Unknown status: {:?}", runner),
    }
}

fn convert_str_flag(flag: &bool) -> &str {
    if *flag {
        "✔"
    } else {
        ""
    }
}
