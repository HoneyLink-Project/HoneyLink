# Stakeholder Attendance & Reminder System

**Badges:** `🚫 No Implementation Code` `🚫 No C/C++ Dependencies`

> Defines reminder and escalation mechanisms to maintain stakeholder attendance rate ≥90%.

---

## 1. Purpose and Scope

### 1.1 Purpose
- Maintain **core member attendance rate ≥90%** at regular meetings of each working group (WG)
- Establish rapid complementary review process for absences to prevent decision delays
- Clarify escalation paths for early blocker resolution

### 1.2 Scope
- Target: All 5 working groups (Architecture/Protocol/UX/Security/Operations)
- Measurement period: Monthly aggregation (review previous month results on 1st of each month)
- Applicable meetings: Weekly regular meetings defined in `spec/notes/governance.md`

---

## 2. Attendance Rate Measurement Method

### 2.1 Definition
- **Attendance Rate** = (Actual attendees / Total core members) × 100
- **Core Members**: Members listed in "Core Members (Role IDs)" column of `spec/notes/governance.md`
- **Actual Attendance**: Counted as "present" if participated in ≥80% of meeting duration
- **Pre-approved Absence**: Excluded from attendance rate calculation if notified to Chair 24 hours in advance with complementary review plan submitted

### 2.2 Recording Method
- Each WG Chair records participant list following `spec/notes/meeting-notes.md` template
- Absentees listed in "Absent:" field with reason (pre-approved/unannounced)
- Chair calculates attendance rate at end of month and reports to Metrics section of `spec/notes/governance.md`

---

## 3. Reminder System

### 3.1 Automatic Reminder Settings

#### 3.1.1 Pre-Meeting Reminders
| Timing | Recipient | Channel | Content |
|--------|-----------|---------|---------|
| 48h before meeting | All core members | Slack + Email | Meeting date/time, agenda, pre-reading links |
| 24h before meeting | Non-respondents only | Slack DM | Attendance confirmation, complementary review procedure guidance for absences |
| 2h before meeting | All core members | Slack `#<wg-channel>` | Final reminder, Zoom/Meet link |

#### 3.1.2 Reminder Implementation Policy
- **Tool**: Slack Workflow Builder or external scheduler (GitHub Actions + Slack API)
- **Setup Responsible**: Each WG Chair (Operations WG maintains after initial setup)
- **Template**:
  ```
  📅 [HoneyLink WG] <WG Name> Regular Meeting Reminder
  
  Date/Time: <YYYY-MM-DD HH:MM JST>
  Location: <Zoom/Meet Link>
  Agenda: <Link to spec/notes/meeting-notes.md>
  
  If absent, contact @<Chair Role ID> 24 hours in advance.
  Complementary Review Form: <Link>
  ```

### 3.2 Absence Response Flow

#### 3.2.1 Pre-approved Absence
1. Member notifies Chair via Slack DM 24 hours before absence
2. Submit complementary review plan (see Section 4.1)
3. Chair approves and records "pre-approved absence" in meeting minutes

#### 3.2.2 Unannounced Absence
1. Chair contacts absentee via Slack DM within 1 hour after meeting ends
2. Confirm absence reason and request complementary review completion within 48 hours
3. If no response after 48 hours, escalate per Section 5

---

## 4. Complementary Review Process

### 4.1 Complementary Review Plan Template
Absentees submit following information to Chair:

```markdown
## Complementary Review Plan

- **Name/Role ID**: <Fill in>
- **Target Meeting**: <YYYY-MM-DD WG Name>
- **Absence Reason**: <Brief>
- **Complementary Review Method**: 
  - [ ] Review minutes within 6 hours after meeting and post comments to Slack thread
  - [ ] Report approval/objection to decisions to Chair individually (within 48 hours)
  - [ ] Conduct 1-on-1 with Chair or proxy member if needed (within 30 minutes)
- **Completion Target Date/Time**: <YYYY-MM-DD HH:MM>
```

### 4.2 Complementary Review Completion Criteria
- Comment posted to minutes or individual report to Chair completed
- Feedback submitted to Decision Log decisions
- Accept/reject explicitly stated for action items under own responsibility

### 4.3 Recording
- After complementary review completion, Chair adds "Complementary Review Completed" note to corresponding entry in `spec/notes/meeting-notes.md`
- "Pre-approved absence + Complementary review completed" excluded from attendance rate calculation

---

## 5. Escalation Path

### 5.1 Escalation Triggers

| Situation | Escalation Target | Timeline | Action |
|-----------|-------------------|----------|--------|
| Monthly attendance <90% | WG Chair → Governance Council | Within 5 business days of month start | Submit improvement plan, set recovery target for next month |
| Same member unannounced absence 2 consecutive times | WG Chair → Member's supervisor | Within 24h after 2nd absence | Conduct 1-on-1, confirm participation continuation |
| Complementary review incomplete (48h exceeded) | WG Chair → Governance Council | Immediately after exceeded | Appoint proxy member, re-approval process for decisions |
| Critical decision quorum not met | WG Chair → All WG Chairs Sync | Within 2h after meeting end | Emergency Slack huddle, convene ad-hoc meeting within 48h |

### 5.2 Governance Council Response
- **Composition**: Architecture + Security + Operations WG Chairs
- **Contact Method**: Matrix secure room `!govcouncil:honeylink.local`
- **Decision Authority**: Member replacement recommendation, temporary relaxation of complementary review process, meeting cadence adjustment
- **Recording**: Record escalation content and countermeasures in `spec/notes/decision-log.md` (Status: `Escalation`)

### 5.3 Improvement Plan Template
WGs with attendance <90% submit:

```markdown
## Attendance Improvement Plan

- **Target WG**: <WG Name>
- **Target Month**: <YYYY-MM>
- **Actual Attendance Rate**: <XX.X%>
- **Root Cause Analysis**:
  - <Bullet points: Meeting time slot mismatch, unclear agenda, etc.>
- **Improvement Measures**:
  1. <Specific measure: Meeting time change, thorough pre-reading distribution, etc.>
  2. <Measure 2>
- **Target**: Achieve next month attendance rate ≥95%
- **Monitoring**: Chair reports attendance status weekly to Slack `#hl-wg-leads`
- **Approver**: Governance Council
- **Submission Date**: <YYYY-MM-DD>
```

---

## 6. Tool Setup Guide

### 6.1 Slack Workflow Builder Setup Example

#### Step 1: Create Workflow
1. Open Slack workspace "Tools" → "Workflow Builder"
2. "Create" → "Scheduled Date & Time"
3. Trigger: Every `<meeting day of week>` at `<meeting time-48h>`

#### Step 2: Message Settings
1. "Add Step" → "Send a message"
2. Destination: `#<wg-channel>` and DM to each member
3. Message: Use template from Section 3.1.1
4. Variables: Dynamically insert `{{meeting_date}}`, `{{agenda_link}}`

#### Step 3: Conditional Branch (Non-respondent Follow-up)
1. "Add Step" → "Wait for a response" (24 hours)
2. If no response → Notify Chair + resend reminder DM to member

### 6.2 GitHub Actions Automation (Optional)

`.github/workflows/meeting-reminder.yml` example:
```yaml
name: WG Meeting Reminder

on:
  schedule:
    # Tuesday 15:00 JST (06:00 UTC) -48h = Sunday 15:00 JST
    - cron: '0 6 * * 0'  # Architecture WG
    - cron: '0 1 * * 1'  # Protocol WG (Wed 10:00 -48h)
    # Configure other WGs similarly

jobs:
  send-reminder:
    runs-on: ubuntu-latest
    steps:
      - name: Send Slack Reminder
        env:
          SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK_URL }}
        run: |
          curl -X POST $SLACK_WEBHOOK_URL \
            -H 'Content-Type: application/json' \
            -d '{
              "text": "📅 [HoneyLink WG] Architecture WG Regular Meeting Reminder\nDate/Time: YYYY-MM-DD 15:00 JST\nAgenda: https://github.com/<org>/<repo>/blob/master/spec/notes/meeting-notes.md"
            }'
```

**Note**: Manage Slack Webhook URL in GitHub Secrets, never embed in code.

---

## 7. Monitoring and Reporting

### 7.1 Monthly Report Items
Operations WG aggregates and appends to `spec/notes/governance.md` by 5th of each month:

| WG | Target Month | Core Members | Actual Attendance Count | Attendance Rate | Pre-approved Absences | Unannounced Absences | Complementary Review Completion Rate |
|----|--------------|--------------|------------------------|----------------|----------------------|---------------------|-------------------------------------|
| Architecture | MM-YYYY | 4 | 15/16 | 93.8% | 1 | 0 | 100% |
| Protocol | MM-YYYY | 4 | 14/16 | 87.5% | 1 | 1 | 50% |
| ... | ... | ... | ... | ... | ... | ... | ... |

### 7.2 Dashboard (Optional)
- **Tool**: Grafana + TimescaleDB or Google Sheets
- **Data Source**: Auto-aggregate from each WG minutes (Operations WG runs monthly script)
- **Visualization Items**: 
  - Monthly attendance rate trend (line chart)
  - WG-by-WG attendance rate comparison (bar chart)
  - Unannounced absentee ranking (for prioritizing improvement measures)

---

## 8. Success Criteria (DoD)

- [ ] Slack Workflow or equivalent automatic reminders set up for all 5 WGs
- [ ] Complementary review process template placed in `spec/notes/` and accessible to all members
- [ ] Escalation path specified in `spec/notes/governance.md` with up-to-date Governance Council contacts
- [ ] First monthly report (Month YYYY) generated by 5th of following month and achieved ≥90% attendance
- [ ] Chair training materials (this document + FAQ) completed and all WG Chairs confirmed review

---

## 9. FAQ

**Q1: What to do for emergency absences like sick leave?**  
A1: Contact Chair via Slack DM before meeting start if possible. If complementary review completed within 48 hours after meeting, treated equivalent to "pre-approved absence."

**Q2: What about members in different timezones?**  
A2: Meeting time considers availability of all members during review cadence formulation in `spec/notes/governance.md`. If participation genuinely difficult, consult Chair to approve recorded viewing + asynchronous review.

**Q3: Are there penalties for falling below 90% attendance?**  
A3: Not a penalty, but improvement plan submission and Governance Council support to recover next month. If unmet for 3 consecutive months, consider restructuring member composition.

**Q4: Are guest participants included in attendance rate?**  
A4: No. Only core members targeted. However, if guest's continued participation is beneficial, Chair can propose guest promotion to core member to Governance Council.

---

## 10. Change History

| Version | Date | Changes | Approver |
|---------|------|---------|----------|
| 1.0 | YYYY-MM-DD | Initial version | Governance Council |

---

**Related Documents**:
- [spec/notes/governance.md](governance.md) - WG composition and weekly cadence
- [spec/notes/meeting-notes.md](meeting-notes.md) - Meeting minutes template
- [spec/notes/decision-log.md](decision-log.md) - Decision record destination
- [spec/roadmap.md](../roadmap.md) - Alignment with measurement metrics (SLI/SLO)
