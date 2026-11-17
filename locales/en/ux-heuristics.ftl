# Texts of the experimental UX heuristics (`ux::ux_heuristic_text`, #1035): one
# message per heuristic kind, and the two lines of the terminal table that carry
# them (`output::terminal`). Which heuristics exist, their severity and the
# counts stay in Rust. Advisory hints, not findings of a rule.

ux-heuristic-cookie-banner = A cookie or consent banner is on the page. Check that it does not cover the main content and that keyboard focus can reach and leave it.
ux-heuristic-dialog-not-modal = A dialog is not marked as modal (aria-modal). If it covers the page, mark it modal and keep keyboard focus inside it.
ux-heuristic-multiple-dialogs = { $count } dialogs are in the page at the same time. Check that they do not cover each other.
ux-heuristic-no-cta = No button or link with call-to-action wording was found. A page meant to lead to an action may need one.
ux-heuristic-many-ctas = { $count } buttons or links with call-to-action wording compete for attention. Check that the main action stands out.

## Terminal table lines (`output::terminal`)
ux-heuristic-terminal-none = UX hints: none
ux-heuristic-terminal-line = UX hint: { $text }
