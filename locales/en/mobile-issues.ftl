# Texts of the mobile findings (`mobile::mobile_issue_text`, #936): one
# message per finding kind, the parts of the touch-target sentence and the
# touch-target context labels (`mobile::mobile_context_label`). Which findings
# exist, their severity and the number format stay in Rust.

mobile-missing-viewport = Missing viewport meta tag
mobile-improper-viewport = Viewport is not properly configured
mobile-not-scalable = Page disables zooming (user-scalable=no)
mobile-small-targets = { $count } touch targets are too small (<44x44px){ $contexts }{ $samples }
mobile-small-targets-contexts = {" "}({ $list })
mobile-small-targets-samples = {" "}— e.g. { $list }
mobile-small-fonts = Smallest font size is { $size }px (recommended: ≥12px)
mobile-horizontal-scroll = Page has horizontal scrolling

mobile-context-navigation = navigation
mobile-context-footer = footer
mobile-context-header = header
mobile-context-sidebar = sidebar
mobile-context-social-utility = social/utility
mobile-context-form = form
mobile-context-button = button
mobile-context-other = other
