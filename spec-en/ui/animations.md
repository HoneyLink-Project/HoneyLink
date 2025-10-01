# docs/ui/animations.md

**Badges:** `🚫 No Implementation Code` `🚫 No C/C++ Dependencies`

> Defines UI animation guidelines for HoneyLink™. Describes types, durations, easing, and accessibility considerations without implementation code or C/C++ dependencies.

## Table of Contents
- [Animation Goals](#animation-goals)
- [Animation Types](#animation-types)
- [Timing and Duration](#timing-and-duration)
- [Easing Functions](#easing-functions)
- [Accessibility Considerations](#accessibility-considerations)
- [Performance Guidelines](#performance-guidelines)
- [Acceptance Criteria (DoD)](#acceptance-criteria-dod)

## Animation Goals
- **Enhance UX:** Guide user attention, provide feedback, indicate state changes.
- **Consistency:** Unified animation language across all UI components.
- **Performance:** Smooth 60fps animations. No janky or sluggish motion.
- **Accessibility:** Respect `prefers-reduced-motion` setting. Provide alternatives.

## Animation Types
| Type | Purpose | Examples |
|------|---------|----------|
| Feedback | Confirm user actions | Button press, toggle switch |
| Transition | Smooth state changes | Page transitions, expand/collapse |
| Loading | Indicate background activity | Spinners, skeleton screens |
| Decorative | Visual polish | Hover effects, micro-interactions |
| Narrative | Guide user through flow | Onboarding tooltips, progress indicators |

## Timing and Duration
| Animation Type | Duration | Rationale |
|----------------|----------|-----------|
| Micro-interactions | 100-200ms | Quick feedback without delay |
| Transitions | 200-400ms | Smooth state change without sluggishness |
| Page transitions | 300-500ms | Sufficient time for context change |
| Loading indicators | Continuous | Until task completion |
| Decorative | 150-300ms | Subtle enhancement |

- **General Rule:** Shorter durations for small elements, longer for full-screen transitions.
- **Consistency:** Use standardized duration tokens (e.g., `duration-fast: 150ms`, `duration-normal: 300ms`, `duration-slow: 500ms`).

## Easing Functions
| Easing | Use Case | CSS Value |
|--------|----------|-----------|
| Ease-out | Elements entering screen | `cubic-bezier(0, 0, 0.2, 1)` |
| Ease-in | Elements exiting screen | `cubic-bezier(0.4, 0, 1, 1)` |
| Ease-in-out | Bi-directional motion | `cubic-bezier(0.4, 0, 0.2, 1)` |
| Linear | Loading indicators, progress bars | `linear` |
| Spring | Playful, energetic motion | Custom spring function |

- **Material Design Easing:** Preferred for consistency with modern design systems.
- **Avoid:** Overly complex easing that feels unnatural or disorienting.

## Accessibility Considerations
### `prefers-reduced-motion`
- Detect user preference via CSS media query: `@media (prefers-reduced-motion: reduce)`.
- **Reduced Motion Mode:**
  - Disable decorative animations.
  - Replace transitions with instant state changes or simple fades.
  - Keep essential feedback animations (e.g., loading indicators) but simplify.
- **Implementation:**
  ```css
  @media (prefers-reduced-motion: reduce) {
    * {
      animation-duration: 0.01ms !important;
      transition-duration: 0.01ms !important;
    }
  }
  ```

### Vestibular Disorders
- Avoid rapid motion, parallax scrolling, or spinning animations that may trigger discomfort.
- Provide user toggle to disable animations in settings.

## Performance Guidelines
- **GPU Acceleration:** Animate `transform` and `opacity` properties (hardware-accelerated).
- **Avoid:** Animating `width`, `height`, `top`, `left` (triggers layout reflow).
- **60fps Target:** Use browser DevTools Performance panel to profile. Ensure no dropped frames.
- **Throttling:** Test on low-end devices and slow networks. Adjust complexity as needed.
- **JavaScript Animations:** Prefer CSS transitions/animations. Use JS only for complex, interactive animations (e.g., gesture-driven).

## Acceptance Criteria (DoD)
- Animation goals, types, and use cases documented.
- Timing, duration, and easing function guidelines defined.
- Accessibility considerations (`prefers-reduced-motion`, vestibular disorders) specified.
- Performance guidelines (GPU acceleration, 60fps target) described.
- C/C++ dependency exclusion explicitly stated.
- Links to related documents consistent.
