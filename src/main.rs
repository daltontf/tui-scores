use ratatui_kit::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    prelude::*,
    ratatui::{
        layout::{Alignment, Constraint, Direction},
        style::{Color, Style, Stylize},
        widgets::{ Block },
        text::Line,
    },
};

use chrono::{Local, NaiveDateTime};
use reqwest::Response;
use serde::Deserialize;
use tokio;

use time::{OffsetDateTime};

use tui_scores::tui_date_picker::{TuiDatePicker};

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
    city: Option<String>,
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
    competitions: Vec<EventCompetition>
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
    ("NCAA Men's Basketball", "http://site.web.api.espn.com/apis/site/v2/sports/basketball/mens-college-basketball/scoreboard?group=80"),
    ("NCAA Men's Soccer", "http://site.web.api.espn.com/apis/site/v2/sports/soccer/usa.ncaa.m.1/scoreboard"),
    ("NCAA Men's Hockey", "http://site.api.espn.com/apis/site/v2/sports/hockey/mens-college-hockey/scoreboard"),    
    ("NCAA Baseball", "http://site.api.espn.com/apis/site/v2/sports/baseball/college-baseball/scoreboard"), 
    ("NCAA Women's Basketball", "http://site.web.api.espn.com/apis/site/v2/sports/basketball/womens-college-basketball/scoreboard"),
    ("NCAA Women's Volleyball", "http://site.web.api.espn.com/apis/site/v2/sports/volleyball/womens-college-volleyball/scoreboard"),
    ("NCAA Women's Soccer", "http://site.web.api.espn.com/apis/site/v2/sports/soccer/usa.ncaa.w.1/scoreboard"),
    ("NCAA Women's Hockey", "http://site.api.espn.com/apis/site/v2/sports/hockey/womens-college-hockey/scoreboard"),    
    ("NCAA Women's Softball", "http://site.api.espn.com/apis/site/v2/sports/baseball/college-softball/scoreboard"),
    ("NWSL", "http://site.api.espn.com/apis/site/v2/sports/soccer/usa.nwsl/scoreboard"),
    ("USL Championship", "http://site.api.espn.com/apis/site/v2/sports/soccer/usa.usl.1/scoreboard"),
    ("U.S. Open Cup", "http://site.api.espn.com/apis/site/v2/sports/soccer/usa.open/scoreboard"),
    ("Liga MX", "http://site.api.espn.com/apis/site/v2/sports/soccer/mex.1/scoreboard"),
    ("English Premier League", "http://site.api.espn.com/apis/site/v2/sports/soccer/eng.1/scoreboard"),
    ("English Championship", "http://site.api.espn.com/apis/site/v2/sports/soccer/eng.2/scoreboard"),
    ("German Bundesliga", "http://site.api.espn.com/apis/site/v2/sports/soccer/ger.1/scoreboard"),
    ("German 2.Bundesliga", "http://site.api.espn.com/apis/site/v2/sports/soccer/ger.2/scoreboard"),
    ("Italian Serie A", "http://site.api.espn.com/apis/site/v2/sports/soccer/ita.1/scoreboard"), 
    ("French Ligue 1", "http://site.api.espn.com/apis/site/v2/sports/soccer/fra.1/scoreboard"),
    ("Spanish LALIGA", "http://site.api.espn.com/apis/site/v2/sports/soccer/esp.1/scoreboard"),
    ("UEFA Champions League", "http://site.api.espn.com/apis/site/v2/sports/soccer/uefa.champions/scoreboard"),
    ("UEFA Europa League", "http://site.api.espn.com/apis/site/v2/sports/soccer/uefa.europa/scoreboard")
];

#[derive(Props, Clone, Default)]
struct RenderEventProps {
    event_id: String,
    date: String,
    top_team: String,
    top_team_score: String,
    bottom_team: String,
    bottom_team_score: String,
    status: String,
    location: String,
    broadcast: String
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

fn event_to_render_props(json_event: &JsonEvent) -> Option<RenderEventProps> {
    json_event.competitions.first().map(|competition| {
        RenderEventProps {
            event_id: json_event.id.clone(),
            date: scheduled_string(&competition.date, ""),
            top_team: competition.competitors.get(1).map(team_string).unwrap_or_default(),
            top_team_score: competition.competitors.get(1).map(|competitor| competitor.score.clone()).unwrap_or_default(),
            bottom_team: competition.competitors.get(0).map(team_string).unwrap_or_default(),
            bottom_team_score: competition.competitors.get(0).map(|competitor| competitor.score.clone()).unwrap_or_default(),
            status: if competition.status.type_.name != "STATUS_SCHEDULED" {
                        competition.status.type_.short_detail.clone()
                    } else {
                        "".to_string()
                    },
            location: competition.venue.as_ref()
                        .map(|venue| format!("{} {}", venue.address.city.as_ref().unwrap_or(&"".into()), 
                            venue.address.state.as_ref().or(venue.address.country.as_ref()).unwrap_or(&"".into()))).unwrap_or_default(),
            broadcast: competition.broadcasts.iter()
                        .find(|broadcast| broadcast.market == "national")
                        .and_then(|broadcast| broadcast.names.first()).cloned().unwrap_or_default()            
        }
    })
}

async fn fetch_scores(url: &str, date: time::Date, cache_avoidance: &str) -> JsonPayload {
    if url.is_empty() {
        return JsonPayload { events: vec![] };
    }

    // ESPN expects dates=YYYYMMDD; some league urls already carry a query string.
    let separator = if url.contains('?') { '&' } else { '?' };
    let url = format!(
        "{url}{separator}dates={:04}{:02}{:02}{cache_avoidance}", 
        date.year(),
        u8::from(date.month()),
        date.day()       
    );

    let client = reqwest::Client::new();

    let response: Response = client
        .get(&url)
        .header(reqwest::header::USER_AGENT, "curl/8.5.0") // Works
        .send()
        .await
        .unwrap();

    response.json().await.unwrap()
}

#[component]
fn RenderEvent(props: &RenderEventProps) -> impl Into<AnyElement<'static>> {
    element!(
        Border(
            width: Constraint::Length(CARD_WIDTH),
            key: props.event_id.clone(),
            border_style: Style::new().fg(Color::LightGreen),
            top_title: Some(Line::from(props.date.clone())),                
        ) {
            View(flex_direction: Direction::Horizontal, height: Constraint::Length(1)) {
                View(width: Constraint::Fill(1)) {
                    Text(text: props.top_team.clone(), style: Style::new().bold())
                }
                View(width: Constraint::Length(5)) {
                    Text(text: props.top_team_score.clone(), alignment: Alignment::Right, style: Style::new().bold())
                    }
                }
                View(flex_direction: Direction::Horizontal, height: Constraint::Length(1)) {
                    View(width: Constraint::Fill(1)) {
                        Text(text: props.bottom_team.clone(), style: Style::new().bold())
                    }
                    View(width: Constraint::Length(5)) {
                        Text(text: props.bottom_team_score.clone(), alignment: Alignment::Right, style: Style::new().bold())
                    }
                }
                Text(text: props.status.clone(), style: Style::new().fg(Color::Yellow).bold())
                View(flex_direction: Direction::Horizontal, height: Constraint::Length(1)) {
                    View(width: Constraint::Fill(2)) {
                        Text(text: props.location.clone(),
                            style: Style::new().fg(Color::LightCyan))
                    }
                    View(width: Constraint::Fill(1)) {
                        Text(text: props.broadcast.clone(),
                            alignment: Alignment::Right,
                            style: Style::new().fg(Color::LightCyan))
                    }
                }    
            }
      )  
}

#[component]
fn Scores(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let now = OffsetDateTime::now_utc();

    let (term_width, _term_height) = hooks.use_terminal_size();
    let mut league = hooks.use_state(|| 0usize);
    let mut now_unix_timestamp = hooks.use_state(|| now.unix_timestamp());
    let mut selected_date = hooks.use_state(|| now.date());

    let (league_name, league_url) = LEAGUES[league.get()];
    // Snapshot the current value; the closure must own its data ('static).
    let url_value: String = league_url.to_string();  
  
    let refresh_text = if now.date() == selected_date.get() && !url_value.is_empty() {
        " [R]:Refresh "
    } else {
        ""
    };  

    let date_value = selected_date.get();
    let now_unix_timestamp_value = now_unix_timestamp.get();

    let json_payload = hooks.use_async_state({
            let url_value = url_value.clone();
            let cache_avoidance = if now.date() == selected_date.get() {
                format!("?secs={}", now_unix_timestamp_value)
            } else {
                "".to_string()
            };
            async move || Ok::<_, JsonPayload>(fetch_scores(&url_value, date_value, &cache_avoidance).await)
        },
        (url_value, date_value, now_unix_timestamp_value), // deps: re-runs the fetch when the url or date changes
    );

    let mut exit = hooks.use_exit();

    let mut league_modal_open = hooks.use_state(|| false);
    let league_modal_layer = hooks.use_input_layer(league_modal_open.get(), true);
    
    hooks.use_event_handler(EventScope::Layer(league_modal_layer), EventPriority::High, move |event| {
        let Event::Key(key) = event else {
            return EventResult::Ignored;
        };
        if key.kind != KeyEventKind::Press {
            return EventResult::Ignored;
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('l') | KeyCode::Char('L') => {
                league_modal_open.set(false);
                EventResult::Consumed
            }, 
            _ => EventResult::Ignored,
        }
    });

    let mut datepicker_modal_open = hooks.use_state(|| false);
    let datepicker_model_layer = hooks.use_input_layer(datepicker_modal_open.get(), true);
    
    hooks.use_event_handler(EventScope::Layer(datepicker_model_layer), EventPriority::High, move |event| {
        let Event::Key(key) = event else {
            return EventResult::Ignored;
        };
        if key.kind != KeyEventKind::Press {
            return EventResult::Ignored;
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('d') | KeyCode::Char('D') => {
                datepicker_modal_open.set(false);
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
                league_modal_open.set(true);
                EventResult::Consumed
            },
            KeyCode::Char('d') | KeyCode::Char('D') => {
                datepicker_modal_open.set(true);
                EventResult::Consumed
            },
            KeyCode::Char('r') | KeyCode::Char('R') if now.date() == selected_date.get() => {
                now_unix_timestamp.set(now.unix_timestamp()); // force a fetch
                EventResult::Consumed
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => {
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
                block: Block::bordered()
                    .title(Line::from(format!(" {} Scores {:04}-{:02}-{:02} ", 
                        league_name,
                        selected_date.get().year(),
                        u8::from(selected_date.get().month()),
                        selected_date.get().day()
                    )).centered())
                    .title_bottom("[L]:League ")
                    .title_bottom(" [D]:Date ")
                    .title_bottom(refresh_text)
                    .title_bottom(Line::from("[Q]:Quit ").right_aligned())
            ) {
                if let Some(json_payload) = json_payload.data.read().as_ref() {
                    for (i, row) in json_payload.events.iter()
                        .filter_map(event_to_render_props)
                        .collect::<Vec<RenderEventProps>>()
                        .chunks(columns).enumerate() {
                        View(flex_direction: Direction::Horizontal, height: Constraint::Length(6), key: i) {
                            for event in row {
                                RenderEvent(..event.to_owned())
                            }
                        }
                    }
                } else {
                    Text(text: "Loading...", style: Style::new().fg(Color::Yellow))
                }
            }
            Modal(
                open: league_modal_open.get(),
                layer: Some(league_modal_layer),
                width: Constraint::Length(36),
                height: Constraint::Length(LEAGUES.len() as u16 + 2),
            ) {
                Select::<&'static str>(
                    top_title: Some(Line::from("Select League").centered()),
                    bottom_title: Some(Line::from("[Esc]:Cancel ").right_aligned()), 
                    items: LEAGUES.iter().map(|(name, _)| *name).collect::<Vec<&'static str>>(),
                    default_index: Some(league.get()),
                    on_select: move |name: &'static str| {
                        if let Some(i) = LEAGUES.iter().position(|(n, _)| *n == name) {
                            league.set(i);
                        }
                        league_modal_open.set(false);
                    },
                )
            }
            Modal(
                open: datepicker_modal_open.get(),
                layer: Some(datepicker_model_layer),
                width: Constraint::Length(24),
                // 2 border rows + month header + weekday header + up to 6 week rows
                height: Constraint::Length(10),
            ) {
                Border(
                    border_style: Style::new().blue(),
                    top_title: Line::from(" Select Day ").blue().bold().centered(),
                    bottom_title: Line::from(" [Esc]: Cancel ").dark_gray().centered(),
                ) {
                    TuiDatePicker(
                        date: selected_date.get(),
                        on_select: move |date| { 
                            selected_date.set(date);
                            datepicker_modal_open.set(false);
                        }
                    )
                }
            }
        }
    )
}

#[tokio::main]
async fn main() {
    element!(Scores)
        .fullscreen()
        .await
        .expect("Failed to run the application")
}
