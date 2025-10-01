# docs/templates/ui-template.md

**Badges:** `🚫 No Implementation Code` `🚫 No C/C++ Dependencies`

> Template for HoneyLink™ UI specification. Describes screen requirements, interactions, and accessibility in a unified format.

---

## 1. Screen Overview
- **Screen Name:** <!-- e.g., HoneyLink Dashboard -->
- **Stakeholder:** <!-- e.g., NOC Operator -->
- **Purpose:** <!-- Goal user aims to achieve -->
- **Related Epic:** <!-- Jira/Epic ID -->

## 2. User Stories
| ID | User | Action | Expected Value |
|----|------|--------|----------------|
| UI-001 | | | |
| UI-002 | | | |

## 3. Information Architecture
- **Main Content Areas:** <!-- Cards/Tables/Charts, etc. -->
- **Information Hierarchy:**
  1. <!-- Primary -->
  2. <!-- Secondary -->
  3. <!-- Tertiary -->
- Reference: [docs/ui/overview.md](../ui/overview.md)

## 4. Layout Specification
- **Breakpoints:** <!-- Mobile/Tablet/Desktop -->
- **Grid:** <!-- e.g., 12 columns, 16px gutter -->
- **Key Components:**
  - <!-- Cards, Filters, Timeline, etc. -->
- Wireframes: [docs/ui/wireframes.md](../ui/wireframes.md)

## 5. Visual Design
- **Colors:** <!-- Primary colors, meanings -->
- **Typography:** <!-- Headings/Body/Numeric fonts -->
- **Icons:** <!-- Library used, meanings -->
- Reference: [docs/ui/visual-design.md](../ui/visual-design.md)

## 6. Interaction & Animation
- **Trigger:** <!-- Click/Hover/Scroll -->
- **Response:** <!-- Animation/Transition -->
- **Reduced Motion:** <!-- Alternative behavior -->
- Details: [docs/ui/animations.md](../ui/animations.md)

## 7. Accessibility
- **WCAG Compliance:** <!-- Level AA, etc. -->
- **Keyboard Operation:** <!-- Focus order/Shortcuts -->
- **Assistive Technology:** <!-- Screen reader announcements -->
- Reference: [docs/ui/accessibility.md](../ui/accessibility.md)

## 8. State Management
| State | Condition | Expression | Notes |
|-------|-----------|------------|-------|
| Initial | | | |
| Loading | | | |
| Error | | | |
| Empty Data | | | |

## 9. Metrics and Testing
- **UX KPI:** <!-- Task Success Rate, Time-on-Task, etc. -->
- **UI Bug Tolerance:** <!-- e.g., Critical UI bugs = 0 → Release blocker -->
- **Testing:** Usability testing, A/B testing, strategy ref: [docs/testing/e2e-tests.md](../testing/e2e-tests.md).

## 10. Dependencies and Risks
- **Backend API:** <!-- Dependent APIs -->
- **Design System:** <!-- Components used -->
- **Risks:** <!-- e.g., Performance/Visibility/Accessibility concerns -->

## 11. Acceptance Criteria (DoD)
- Information architecture and layout are defined.
- Visual, interaction, and accessibility specs are consistent with other documents.
- State management and testing/KPIs are documented.
- C/C++ dependency exclusion policy is stated.
- Risks and dependencies are documented.
