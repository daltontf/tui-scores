use super::*;

use assertr::prelude::*;
use ratatui_kit::test_util::render_frame;

const STATUS_SCHEDULED:&str = "STATUS_SCHEDULED";

fn create_base_event() -> PayloadEvent {
    PayloadEvent { id: "foo".into(), 
        competitions: vec![              
            EventCompetition {
                date: "2026-09-20Z".into(),
                competitors: vec![
                    CompetitionCompetitor {
                        team: CompetitorTeam {
                            display_name: "South Central Louisiana State University Mud Dogs".into(),
                        },
                        score: "42".into(),
                        records: vec![CompetitorRecord {
                            name: RECORD_OVERALL.into(),
                            summary: "1-0".into(),
                        }],
                        curated_rank: Some(CompetitorCuratedRank {
                            current: 20u32
                        })
                    },
                    CompetitionCompetitor {
                        team: CompetitorTeam {
                            display_name: "South Georgia Catfish".into(),
                        },
                        score: "41".into(),
                        records: vec![CompetitorRecord {
                            name: RECORD_OVERALL.into(),
                            summary: "0-1".into(),
                        }],
                        curated_rank: Option::None
                    }                        
                ],
                status: CompetitionStatus { 
                    type_: StatusType { 
                        name: STATUS_SCHEDULED.into(),
                        short_detail:  "".into()
                    }
                },
                venue: Some(CompetitionVenue {
                    address: Some(VenueAddress { 
                        city: Some("Pasadena".into()),
                        state: Some("CA".into()),
                        country: Option::None
                    })
                }),
                notes: vec![
                  CompetitionNote { 
                    headline: "Fiction Bowl".into()
                  }  
                ],
                series: Some(CompetitionSeries {
                    summary: "Not a series".into()
                }),
                odds: vec![
                    CompetitionOdds {
                    details: Some("SGU -3.5".into())
                }],
                broadcasts: vec![
                    CompetitionBroadcast {
                        market: "national".into(),
                        names: vec!["SCTV".into(), "MTV".into()]
                    }
                ]
            }
        ]}  
}

#[test]
fn test_json_to_event() {
    let mut event = create_base_event();

    assert_that!(event_to_render_props(&event)).get_some()
        .satisfies(|it| &it.top_team, |it| { 
            it.is_equal_to("South Georgia Catfish"); })
        .satisfies(|it| &it.top_team_score, |it| { 
            it.is_equal_to("0-1"); })
        .satisfies(|it| &it.bottom_team, |it| {
            it.is_equal_to("South Central Louisiana State University Mud Dogs #20"); })
        .satisfies(|it| &it.bottom_team_score, |it| { 
            it.is_equal_to("1-0"); })
        .satisfies(|it| &it.status, |it| {
            it.is_equal_to(Some("SGU -3.5".into())); })
        .satisfies(|it| &it.location, |it| {
            it.is_equal_to("Pasadena CA"); })
        .satisfies(|it| &it.broadcast, |it| {
            it.is_equal_to("SCTV"); });

    event.competitions[0].status.type_.name = "STATUS_END_PERIOD".into();
    event.competitions[0].status.type_.short_detail = "End of 1st".into();

    assert_that!(event_to_render_props(&event)).get_some()
        .satisfies(|it| &it.top_team_score, |it| { 
            it.is_equal_to("41"); })
        .satisfies(|it| &it.bottom_team_score, |it| { 
            it.is_equal_to("42"); })
        .satisfies(|it| &it.status, |it| {
            it.is_equal_to(Some("End of 1st".to_string())); });
}

#[test]
fn test_event_render() {
    let buffer = render_frame(element!(RenderEvent (
        card_width: 45u16,
        top_team: "South Georgia Catfish".to_string(),
        top_team_score: "41".to_string(),
        bottom_team: "South Central Louisiana State University Mud Dogs #20".to_string(),
        bottom_team_score: "38".to_string(),
        status: "Postponed".to_string(),
        description: "Fiction Team Bowl".to_string(),
        location: "Pasadena CA".to_string(),
        broadcast: "MTV".to_string()        
    )), 45, 7);

    let content: Vec<String> = buffer.content()
        .chunks(45 as usize)
        .map(|row| row.iter().map(|c| c.symbol()).collect::<String>())
        .collect();

    assert_eq!("┌───────────────────────────────────────────┐", content.get(0).unwrap());
    assert_eq!("│South Georgia Catfish                    41│", content.get(1).unwrap());
    assert_eq!("│South Central Louisiana State Univer     38│", content.get(2).unwrap());
    assert_eq!("│Postponed                                  │", content.get(3).unwrap());
    assert_eq!("│Fiction Team Bowl                          │", content.get(4).unwrap());
    assert_eq!("│Pasadena CA                             MTV│", content.get(5).unwrap());
    assert_eq!("└───────────────────────────────────────────┘", content.get(6).unwrap());
}