## TUI-SCORES 

Application to display sports score data in a responsive TUI (Text User Interface). 

An exercise in "kicking the tires" with Ratatui-Kit [https://yexiyue.github.io/ratatui-kit/]

### Quirks

- All sports list the home team second which is against the convention used by soccer.
- Currently lagging when displaying large amounts of events like some days of NCAA sports. This issue could be a Ratatui Kit problem. 

### Future Features
- The highlight dates on the date picker where there are events for the current league (Maybe). 
- When running check for existing of hidden config file in home directory. If it exists use it to provide the list of leagues. 
If it doens't create one with extensive list of leagues the user can edit and pare down.
- Maybe select events from different leagues into one view so one can follow favorite teams on one page.