use ratatui_kit::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    prelude::*,
    ratatui::{
        layout::{Alignment, Constraint, Direction},
        style::{Color, Style},
        widgets::{ Block },
        text::Line,
    },
};

use chrono::{Local, NaiveDateTime};
use reqwest::Response;
use serde::Deserialize;
use tokio;

#[derive(Deserialize, Clone)]
struct CompetitorCuratedRank {
    current: u32
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CompetitorTeam {
    display_name: String,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CompetitionCompetitor {
    team: CompetitorTeam,
    curated_rank: Option<CompetitorCuratedRank>,
    score: String,
}

#[derive(Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct StatusType {
    name: String,
    short_detail: String,
}

#[derive(Deserialize,Default, Clone)]
#[serde(rename_all = "camelCase")]
struct EventStatus {
    #[serde(rename = "type")]
    type_: StatusType,
}

#[derive(Deserialize, Clone)]
struct VenueAddress {
    city: String,
    state: Option<String>,
    country: Option<String>,
}

#[derive(Deserialize, Clone)]
struct CompetitionVenue {
    address: VenueAddress,
}

#[derive(Deserialize, Clone)]
struct CompetitionBroadcast {
    market: String,
    names: Vec<String>,
}

#[derive(Deserialize, Clone)]
struct EventCompetition {
    competitors: Vec<CompetitionCompetitor>,
    date: String,
    status: EventStatus,
    venue: Option<CompetitionVenue>,  
    broadcasts: Vec<CompetitionBroadcast>, 
}

#[derive(Deserialize, Default, Clone)]
struct JsonEvent {
    id: String,
    competitions: Vec<EventCompetition>,
    status: EventStatus,
}

#[derive(Deserialize)]
struct JsonPayload {
    events: Vec<JsonEvent>,
}

const CARD_WIDTH: u16 = 40;
const BORDER_PADDING: u16 = 4; // account for ScrollView's border/scrollbar

const LEAGUES: &[(&str, &str)] = &[
    ("", ""),
    ("NFL", "https://site.api.espn.com/apis/site/v2/sports/football/nfl/scoreboard"),
    ("UFL", "https://site.api.espn.com/apis/site/v2/sports/football/ufl/scoreboard"),
    ("MLB", "https://site.api.espn.com/apis/site/v2/sports/baseball/mlb/scoreboard"),
    ("NBA", "https://site.api.espn.com/apis/site/v2/sports/basketball/nba/scoreboard"),
    ("NHL", "https://site.api.espn.com/apis/site/v2/sports/hockey/nhl/scoreboard"),
    ("MLS", "https://site.api.espn.com/apis/site/v2/sports/soccer/usa.1/scoreboard"),
    ("NCAA Football", "http://site.web.api.espn.com/apis/site/v2/sports/football/college-football/scoreboard?group=80"),
    ("NCAA Men's Basketball", "http://site.web.api.espn.com/apis/site/v2/sports/basketball/mens-college-basketball/scoreboard"),
    ("NCAA Women's Basketball", "http://site.web.api.espn.com/apis/site/v2/sports/basketball/womens-college-basketball/scoreboard"),
    ("NCAA Women's Volleyball", "http://site.web.api.espn.com/apis/site/v2/sports/volleyball/womens-college-volleyball/scoreboard"),
    ("NWSL", "http://site.api.espn.com/apis/site/v2/sports/soccer/usa.nwsl/scoreboard"),
    ("USL Championship", "http://site.api.espn.com/apis/site/v2/sports/soccer/usa.usl.1/scoreboard"),
    ("U.S. Open Cup", "http://site.api.espn.com/apis/site/v2/sports/soccer/usa.open/scoreboard"),
    ("UEFA Champions League", "http://site.api.espn.com/apis/site/v2/sports/soccer/uefa.champions/scoreboard"),
    ("English Premier League", "http://site.api.espn.com/apis/site/v2/sports/soccer/eng.1/scoreboard"),
    ("English Championship", "http://site.api.espn.com/apis/site/v2/sports/soccer/eng.2/scoreboard"),
    ("German Bundesliga", "http://site.api.espn.com/apis/site/v2/sports/soccer/ger.1/scoreboard"),
    ("German 2.Bundesliga", "http://site.api.espn.com/apis/site/v2/sports/soccer/ger.2/scoreboard"),
];

async fn fetch_scores(url: &str) -> JsonPayload {
    if url.is_empty() {
        return JsonPayload { events: vec![] };
    }
    
    let client = reqwest::Client::new();

    let response: Response = client
        .get(url)
        .header(reqwest::header::USER_AGENT, "curl/8.5.0") 
        .send()
        .await
        .unwrap();

    response.json().await.unwrap()
}

#[tokio::main]
async fn main() {
    element!(Scores)
        .fullscreen()
        .await
        .expect("Failed to run the application")
}

#[derive(Props, Clone, Default)]
struct RenderEventProps {
    event: JsonEvent
}

fn team_string(competitor: &CompetitionCompetitor) -> String {
    match &competitor.curated_rank {
        Some(curated_rank) if curated_rank.current <= 25 => format!("{} #{}", competitor.team.display_name, curated_rank.current),
        _ => competitor.team.display_name.clone()  
    }
}

// ESPN dates look like "2026-09-23T00:00Z" (no seconds), so RFC 3339 parsing won't work.
fn scheduled_string(date: &str, fallback: &str) -> String {
    NaiveDateTime::parse_from_str(date, "%Y-%m-%dT%H:%MZ")
        .map(|utc| utc.and_utc().with_timezone(&Local).format("%a %b %-d, %-I:%M %p").to_string())
        .unwrap_or_else(|_| fallback.to_string())
}

#[component]
fn RenderEvent(props: &RenderEventProps) -> impl Into<AnyElement<'static>> {
    if let Some(competition) = props.event.competitions.get(0) {
        let team1 = competition.competitors.get(1).map(team_string).unwrap_or_default();
        let team0 = competition.competitors.get(0).map(team_string).unwrap_or_default();

        element!(
            Border(
                width: Constraint::Length(CARD_WIDTH),
                key: props.event.id.clone(),
                border_style: Style::new().fg(Color::LightGreen),
                top_title: Some(Line::from(scheduled_string(&competition.date, ""))),
                
            ) {
                View(flex_direction: Direction::Horizontal, height: Constraint::Length(1)) {
                    View(width: Constraint::Fill(1)) {
                        Text(text: team1, style: Style::new().bold())
                    }
                    View(width: Constraint::Length(5)) {
                        Text(text: competition.competitors.get(1).map(|competitor| competitor.score.clone()).unwrap_or_default(), alignment: Alignment::Right, style: Style::new().bold())
                    }
                }
                View(flex_direction: Direction::Horizontal, height: Constraint::Length(1)) {
                    View(width: Constraint::Fill(1)) {
                        Text(text: team0, style: Style::new().bold())
                    }
                    View(width: Constraint::Length(5)) {
                        Text(text: competition.competitors.get(0).map(|competitor| competitor.score.clone()).unwrap_or_default(), alignment: Alignment::Right, style: Style::new().bold())
                    }
                }
                if competition.status.type_.name != "STATUS_SCHEDULED" {
                    Text(text: props.event.status.type_.short_detail.clone(), style: Style::new().fg(Color::Yellow).bold())
                } else {
                    Text(text: "")
                }
                View(flex_direction: Direction::Horizontal, height: Constraint::Length(1)) {
                    View(width: Constraint::Fill(2)) {
                        Text(text: competition.venue.as_ref()
                            .map(|venue| format!("{} {}", venue.address.city, 
                                venue.address.state.as_ref().or(venue.address.country.as_ref()).unwrap_or(&"".into()))).unwrap_or_default(),
                            style: Style::new().fg(Color::LightCyan))
                    }
                    View(width: Constraint::Fill(1)) {
                        Text(text: competition.broadcasts.iter()
                            .find(|broadcast| broadcast.market == "national")
                            .and_then(|broadcast| broadcast.names.first()).cloned().unwrap_or_default(),
                            alignment: Alignment::Right,
                            style: Style::new().fg(Color::LightCyan))
                    }
                }    
            }
        )
    } else {
        element!(
            Border(width: Constraint::Length(CARD_WIDTH), key: props.event.id.clone()) {
                Text(text: "No competition data available", style: Style::new().fg(Color::Red).bold())
            }
        )
    }
}

#[component]
fn Scores(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let (term_width, _term_height) = hooks.use_terminal_size();

    // Index into LEAGUES, and whether the dropdown is showing.
    let mut league = hooks.use_state(|| 0usize);
    let mut dropdown_open = hooks.use_state(|| false);

    //let date = hooks.use_state(|| "20260922".to_string());

    let (league_name, league_url) = LEAGUES[league.get()];
    // Snapshot the current value; the closure must own its data ('static).
    let url_value: String = league_url.to_string();

    let json_payload = hooks.use_async_state(
        {
            let url_value = url_value.clone();
            async move || Ok::<_, JsonPayload>(fetch_scores(&url_value).await)
        },
        url_value, // deps: re-runs the fetch when the url changes
    );

    let mut exit = hooks.use_exit();

    // While the modal is open it owns an exclusive input layer, so the
    // main handler below goes quiet and this one closes the modal.
    let layer = hooks.use_input_layer(dropdown_open.get(), true);

    hooks.use_event_handler(EventScope::Layer(layer), EventPriority::High, move |event| {
        let Event::Key(key) = event else {
            return EventResult::Ignored;
        };
        if key.kind != KeyEventKind::Press {
            return EventResult::Ignored;
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('l') | KeyCode::Char('L') => {
                dropdown_open.set(false);
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    });

    hooks.use_event_handler(EventScope::Current, EventPriority::Normal, move |event| {
        let Event::Key(key) = event else {
            return EventResult::Ignored;
        };
        if key.kind != KeyEventKind::Press {
            return EventResult::Ignored;
        }

        match key.code {
            KeyCode::Char('l') | KeyCode::Char('L') => {
                dropdown_open.set(true);
                EventResult::Consumed
            }
            KeyCode::Esc => {
                exit();
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    });

    let columns = ((term_width.saturating_sub(BORDER_PADDING)) / CARD_WIDTH).max(1) as usize;

    element!(
        Center(
            width: Constraint::Percentage(100),
            height: Constraint::Percentage(100),
        ) {
            ScrollView(
                flex_direction: Direction::Vertical,
                block: Block::bordered().title(Line::from(format!("Scores - {} (l)", league_name)).centered()),
            ) {
                if let Some(json_payload) = json_payload.data.read().as_ref() {
                    for (i, row) in json_payload.events.chunks(columns).enumerate() {
                        View(flex_direction: Direction::Horizontal, height: Constraint::Length(6), key: i) {
                            for event in row {
                                RenderEvent(event: event.clone())
                            }
                        }
                    }
                } else {
                    Text(text: "Loading...", style: Style::new().fg(Color::Yellow))
                }
            }
            Modal(
                open: dropdown_open.get(),
                layer: Some(layer),
                width: Constraint::Length(36),
                height: Constraint::Length(LEAGUES.len() as u16 + 2),
            ) {
                Select::<&'static str>(
                    top_title: Some(Line::from("Select League").centered()),
                    items: LEAGUES.iter().map(|(name, _)| *name).collect::<Vec<&'static str>>(),
                    default_index: Some(league.get()),
                    on_select: move |name: &'static str| {
                        if let Some(i) = LEAGUES.iter().position(|(n, _)| *n == name) {
                            league.set(i);
                        }
                        dropdown_open.set(false);
                    },
                )
            }
        }
    )
}