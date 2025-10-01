# docs/ui/wireframes.md

**Badges:** `🚫 No Implementation Code` `🚫 No C/C++ Dependencies`

> Defines UI wireframes for HoneyLink™. Describes screen flows and interaction patterns without implementation code or C/C++ dependencies.

## Table of Contents
- [Wireframe Purpose](#wireframe-purpose)
- [Key Screens](#key-screens)
- [User Flows](#user-flows)
- [Interaction Patterns](#interaction-patterns)
- [Wireframe Tools](#wireframe-tools)
- [Acceptance Criteria (DoD)](#acceptance-criteria-dod)

## Wireframe Purpose
- Visualize information architecture and layout before high-fidelity design.
- Facilitate early feedback from stakeholders and users.
- Define screen-to-screen navigation and user flows.
- Serve as blueprint for developers during implementation.

## Key Screens
### Dashboard (Home)
- **Layout:** AppBar + Sidebar + Main content area
- **Widgets:**
  - Active Devices count (card with trend chart)
  - Alert summary (critical/warning counts)
  - Recent activity feed
  - QoS metrics (latency, throughput gauges)
- **Actions:** Refresh, filter by device group, export report

### Device Management
- **List View:**
  - Table: Device ID, Name, Status, Connection Quality, Last Seen
  - Filters: Status (online/offline), Device Type, Tags
  - Bulk actions: Delete, Update firmware, Apply policy
- **Detail View:**
  - Device metadata (name, ID, firmware version, IP address)
  - Connection status and history chart
  - Logs and telemetry stream (real-time)
  - Actions: Restart, Update firmware, Edit settings

### Policy Configuration
- **Policy List:**
  - Cards: Policy name, version, applied devices count
  - Actions: Create new, Edit, Delete, Duplicate
- **Policy Editor:**
  - Form: Policy name, description, rules (JSON/YAML editor or form builder)
  - Preview: Affected devices list
  - Validation: Real-time syntax check, conflict detection
  - Actions: Save draft, Publish, Rollback to previous version

### User Settings
- **Profile:** Name, email, avatar, language preference
- **Security:** Change password, enable 2FA, active sessions
- **Preferences:** Theme (light/dark), timezone, notification settings
- **Accessibility:** Enable reduced motion, font size adjustment

### Login / Onboarding
- **Login:** Email/password form, SSO options, "Forgot password" link
- **Onboarding Wizard:**
  - Step 1: Welcome + product intro
  - Step 2: Connect first device (pairing instructions)
  - Step 3: Create first policy
  - Step 4: Dashboard tour

## User Flows
### Device Onboarding Flow
```
[Unbox device] → [Scan QR code / Enter pairing code] → [Confirm device identity]
  → [Select policy] → [Activate device] → [Dashboard confirmation]
```

### Alert Response Flow
```
[Receive alert notification] → [View alert details in dashboard]
  → [Investigate (view logs, telemetry)] → [Take action (restart device, adjust policy)]
  → [Mark alert as resolved] → [Add notes for audit]
```

### OTA Firmware Update Flow
```
[Select devices] → [Choose firmware version] → [Configure rollout (staged/immediate)]
  → [Review and confirm] → [Monitor progress] → [Verify completion / Rollback if failed]
```

## Interaction Patterns
| Pattern | Usage | Example |
|---------|-------|---------|
| Progressive Disclosure | Hide advanced options by default | "Show advanced settings" expandable section |
| Confirmation Dialog | Prevent accidental destructive actions | "Delete device? This cannot be undone." |
| Inline Editing | Quick edits without full form | Click device name to edit inline |
| Drag and Drop | Reorder priority, assign devices | Drag devices to policy groups |
| Real-time Updates | Live telemetry, status changes | WebSocket-based live log stream |
| Contextual Help | Tooltips, inline hints | "?" icon next to complex fields |
| Undo/Redo | Allow reverting recent changes | Undo policy change within 5 min |

## Wireframe Tools
- **Low-Fidelity:** Balsamiq, Whimsical, hand-drawn sketches
- **High-Fidelity:** Figma, Sketch, Adobe XD
- **Prototyping:** Figma interactive prototypes, InVision
- **Collaboration:** Share links for async feedback, conduct live design reviews
- **Handoff:** Export assets (SVG icons, images), generate CSS specs, Zeplin/Figma inspect

## Acceptance Criteria (DoD)
- Wireframe purpose and key screens documented.
- User flows for critical journeys described.
- Interaction patterns defined with examples.
- Wireframe tools and collaboration methods specified.
- C/C++ dependency exclusion explicitly stated.
- Links to related documents consistent.
