# docs/ui/visual-design.md

**Badges:** `🚫 No Implementation Code` `🚫 No C/C++ Dependencies`

> Defines visual design guidelines for HoneyLink™. Describes color palette, typography, iconography, and branding without implementation code or C/C++ dependencies.

## Table of Contents
- [Visual Design Goals](#visual-design-goals)
- [Color Palette](#color-palette)
- [Typography](#typography)
- [Iconography](#iconography)
- [Spacing and Layout](#spacing-and-layout)
- [Elevation and Shadows](#elevation-and-shadows)
- [Branding](#branding)
- [Acceptance Criteria (DoD)](#acceptance-criteria-dod)

## Visual Design Goals
- **Modern:** Clean, minimalist aesthetic aligned with contemporary design trends.
- **Professional:** Convey trust, reliability, and technical sophistication.
- **Accessible:** Meet WCAG 2.1 AA contrast requirements.
- **Consistent:** Unified visual language across all interfaces.
- **Scalable:** Design system adaptable to future products and features.

## Color Palette
### Primary Colors
| Color | Hex | Usage |
|-------|-----|-------|
| Primary | `#0066CC` | Buttons, links, active states |
| Primary Dark | `#004C99` | Hover states, emphasis |
| Primary Light | `#3385D6` | Backgrounds, highlights |

### Secondary Colors
| Color | Hex | Usage |
|-------|-----|-------|
| Secondary | `#6C757D` | Secondary actions, borders |
| Secondary Dark | `#495057` | Hover states |
| Secondary Light | `#ADB5BD` | Disabled states |

### Semantic Colors
| Color | Hex | Usage |
|-------|-----|-------|
| Success | `#28A745` | Success messages, confirmations |
| Warning | `#FFC107` | Warnings, cautions |
| Error | `#DC3545` | Errors, destructive actions |
| Info | `#17A2B8` | Informational messages |

### Neutral Colors
| Color | Hex | Usage |
|-------|-----|-------|
| Gray 900 | `#212529` | Text, headings |
| Gray 700 | `#495057` | Secondary text |
| Gray 500 | `#6C757D` | Placeholders, disabled text |
| Gray 300 | `#DEE2E6` | Borders, dividers |
| Gray 100 | `#F8F9FA` | Backgrounds, cards |
| White | `#FFFFFF` | Page background, text on dark |

### Accessibility
- All text colors meet 4.5:1 contrast ratio on backgrounds.
- Interactive elements meet 3:1 contrast ratio.

## Typography
### Font Families
- **Primary:** `Inter`, fallback: `system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`
- **Monospace:** `JetBrains Mono`, fallback: `"Courier New", Courier, monospace`

### Type Scale
| Element | Size | Weight | Line Height |
|---------|------|--------|-------------|
| H1 | 32px / 2rem | 700 | 1.2 |
| H2 | 28px / 1.75rem | 600 | 1.3 |
| H3 | 24px / 1.5rem | 600 | 1.4 |
| H4 | 20px / 1.25rem | 600 | 1.5 |
| Body | 16px / 1rem | 400 | 1.5 |
| Small | 14px / 0.875rem | 400 | 1.4 |
| Caption | 12px / 0.75rem | 400 | 1.3 |

### Font Weights
- Regular: 400
- Medium: 500
- Semibold: 600
- Bold: 700

## Iconography
- **Library:** Material Icons, Feather Icons, or custom SVG icon set.
- **Style:** Outlined, 24x24px base size, 2px stroke width.
- **Colors:** Match text color or use primary/semantic colors for emphasis.
- **Accessibility:** Icons accompanied by text labels or `aria-label` attributes.
- **Usage:**
  - Navigation: Consistent icons for common actions (home, settings, search)
  - Status: Check, warning, error, info icons
  - Actions: Edit, delete, download, upload icons

## Spacing and Layout
### Spacing Scale (8px Grid)
| Token | Value | Usage |
|-------|-------|-------|
| xs | 4px | Tight spacing, inline elements |
| sm | 8px | Compact spacing, form fields |
| md | 16px | Default spacing, cards |
| lg | 24px | Section spacing |
| xl | 32px | Page margins |
| 2xl | 48px | Large separations |

### Layout Grid
- **Columns:** 12-column grid
- **Gutter:** 16px (mobile), 24px (tablet/desktop)
- **Margins:** 16px (mobile), 32px (desktop)

## Elevation and Shadows
| Level | Shadow | Usage |
|-------|--------|-------|
| 0 | None | Flat elements, backgrounds |
| 1 | `0 1px 3px rgba(0,0,0,0.12)` | Cards, panels |
| 2 | `0 3px 6px rgba(0,0,0,0.16)` | Raised buttons, dropdowns |
| 3 | `0 10px 20px rgba(0,0,0,0.19)` | Modals, popovers |
| 4 | `0 14px 28px rgba(0,0,0,0.25)` | Navigation drawer |

## Branding
- **Logo:** HoneyLink™ wordmark + icon. Provided in SVG, PNG (1x, 2x, 3x).
- **Logo Usage:**
  - Minimum clear space: Logo height / 2 on all sides
  - Minimum size: 120px width (horizontal), 40px height (stacked)
  - Do not distort, rotate, or recolor outside brand guidelines
- **Tagline:** "Next-Gen Wireless Protocol"
- **Brand Voice:** Technical, authoritative, innovative, approachable

## Acceptance Criteria (DoD)
- Visual design goals and color palette documented with hex values.
- Typography scale, font families, and weights specified.
- Iconography style and usage guidelines defined.
- Spacing scale, layout grid, and elevation levels described.
- Branding guidelines and logo usage rules specified.
- C/C++ dependency exclusion explicitly stated.
- Links to related documents consistent.
