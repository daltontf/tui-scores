use super::*;

use assertr::prelude::*;

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
                        curated_rank: Some(CompetitorCuratedRank {
                            current: 20u32
                        })
                    },
                    CompetitionCompetitor {
                        team: CompetitorTeam {
                            display_name: "South Georgia Catfish".into(),
                        },
                        score: "41".into(),
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
                    address: VenueAddress { 
                        city: Some("Pasadena".into()),
                        state: Some("CA".into()),
                        country: Option::None
                    }
                }),
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
fn test_it() {
    let mut event = create_base_event();

    assert_that!(event_to_render_props(&event)).get_some()
        .satisfies(|it| &it.top_team, |top_team| { 
            top_team.is_equal_to("South Georgia Catfish"); })
        .satisfies(|it| &it.bottom_team, |bottom_team| {
            bottom_team.is_equal_to("South Central Louisiana State University Mud Dogs #20"); })
        .satisfies(|it| &it.status, |status| {
            status.is_equal_to(""); })
        .satisfies(|it| &it.location, |location| {
            location.is_equal_to("Pasadena CA"); })
        .satisfies(|it| &it.broadcast, |broadcast| {
            broadcast.is_equal_to("SCTV"); });

    event.competitions[0].status.type_.name = "STATUS_END_PERIOD".into();
    event.competitions[0].status.type_.short_detail = "End of 1st".into();

    assert_that!(event_to_render_props(&event)).get_some()
        .satisfies(|it| &it.status, |status| {
            status.is_equal_to("End of 1st"); });
}