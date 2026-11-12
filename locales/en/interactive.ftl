# Texts of the interactive (journey) findings
# (`audit::interactive_finding::interactive_finding_text`, #936): per finding
# kind one message and one `-fix` message for the fix suggestion, plus the
# uncertainty note (`finding_uncertainty_text`). Which kind maps to which
# message, the default values, the quoted page title and the truncation
# marker of the example list stay in Rust.

interactive-hidden-focusable-aria-hidden = Keyboard focus lands on an element inside an aria-hidden region ({ $selector }). Screen reader users reach an element that is hidden from the accessibility tree.
interactive-hidden-focusable-aria-hidden-fix = Remove the element from the aria-hidden region or set tabindex="-1" on it.
interactive-hidden-focusable-inert = Keyboard focus lands on an element inside an inert region ({ $selector }). Inert regions should not be reachable by keyboard.
interactive-hidden-focusable-inert-fix = Remove the element from the inert region or correct the tabindex/focus chain.
interactive-hidden-focusable-style = Keyboard focus lands on a visually hidden element ({ $selector }: display:none, visibility:hidden, or opacity:0). Keyboard users lose orientation.
interactive-hidden-focusable-style-fix = Remove the element from the tab sequence (tabindex="-1") or make it visible before it receives focus.
interactive-focus-indicator-not-detected = Element ({ $selector }) shows no visible focus indicator when focused (no outline, no box-shadow, no border change). Keyboard users lose orientation.
interactive-focus-indicator-not-detected-fix = Add a CSS :focus-visible rule with a clear outline, box-shadow, or border change compared to the unfocused state.
interactive-tab-order-backward-jumps =
    Tab order deviates from DOM order: { $count } backward { $count ->
        [one] jump
       *[other] jumps
    } observed. First affected elements: { $examples }{ $suffix }. Keyboard users may not be able to follow the reading flow.
interactive-tab-order-backward-jumps-fix = Avoid negative or high tabindex values. Arrange the reading/DOM order to match the visual order.
interactive-focus-trap-not-entered = After opening modal, focus did not move inside the dialog. Keyboard users cannot interact with it.
interactive-focus-trap-not-entered-fix = Move focus to the first focusable element inside the dialog when it opens, or to the dialog element itself (tabindex="-1").
interactive-focus-trap-background-not-hidden = Background content is not hidden from assistive technology when modal is open.
interactive-focus-trap-background-not-hidden-fix = Set aria-hidden="true" on the application root when a modal is open, or use the inert attribute.
interactive-focus-trap-escaped = Focus is not trapped inside the modal dialog. Keyboard users can navigate to background content.
interactive-focus-trap-escaped-fix = Intercept Tab and Shift+Tab inside the dialog to cycle focus among dialog descendants only.
interactive-focus-trap-escape-not-closing = Escape key does not close the modal. Keyboard users cannot dismiss it.
interactive-focus-trap-escape-not-closing-fix = Add a keydown handler on the dialog or document that calls close() or hides the dialog when Escape is pressed.
interactive-focus-restoration-lost-to-body = After closing the modal, focus returned to body instead of the trigger. Keyboard users lose their place on the page.
interactive-focus-restoration-lost-to-body-fix = Store a reference to the trigger element before opening the dialog and call trigger.focus() when the dialog closes.
interactive-modal-not-opened = The trigger announces a dialog via aria-haspopup, but activating it makes no dialog appear in the accessibility tree.
interactive-modal-not-opened-fix = Either open a dialog on activation, or drop aria-haspopup="dialog" from the trigger so it does not promise something it does not do.
interactive-menu-not-opened = Menu trigger was clicked but menu did not open. Keyboard users cannot access menu items.
interactive-menu-not-opened-fix = Set aria-expanded="true" on the trigger and make the menu items visible when the trigger is activated.
interactive-menu-focus-not-moved = After opening menu, focus did not move to menu items. Keyboard users may not know the menu opened.
interactive-menu-focus-not-moved-fix = Move focus to the first menu item after the menu opens.
interactive-menu-escape-not-closing = Escape key does not close the menu.
interactive-menu-escape-not-closing-fix = Add a keydown handler that closes the menu and returns focus to the trigger when Escape is pressed.
interactive-tabs-selection-not-moved = Arrow key navigation does not move selection between tabs. Keyboard users cannot navigate the tab list.
interactive-tabs-selection-not-moved-fix = Implement the roving tabindex pattern: ArrowRight moves focus and aria-selected to the next tab.
interactive-tabs-focus-not-on-tab = After pressing ArrowRight in the tab list, focus is not on a tab element.
interactive-tabs-focus-not-on-tab-fix = Ensure arrow key navigation also moves focus (not just selection) to the next tab in the roving tabindex pattern.
interactive-disclosure-not-opened = Activating the disclosure trigger does not expand it: neither its own expanded state nor the accessibility tree changes.
interactive-disclosure-not-opened-fix = Toggle aria-expanded="true|false" on the activated element and reveal the controlled region.
interactive-disclosure-not-closed = Activating the disclosure trigger does not collapse it: neither its own expanded state nor the accessibility tree changes.
interactive-disclosure-not-closed-fix = Make the activation handler toggle aria-expanded in both directions, not just open.
interactive-disclosure-state-without-content = The disclosure trigger reports its new expanded state, but nothing enters or leaves the accessibility tree. Screen reader users hear the state change and find nothing to read.
interactive-disclosure-state-without-content-fix = Check that the controlled region really enters the accessibility tree — display:none, hidden or aria-hidden on the region keep it out.
interactive-disclosure-content-without-state = Activating the trigger changes the accessibility tree, but the trigger itself reports no expanded state. Screen reader users are not told that something opened or closed.
interactive-disclosure-content-without-state-fix = Put aria-expanded on the element that is actually activated and toggle it together with the region.
interactive-spa-no-announcement-signal = After SPA navigation neither the page title (before: { $title }) nor the H1 heading changed, and focus remained in the same place. Screen readers will not announce the new content.
interactive-spa-no-announcement-signal-fix = After each client-side navigation: (1) update document.title, (2) move focus to the <main> element or the new H1 heading, (3) alternatively populate an aria-live region with the new page name.
interactive-spa-title-unchanged = After SPA navigation document.title remains unchanged ({ $title }). Screen readers often primarily announce page transitions via the title.
interactive-spa-title-unchanged-fix = Update document.title to the new page name after every client-side navigation.
interactive-spa-focus-not-moved = After SPA navigation focus is not moved to the new main area. Keyboard users must manually navigate to the new content.
interactive-spa-focus-not-moved-fix = After navigation, move focus to the <main> element or the first H1 heading of the new content.
interactive-skip-link-focus-not-moved = Skip link is present but does not move focus to the target. Keyboard users cannot bypass navigation.
interactive-skip-link-focus-not-moved-fix = Ensure the skip link target has tabindex="-1" and receives focus via an anchor link, or explicitly call target.focus() after navigation.
interactive-form-error-silent-failure = Form errors are not announced via a live region (role="alert" or aria-live) and aria-invalid is not set. Screen reader users receive no feedback when a required field is left empty.
interactive-form-error-silent-failure-fix = Output error messages inside a role="alert" element and set aria-invalid="true" on each invalid field.
interactive-form-error-invalid-without-live-region = aria-invalid is set after submission, but no live region (role="alert" or aria-live) announces the error. Screen reader users will only notice the error state when they explicitly navigate back to the field.
interactive-form-error-invalid-without-live-region-fix = Add a role="alert" container that outputs the error message after form submission.
interactive-form-error-unlinked-fields =
    { $count } { $count ->
        [one] field
       *[other] fields
    } with aria-invalid="true" are not linked to their error message via aria-describedby or aria-errormessage. Screen reader users hear the error state but cannot associate it with the field.
interactive-form-error-unlinked-fields-fix = Add aria-describedby="error-message-id" on each field with aria-invalid="true".
interactive-form-error-focus-not-managed = An error was announced after submission, but keyboard focus stayed on the document body instead of moving to the first invalid field or an error summary. Screen reader and keyboard users are not led to the error and must search for it manually.
interactive-form-error-focus-not-managed-fix = Move focus to the first invalid field or to an error summary (e.g. role="alert") after a failed submission.
interactive-form-error-live-region-late-insertion = The live region (role="alert" or aria-live) announcing the error was not present on initial page load — it was only inserted into the DOM after form submission. Screen readers register live regions when the accessibility tree is first built; some browser/assistive-technology combinations do not reliably announce a live region that appears only after the page has already loaded.
interactive-form-error-live-region-late-insertion-fix = Render the live-region container empty in the initial markup (e.g. <div role="status"></div>) and only fill its text content on submission, instead of inserting the container itself after the interaction.
interactive-add-to-cart-no-status-announcement = Adding the item to the cart visibly changed the page (e.g. a cart counter), but no live region (role="status"/"alert" or aria-live) announced it and focus did not move into a cart dialog. Screen reader users receive no confirmation that the item was added.
interactive-add-to-cart-no-status-announcement-fix = Announce the outcome via a role="status" live region (e.g. "Item added to cart"), or move focus into the cart drawer/dialog when it opens.
interactive-add-to-cart-no-feedback-detected = Clicking the add-to-cart trigger produced no detectable change: no live-region announcement, no focus change into a dialog, and no recognizable cart-counter update. Manual review is needed to confirm whether the action succeeded and how it is communicated.
interactive-add-to-cart-no-feedback-detected-fix = Verify manually that adding to cart succeeds and is announced (e.g. via a role="status" live region or a focus-managed cart dialog).
interactive-quantity-stepper-keyboard-inoperable = The quantity control could not be operated by keyboard: it either could not receive focus, or its value did not change after pressing Arrow Up. Keyboard-only users cannot adjust the quantity.
interactive-quantity-stepper-keyboard-inoperable-fix = Ensure the quantity control is a native <input type="number"> or a fully keyboard-operable ARIA spinbutton (focusable, responds to Arrow Up/Down).
interactive-quantity-stepper-value-not-exposed = Arrow Up changed the quantity value, but this custom (non-native) spinbutton widget did not update aria-valuenow to match. Screen reader users hear no change even though the value did change.
interactive-quantity-stepper-value-not-exposed-fix = Update aria-valuenow (and ideally aria-valuetext) on the spinbutton element whenever its value changes.
interactive-link-text-generic =
    { $count } { $count ->
        [one] link
       *[other] links
    } carry generic or non-descriptive text ({ $examples }). Without surrounding context they are hard to tell apart for screen reader users. Heuristic warning on WCAG 2.4.4 — whether the link purpose is clear from its context needs a human check.
interactive-link-text-generic-fix = Write link text that is meaningful without the surrounding page context, e.g. 'Learn more about accessibility' instead of 'Learn more'.
interactive-link-text-duplicate =
    { $count } { $count ->
        [one] link text
       *[other] link texts
    } appear 3 or more times on the page: { $examples }. If they point to different targets, screen reader users cannot distinguish them.
interactive-link-text-duplicate-fix = Replace repeated link texts with unique wording or supplement the visible text with aria-label / aria-labelledby.
interactive-heading-missing-h1 = The page has no H1 heading. Screen reader users cannot identify the main structure of the page without an H1.
interactive-heading-missing-h1-fix = Use exactly one H1 heading per page that describes the main content.
interactive-heading-multiple-h1 = { $count } H1 headings found. That is allowed; it can blur orientation when it is unclear which one names the page topic.
interactive-heading-multiple-h1-fix = Check the heading structure; one H1 for the page topic is recommended.
interactive-heading-level-skip = Heading hierarchy skips levels ({ $examples }). Screen reader users may not be able to reliably parse the page structure.
interactive-heading-level-skip-fix = Never skip heading levels. After H1 comes H2, after H2 comes H3, and so on.
interactive-landmark-missing-main = No <main> landmark found. Screen reader users cannot jump directly to the main content.
interactive-landmark-missing-main-fix = Wrap the main content in a <main> element or set role="main" on the appropriate container.
interactive-landmark-nav-without-labels = { $count } navigation landmarks without distinct labels. Screen reader users cannot tell which navigation covers which area.
interactive-landmark-nav-without-labels-fix = Label each <nav> region with an aria-label, e.g. aria-label="Main navigation" and aria-label="Footer navigation".
interactive-landmark-duplicate-unique = Landmark role "{ $role }" appears { $count }× on the page. This role should only occur once per page.
interactive-landmark-duplicate-unique-fix = Use only one element with role="{ $role }" (or the corresponding HTML element) per page.
interactive-media-controls-missing-name = Native video player ({ $selector }) is reachable via keyboard but has no accessible name (no aria-label, aria-labelledby, or title). Screen reader users cannot identify which video the control refers to.
interactive-media-controls-missing-name-fix = Add an aria-label, aria-labelledby, or title attribute to the video element describing its content.
interactive-media-controls-not-reachable =
    { $count } native video { $count ->
        [one] element
       *[other] elements
    } with visible controls could not be reached via keyboard Tab navigation: { $examples }{ $suffix }. Keyboard users may be unable to operate playback.
interactive-media-controls-not-reachable-fix = Verify no earlier element traps focus and that the video element itself is not removed from the tab sequence (tabindex="-1").

interactive-uncertainty-late-reaction = Uncertain: the page settled { $waited_ms } ms after the click; a slower reaction would not have been observed.
