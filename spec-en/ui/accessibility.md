# docs/ui/accessibility.md

**Badges:** `🚫 No Implementation Code` `🚫 No C/C++ Dependencies`

> Defines accessibility guidelines for HoneyLink™ UI. Ensures WCAG 2.1 AA compliance without implementation code or C/C++ dependencies.

## Table of Contents
- [Accessibility Goals](#accessibility-goals)
- [WCAG 2.1 Compliance](#wcag-21-compliance)
- [Keyboard Navigation](#keyboard-navigation)
- [Screen Reader Support](#screen-reader-support)
- [Color and Contrast](#color-and-contrast)
- [Focus Management](#focus-management)
- [ARIA Attributes](#aria-attributes)
- [Testing and Validation](#testing-and-validation)
- [Acceptance Criteria (DoD)](#acceptance-criteria-dod)

## Accessibility Goals
- Achieve WCAG 2.1 Level AA compliance for all user-facing interfaces.
- Support keyboard-only navigation for all features.
- Ensure screen reader compatibility (NVDA, JAWS, VoiceOver).
- Provide accessible alternatives for visual/audio content.
- Maintain accessibility across all supported browsers and devices.

## WCAG 2.1 Compliance
| Principle | Requirements | Implementation |
|-----------|--------------|----------------|
| Perceivable | Text alternatives, adaptable content, distinguishable | Alt text for images, semantic HTML, 4.5:1 contrast ratio |
| Operable | Keyboard accessible, enough time, navigable, input modalities | Focus indicators, skip links, no keyboard traps |
| Understandable | Readable, predictable, input assistance | Clear labels, consistent navigation, error messages |
| Robust | Compatible with assistive technologies | Valid HTML, ARIA attributes, cross-browser testing |

## Keyboard Navigation
- **Tab Order:** Logical tab order following visual layout. No keyboard traps.
- **Focus Indicators:** Visible focus outline (minimum 2px, high contrast). Custom styles allowed if meeting contrast requirements.
- **Shortcuts:**
  - `Tab` / `Shift+Tab`: Navigate forward/backward
  - `Enter` / `Space`: Activate buttons/links
  - `Esc`: Close modals/dropdowns
  - `Arrow keys`: Navigate within menus, tabs, radio groups
- **Skip Links:** "Skip to main content" link at top of page for keyboard users.

## Screen Reader Support
- **Semantic HTML:** Use native elements (`<button>`, `<nav>`, `<main>`) over generic `<div>`.
- **Landmarks:** Define regions with ARIA landmarks (`role="navigation"`, `role="main"`, etc.).
- **Live Regions:** Use `aria-live` for dynamic content updates (e.g., notifications, alerts).
- **Hidden Content:** Use `aria-hidden="true"` for decorative elements. Ensure interactive elements not hidden from screen readers.

## Color and Contrast
- **Contrast Ratios:**
  - Normal text: Minimum 4.5:1
  - Large text (18pt+ or 14pt+ bold): Minimum 3:1
  - UI components and graphics: Minimum 3:1
- **Color Blindness:** Do not rely on color alone to convey information. Use icons, patterns, labels.
- **Tools:** Validate with contrast checkers (WebAIM, Stark, browser DevTools).

## Focus Management
- **Modal Dialogs:** Trap focus within modal. Restore focus to trigger element on close.
- **Dynamic Content:** Move focus to newly loaded content when appropriate (e.g., after form submission).
- **Focus Order:** Programmatically manage focus for single-page app (SPA) route changes.

## ARIA Attributes
| Attribute | Purpose | Example |
|-----------|---------|---------|
| `aria-label` | Provides accessible name | `<button aria-label="Close dialog">×</button>` |
| `aria-labelledby` | References element(s) providing label | `<div role="dialog" aria-labelledby="dialog-title">` |
| `aria-describedby` | References element(s) providing description | `<input aria-describedby="password-help">` |
| `aria-expanded` | Indicates collapsed/expanded state | `<button aria-expanded="false">Menu</button>` |
| `aria-live` | Announces dynamic updates | `<div aria-live="polite">Status: Saved</div>` |
| `aria-hidden` | Hides decorative elements from AT | `<span aria-hidden="true">🔒</span>` |

- Use ARIA sparingly. Prefer native HTML semantics.

## Testing and Validation
- **Automated Testing:**
  - Run `axe-core` or `pa11y` in CI pipeline.
  - Block merge on accessibility violations.
- **Manual Testing:**
  - Keyboard-only navigation test.
  - Screen reader test (NVDA on Windows, VoiceOver on macOS/iOS).
  - Zoom to 200% and verify layout integrity.
- **User Testing:**
  - Include users with disabilities in usability testing.
  - Document feedback in [docs/notes/decision-log.md](../notes/decision-log.md).

## Acceptance Criteria (DoD)
- Accessibility goals and WCAG 2.1 AA compliance documented.
- Keyboard navigation, screen reader support, and focus management specified.
- Color/contrast requirements and ARIA attribute usage defined.
- Testing and validation procedures described.
- C/C++ dependency exclusion explicitly stated.
- Links to related documents consistent.
