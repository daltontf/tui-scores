use ratatui_kit::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    prelude::*,
    ratatui::{
        style::{Style, Modifier},
        widgets::calendar::{ CalendarEventStore, Monthly }
    }
};
use time::{Date, Duration, OffsetDateTime};

#[derive(Clone, Copy)]
pub struct CalendarDate(Date);

impl Default for CalendarDate {
    fn default() -> Self {
        Self(OffsetDateTime::now_utc().date()) 
    }
}

impl Into<CalendarDate> for Date {
    fn into(self) -> CalendarDate {
        CalendarDate(self)
    }
}

#[derive(Props, Default)]
pub struct TuiDatePickerProps<'a> {
    pub date: CalendarDate,
    pub on_select: Handler<'a, Date>
}

#[component]
pub fn TuiDatePicker<'a>(mut hooks: Hooks, props: &mut TuiDatePickerProps<'static>) -> impl Into<AnyElement<'static>> {
    let initial = props.date.0;
    let mut date = hooks.use_state(move || initial);
    let mut on_select = props.on_select.take(); // moved into the closure below

    hooks.use_event_handler(EventScope::Current, EventPriority::Normal, move |event| {
        let Event::Key(key) = event else {
            return EventResult::Ignored;
        };
        if key.kind != KeyEventKind::Press {
            return EventResult::Ignored;
        } 

        let current = date.get();  
        
        if key.code == KeyCode::Enter {
            on_select(date.get());
            return EventResult::Consumed;
        }        
       
        let next = match key.code {
            KeyCode::Left => current.checked_add(Duration::days(-1)),
            KeyCode::Right => current.checked_add(Duration::days(1)),
            KeyCode::Up => current.checked_add(Duration::weeks(-1)),
            KeyCode::Down => current.checked_add(Duration::weeks(1)),
            KeyCode::PageUp => add_years(current, -1),
            KeyCode::PageDown => add_years(current, 1),
            KeyCode::Home => Some(OffsetDateTime::now_utc().date()),
            KeyCode::Enter => {
                on_select(date.get());
                Option::None
            },   
            _ => return EventResult::Ignored,
        };
        // None means we hit the edge of the supported date range; keep the current date
        if let Some(next) = next {
            date.set(next);
        }
        EventResult::Consumed
    });

    element!(CalendarView(date: CalendarDate(date.get())))
}

/// Shifts a date by whole years, clamping Feb 29 to Feb 28 when the target year isn't a leap year.
fn add_years(date: Date, years: i32) -> Option<Date> {
    let year = date.year().checked_add(years)?;
    let day = date.day().min(date.month().length(year));
    Date::from_calendar_date(year, date.month(), day).ok()
}

struct CalendarView {
   date: CalendarDate
}

impl Component for CalendarView {
    type Props<'a> = TuiDatePickerProps<'a>;

    fn new(props: &Self::Props<'_>) -> Self {
        Self { date: props.date.clone() }
    }

    fn update(
        &mut self,
        props: &mut Self::Props<'_>,
        _hooks: Hooks,
        _updater: &mut ComponentUpdater,
    ) {
        self.date = props.date.clone();
    }

    fn draw(&mut self, drawer: &mut ComponentDrawer<'_, '_>) {
        let mut event_store = CalendarEventStore::today(Style::default().red().bold());
        event_store.add(self.date.0, Style::default().blue().italic());
        
        let monthly = Monthly::new(
            self.date.0,
            event_store
        ).show_month_header(Modifier::BOLD)
            .show_weekdays_header(Modifier::ITALIC);
        drawer.render_widget(monthly, drawer.area);
    }
}