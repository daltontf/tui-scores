## TUI-SCORES 

Application to display sports score data in a responsive TUI (Text User Interface). 

An exercise in "kicking the tires" with Ratatui-Kit [https://yexiyue.github.io/ratatui-kit/]

### Quirks

- All sports list the home team second which is against the convention used by soccer.
- Currently lagging when displaying large amounts of events like some days of NCAA sports. This issue could be a Ratatui Kit problem. 
- Dates where there are scheduled events are bold on the date picker only for leagues that provide a discete list of such dates. This is for the most recent retrieval meaning that if one navigates to a past season that displaying random date for the year will retreive season in progress or recently completed at the time.

### Screenshots

![Select League](./select_league.png)

![Current Day](./inprogress.png)

![Resized](./resized.png)

![Upcoming](./upcoming.png)

![Past Date](./pastdate.png)