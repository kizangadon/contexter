# Bug: Pages Deviate from Approved Wireframes

## Issues
1. **DashboardPage** is missing the TimeframeFilter in the header area (design shows `[Timeframe ▾]` next to "Dashboard" title)
2. **EfficiencyPage** shows 4 stat cards but design shows a 3x2 metric card grid layout with detailed cards for Memory Usage, Session Activity, Agent Performance, Skill Effectiveness, Token Usage, and Correlation Matrix
3. **SessionDetailPage** is missing the Resume button and overflow menu (⋮) in the header as shown in the wireframe
4. **MessageBubble** is missing turn number display (design shows "Turn 1", "Turn 2" above each turn)

## Fix
1. **DashboardPage**: Add `<TimeframeFilter>` component next to the PageHeader title
2. **EfficiencyPage**: Redesign to match the 3x2 metric card grid wireframe with detailed cards showing mini-charts/progress bars
3. **SessionDetailPage**: Add Resume button (with useResumeSession hook) and overflow menu (⋮) dropdown in the PageHeader actions area
4. **MessageBubble**: Add turn number display (e.g., "Turn 1", "Turn 2") above each message bubble
