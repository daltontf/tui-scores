use ratatui_kit::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    prelude::*,
    ratatui::{
        layout::{Alignment, Constraint, Direction},
        style::{Style, Stylize},
        widgets::{ Block },
        text::Line,
    },
};

use chrono::{Local, NaiveDateTime};
use reqwest::Response;
use serde::Deserialize;
use std::sync::LazyLock;
use tokio;

use time::{OffsetDateTime};

use tui_scores::tui_date_picker::{TuiDatePicker};

// Treats a null array as empty and drops null elements, e.g. `null` or `[null]` -> `[]`.
fn vec_skip_nulls<'de, D, T>(d: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    let items = Option::<Vec<Option<T>>>::deserialize(d)?;
    Ok(items.unwrap_or_default().into_iter().flatten().collect())
}

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
struct CompetitorRecord {
    name: String,
    summary: String
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CompetitionCompetitor {
    team: CompetitorTeam,
    curated_rank: Option<CompetitorCuratedRank>,
    #[serde(default, deserialize_with = "vec_skip_nulls")]
    records: Vec<CompetitorRecord>,
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
struct CompetitionStatus {
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
    address: Option<VenueAddress>,
}

#[derive(Deserialize, Clone)]
struct CompetitionBroadcast {
    market: String,
    names: Vec<String>,
}

#[derive(Deserialize, Clone)]
struct CompetitionNote {
    headline: String
}

#[derive(Deserialize, Clone)]
struct CompetitionSeries {
    summary: String
}

#[derive(Deserialize, Clone)]
struct CompetitionOdds {
    details: Option<String>,
}

#[derive(Deserialize, Clone)]
struct EventCompetition {
    competitors: Vec<CompetitionCompetitor>,
    date: String,
    status: CompetitionStatus,
    venue: Option<CompetitionVenue>,  
    broadcasts: Vec<CompetitionBroadcast>, 
    notes: Vec<CompetitionNote>,
    series: Option<CompetitionSeries>,
    #[serde(default, deserialize_with = "vec_skip_nulls")]
    odds: Vec<CompetitionOdds>
}

#[derive(Deserialize, Default, Clone)]
struct PayloadEvent {
    id: String,
    competitions: Vec<EventCompetition>
}

#[derive(Deserialize)]
struct JsonPayload {
    events: Vec<PayloadEvent>,
}

const LEAGUES_CSV: &str = include_str!("../leagues.csv");

// (name, url) pairs borrowed from LEAGUES_CSV; index 0 is the empty "no league" entry.
static LEAGUES: LazyLock<Vec<(&'static str, &'static str)>> = LazyLock::new(|| {
    LEAGUES_CSV.lines().filter_map(|line| {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
             return None;
        }
        let (name, url) = line.split_once(',')?;
        Some((name.trim().trim_matches('"'), url.trim().trim_matches('"')))
    }).collect()
});

const MIN_CARD_WIDTH: u16 = 40;
const BORDER_PADDING: u16 = 4; // account for ScrollView's border/scrollbar
const SCHEDULED_STATUS: &str = "STATUS_SCHEDULED";
const RECORD_OVERALL: &str = "overall";
const NATIONAL: &str = "national";

const SCORE_BORDER_STYLE: Style = Style::new().light_green();
const TEAM_NAME_STYLE: Style = Style::new().bold();
const TEAM_RECORD_STYLE: Style = Style::new().gray();
const TEAM_SCORE_STYLE: Style = Style::new().bold().light_blue();
const STATUS_STYLE: Style = Style::new().bold().yellow();
const ODDS_STYLE: Style = Style::new().gray();
const DESCRIPTION_STYLE: Style = Style::new().light_blue();
const LOCATION_STYLE: Style = Style::new().light_cyan();
const BROADCAST_STYLE: Style = Style::new().light_cyan();

#[derive(Props, Clone, Default)]
struct RenderEventProps {
    scheduled: bool,
    card_width: u16,
    event_id: String,
    date: String,
    top_team: String,
    top_team_score: String,
    bottom_team: String,
    bottom_team_score: String,
    description: String,
    status: Option<String>,
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

fn event_to_render_props(json_event: &PayloadEvent) -> Option<RenderEventProps> {
    json_event.competitions.first().map(|competition| {
        let scheduled = competition.status.type_.name == SCHEDULED_STATUS;
        RenderEventProps {
            scheduled,
            card_width: MIN_CARD_WIDTH,
            event_id: json_event.id.clone(),
            date: scheduled_string(&competition.date, ""),
            top_team: competition.competitors.get(1).map(team_string).unwrap_or_default(),
            top_team_score: if scheduled {
                competition.competitors.get(1)
                    .and_then(|competitor| competitor.records.iter().find(|record| record.name == RECORD_OVERALL))
                    .map(|record| record.summary.clone())
                    .unwrap_or_default()
            } else {
                competition.competitors.get(1).map(|competitor| competitor.score.clone()).unwrap_or_default()
            },
            bottom_team: competition.competitors.get(0).map(team_string).unwrap_or_default(),
            bottom_team_score: if scheduled {
                competition.competitors.get(0)
                    .and_then(|competitor| competitor.records.iter().find(|record| record.name == RECORD_OVERALL))
                    .map(|record| record.summary.clone())
                    .unwrap_or_default()
            } else {
                competition.competitors.get(0).map(|competitor| competitor.score.clone()).unwrap_or_default()
            },
            status: if scheduled {
                competition.odds.first().and_then(|odds| odds.details.clone())
            } else {
                Some(competition.status.type_.short_detail.clone())
            },
            description: {
                let mut result = String::new();
                if let Some(note) = competition.notes.first() {
                    result.push_str(&note.headline);
                }
                if let Some(series) = &competition.series {
                    if !result.is_empty() {
                        result.push_str(" ");
                    }
                    result.push_str(&series.summary);
                }
                result
            },
            location: match competition.venue.as_ref() {
                Some(CompetitionVenue { address: Some(address) }) => {
                    format!("{} {}", 
                        address.city.as_ref().unwrap_or(&"".into()), 
                        address.state.as_ref().or(address.country.as_ref()).unwrap_or(&"".into())
                    )
                },
                _ => "".to_string()
            },
              broadcast: competition.broadcasts.iter()
                        .find(|broadcast| {                            
                            broadcast.market == NATIONAL
                        })
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
            width: Constraint::Length(props.card_width),
            key: props.event_id.clone(),
            border_style: SCORE_BORDER_STYLE,
            top_title: Some(Line::from(props.date.clone())),                
        ) {
            View(flex_direction: Direction::Horizontal, height: Constraint::Length(1)) {
                View(width: Constraint::Fill(1)) {
                    Text(text: props.top_team.clone(), style: TEAM_NAME_STYLE)
                }
                View(width: Constraint::Length(7)) {
                    Text(text: props.top_team_score.clone(),
                         alignment: Alignment::Right,
                         style: if props.scheduled { 
                            TEAM_RECORD_STYLE
                         } else {
                            TEAM_SCORE_STYLE
                         })
                    }
                }
                View(flex_direction: Direction::Horizontal, height: Constraint::Length(1)) {
                    View(width: Constraint::Fill(1)) {
                        Text(text: props.bottom_team.clone(), style: TEAM_NAME_STYLE)
                    }
                    View(width: Constraint::Length(7)) {
                        Text(text: props.bottom_team_score.clone(),
                             alignment: Alignment::Right,
                             style: if props.scheduled { 
                                 TEAM_RECORD_STYLE
                              } else {
                                 TEAM_SCORE_STYLE
                              })
                    }
                }
                if let Some(status) = &props.status {
                    Text(text: status.clone(), 
                        style: if props.scheduled { 
                           ODDS_STYLE
                        } else {
                           STATUS_STYLE
                    })
                }
                Text(text: props.description.clone(), style: DESCRIPTION_STYLE)   
                View(flex_direction: Direction::Horizontal, height: Constraint::Length(1)) {
                    View(width: Constraint::Fill(2)) {
                        Text(text: props.location.clone(),
                            style: LOCATION_STYLE)
                    }
                    View(width: Constraint::Fill(1)) {
                        Text(text: props.broadcast.clone(),
                            alignment: Alignment::Right,
                            style: BROADCAST_STYLE)
                    }
                }    
            }
      )  
}

#[component]
fn Scores(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let now = OffsetDateTime::now_local().expect("Can't get local time");

    let (term_width, _term_height) = hooks.use_terminal_size();
    let mut league = hooks.use_state(|| None::<usize>);
    let mut now_unix_timestamp = hooks.use_state(|| now.unix_timestamp());
    let mut selected_date = hooks.use_state(|| now.date());

    let (league_name, league_url) = league.get().map(|index| LEAGUES[index]).unwrap_or(("", ""));
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

    let event_props = json_payload.data.read().as_ref().map(|payload| payload.events.iter()
        .filter_map(event_to_render_props)
        .collect::<Vec<RenderEventProps>>())
        .unwrap_or_default();

    let card_width = event_props.iter().map(|props| {
        props.description.chars().count() as u16 + 2 // 2 for the border
    }).max().unwrap_or(0).max(MIN_CARD_WIDTH);

    let columns = ((term_width.saturating_sub(BORDER_PADDING)) / card_width).max(1) as usize;
    
    // Don't waste space on the status line if all events are only scheduled; otherwise, leave room for the status line.
    let cell_height = if event_props.iter().all(|event| event.status.is_none()) { 6 } else { 7 };

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
                for (i, row) in event_props.chunks(columns).enumerate() {
                    View(flex_direction: Direction::Horizontal, height: Constraint::Length(cell_height), key: i) {
                        for event in row {
                            RenderEvent(card_width: card_width, ..event.to_owned())
                        }
                    }
                }
            }
            Modal(
                open: league_modal_open.get(),
                layer: Some(league_modal_layer),
                width: Constraint::Length(36),
                height: Constraint::Length(LEAGUES.len() as u16 + 2),
            ) {
                Select::<&'static str>(
                    top_title: Some(Line::from("Select League").centered().white()),
                    bottom_title: Some(Line::from("[Esc]:Cancel ").right_aligned().gray()), 
                    items: LEAGUES.iter().map(|(name, _)| *name).collect::<Vec<&'static str>>(),
                    default_index: league.get().unwrap_or(0),
                    on_select: move |name: &'static str| {
                        league.set(LEAGUES.iter().position(|(n, _)| *n == name));
                        league_modal_open.set(false);
                    }
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
                    border_style: Style::new().light_blue(),
                    top_title: Line::from(" Select Day ").light_blue().bold().centered(),
                    bottom_title: Line::from(" [Esc]: Cancel ").gray().centered(),
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

#[cfg(test)]
mod tests;