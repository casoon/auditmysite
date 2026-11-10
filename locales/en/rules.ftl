# Rule catalogue texts: one message per rule id of `src/taxonomy/rules.rs`
# (`RULES`), in the order of that table. The message value is the title,
# the attributes carry the user impact and the technical impact.
# The key is `rule-` plus the rule id with `/` written as `-`
# (`taxonomy::rules::rule_text_key`).

## Content & alternatives

rule-image-alt = Missing alternative text on images
    .user-impact = Screen reader users receive no image information.
    .technical-impact = Non-conformant image markup.

rule-images-alt-missing = Missing alternative text on images
    .user-impact = Screen reader users receive no image information.
    .technical-impact = Non-conformant image markup.

rule-images-alt-suspicious = Alternative text probably not descriptive
    .user-impact = Screen reader users hear text that does not convey the image content.
    .technical-impact = alt attribute present but without descriptive content.

rule-video-caption = Missing captions for videos
    .user-impact = Deaf or hard-of-hearing users cannot perceive spoken content.
    .technical-impact = Missing track element for captions.

rule-media-alt = Missing media alternative (AAA)
    .user-impact = Users who cannot play video or audio receive no text alternative.
    .technical-impact = Missing transcript or audio description.

## Structure & semantics

rule-order-visual-mismatch = Visual order diverges from the reading order
    .user-impact = Screen reader users hear content in a different sequence than sighted users perceive visually.
    .technical-impact = CSS `order` changes the visual sequence while the DOM and accessibility tree stay unchanged.

rule-headings-section-without-heading = Missing section headings
    .user-impact = Users cannot jump directly to specific content.
    .technical-impact = Missing sub-headings in long content blocks.

rule-orientation-root-rotated = Restricted screen orientation
    .user-impact = Users with fixed-mount screens (e.g. wheelchair-mounted) cannot read the page.
    .technical-impact = The computed transform (matrix) or rotate of html or body turns by a quarter turn, measured in the audited viewport.

rule-orientation-content-hidden = Restricted screen orientation
    .user-impact = Users with fixed-mount screens (e.g. wheelchair-mounted) cannot read the page.
    .technical-impact = CSS media queries lock the orientation.

## Navigation & operation

rule-keyboard-interactive-not-focusable = Content not operable by keyboard
    .user-impact = Keyboard users are excluded from functionality.
    .technical-impact = Click handler without keyboard alternative.

rule-keyboard-trap = Keyboard trap — focus cannot leave the element
    .user-impact = The page becomes effectively unusable for keyboard users.
    .technical-impact = Missing focus management in modal elements.

rule-headings-none = Missing bypass navigation
    .user-impact = Keyboard users must tab through the navigation on every page change.
    .technical-impact = Missing skip link, missing HTML5 landmarks.

rule-document-title = Missing or insufficient page title
    .user-impact = Screen reader users cannot identify the page.
    .technical-impact = Empty or generic <title> tag.

rule-document-title-empty = Missing or insufficient page title
    .user-impact = Screen reader users cannot identify the page.
    .technical-impact = Empty or generic <title> tag.

rule-document-title-missing = Missing or insufficient page title
    .user-impact = Screen reader users cannot identify the page.
    .technical-impact = Empty or generic <title> tag.

rule-landmarks-not-unique = Landmarks are not uniquely named
    .user-impact = Screen reader users cannot distinguish similar regions.
    .technical-impact = Repeated landmark roles without unique aria-label/aria-labelledby values.

rule-dialog-focusable-missing = Illogical focus order
    .user-impact = Confusing navigation for keyboard users.
    .technical-impact = DOM order diverges from the visual order.

rule-links-generic-name = Unclear or generic link text
    .user-impact = Screen reader link lists show meaningless text.
    .technical-impact = Generic link text without context.

rule-links-single-character-name = Unclear or generic link text
    .user-impact = Screen reader link lists show meaningless text.
    .technical-impact = Generic link text without context.

rule-links-url-name = Unclear or generic link text
    .user-impact = Screen reader link lists show meaningless text.
    .technical-impact = Generic link text without context.

rule-focus-visible = Focus indicator not visible
    .user-impact = Keyboard users cannot see which element is focused.
    .technical-impact = CSS outline:none without an alternative.

rule-focus-obscured = Focused element entirely obscured
    .user-impact = Keyboard users tab to an element that is entirely covered by a sticky header, footer, cookie banner, or similar overlay and is therefore not visible.
    .technical-impact = Focus pass: element focused, scrolled into view with scrollIntoView({"{"}block:'nearest'{"}"}), 3×3 points sampled with elementsFromPoint; every point lies under a position: fixed/sticky element.

rule-focus-obscured-enhanced = Focused element partially obscured
    .user-impact = Keyboard users tab to an element whose visible area is partially covered by a sticky header, footer, cookie banner, or similar overlay.
    .technical-impact = Focus pass as for focus/obscured; at least one sample point lies under a position: fixed/sticky element.

rule-label-in-name-mismatch = Visible text ≠ accessible name
    .user-impact = Voice-control users cannot address the element.
    .technical-impact = aria-label diverges from the visible text.

rule-meta-refresh = Non-adjustable time limit
    .user-impact = Users need more time to enter data and may lose data through timeouts.
    .technical-impact = Missing refresh or timeout handling in the script/markup.

rule-timing-meta-refresh = Non-adjustable time limit
    .user-impact = Users need more time to enter data and may lose data through timeouts.
    .technical-impact = Missing refresh or timeout handling in the script/markup.

rule-motion-infinite-animation = Missing pause/stop/hide control
    .user-impact = Users with attention or cognitive impairments, and screen reader users, are distracted or disrupted by continuously running motion they cannot turn off.
    .technical-impact = `<marquee>` element without pause control, or an element with animation-iteration-count: infinite exceeding 5 seconds per cycle with no detectable pause control or prefers-reduced-motion support.

rule-manual-no-timing = Timing present (AAA)
    .user-impact = Users cannot complete tasks at their own pace.
    .technical-impact = Active time limits on the page.

rule-manual-interruptions = Non-suppressible interruptions (AAA)
    .user-impact = Users with cognitive impairments or screen reader users are distracted.
    .technical-impact = Unexpected dialogs or assertive live regions.

rule-manual-re-authenticating = No data preservation on re-authentication (AAA)
    .user-impact = Users must re-enter all form data.
    .technical-impact = Missing session data preservation.

rule-timeouts = No warning before inactivity timeout (AAA)
    .user-impact = Users lose unsaved data without warning.
    .technical-impact = Missing timeout warning system.

rule-navigation-location-missing = Missing location indication (AAA)
    .user-impact = Users lose orientation within nested websites.
    .technical-impact = Missing breadcrumb navigation or sitemap linkage.

rule-links-purpose-link-only = Unclear link label without context (AAA)
    .user-impact = Users must read the entire paragraph to understand a link's destination.
    .technical-impact = Generic link text such as 'read more' or 'click here'.

rule-links-new-window-unannounced = New window without notice (AAA)
    .user-impact = Users unexpectedly lose context; the back button does not lead back.
    .technical-impact = target="_blank" without a notice in the accessible name.

rule-manual-pointer-gestures = Missing alternative for pointer gestures
    .user-impact = Users with motor impairments cannot operate the functionality.
    .technical-impact = Required touch gestures without a click fallback.

rule-pointer-cancellation = Action triggered on pointer down
    .user-impact = Users may trigger actions accidentally without being able to cancel them.
    .technical-impact = Event handlers respond to mousedown or touchstart instead of click or mouseup.

rule-manual-motion-actuation = Missing alternative for motion actuation
    .user-impact = Users who cannot move the device are excluded from the functionality.
    .technical-impact = Use of device-motion APIs without UI controls.

rule-targets-size-enhanced = Insufficient click target size (AAA)
    .user-impact = Users with hand tremors or on mobile devices easily tap the wrong elements.
    .technical-impact = Links, buttons or form controls dimensioned too small in CSS.

rule-targets-size = Insufficient click target size
    .user-impact = Users with hand tremors or on mobile devices easily tap the wrong elements.
    .technical-impact = Links, buttons or form controls dimensioned too small in CSS.

## Forms & interaction

rule-forms-autocomplete-invalid = Missing input purpose identification
    .user-impact = No autofill possible, data entry more error-prone.
    .technical-impact = Missing autocomplete attributes.

rule-forms-purpose-missing = Missing input purpose identification
    .user-impact = No autofill possible, data entry more error-prone.
    .technical-impact = Missing autocomplete attributes.

rule-forms-group-name-missing = Missing form labels
    .user-impact = Users do not know what input is expected.
    .technical-impact = Missing <label> elements.

rule-forms-instructions-missing = Missing form labels
    .user-impact = Users do not know what input is expected.
    .technical-impact = Missing <label> elements.

rule-forms-label-missing = Missing form labels
    .user-impact = Users do not know what input is expected.
    .technical-impact = Missing <label> elements.

rule-forms-placeholder-as-label = Missing form labels
    .user-impact = Users do not know what input is expected.
    .technical-impact = Missing <label> elements.

rule-forms-required-unmarked = Missing form labels
    .user-impact = Users do not know what input is expected.
    .technical-impact = Missing <label> elements.

rule-forms-redundant-entry = Missing reuse of previously entered data
    .user-impact = Users must retype data they already entered, which costs time and is an extra burden especially for people with motor or cognitive impairments.
    .technical-impact = Form fields without an autocomplete attribute, prefilled value, or reuse option for already-captured values.

rule-auth-paste-blocked = Paste blocked in password or code field
    .user-impact = Users can use neither a password manager nor the clipboard. Anyone who cannot memorise passwords or transcribe them accurately, for example because of cognitive or motor impairments, fails to sign in.
    .technical-impact = A synthetic paste event is cancelled and the probe text is not in the field afterwards (a paste handler calling preventDefault() or returning false); measured in same-origin frames and open shadow roots too.

rule-auth-captcha = Review CAPTCHA in authentication form
    .user-impact = Text and puzzle CAPTCHAs lock people with cognitive impairments, dyslexia or low vision out of signing in when no alternative is offered.
    .technical-impact = Interactive CAPTCHA widget (reCAPTCHA, hCaptcha, image CAPTCHA) in a form with a password or one-time-code field.

rule-forms-no-submit = Form without submit button
    .user-impact = Keyboard and screen reader users have a harder time completing the form.
    .technical-impact = Missing submit control in the form.

rule-manual-help = Missing input help (AAA)
    .user-impact = Users do not know what format or data to enter in a field.
    .technical-impact = Missing form help system.

## Language & clarity

rule-document-lang-missing = Missing language declaration
    .user-impact = Screen readers read aloud with incorrect pronunciation.
    .technical-impact = Missing lang attribute.

rule-html-has-lang = Missing language declaration
    .user-impact = Screen readers read aloud with incorrect pronunciation.
    .technical-impact = Missing lang attribute.

rule-context-autofocus = Unexpected context change on focus
    .user-impact = Confusing page changes for keyboard users.
    .technical-impact = onfocus handlers trigger navigation.

rule-context-on-focus = Unexpected context change on focus
    .user-impact = Confusing page changes for keyboard users.
    .technical-impact = onfocus handlers trigger navigation.

rule-context-on-input = Unexpected context change on input
    .user-impact = Confusing changes while filling out forms.
    .technical-impact = Auto-submit on onchange.

rule-manual-unusual-words = Missing explanation of unusual words (AAA)
    .user-impact = Users with cognitive impairments or non-native speakers do not understand the text.
    .technical-impact = Missing glossary linkage or in-text explanation.

rule-language-part-undetermined = Check text in another language without lang
    .user-impact = Screen readers read the passage with the page language's pronunciation, which makes it hard to understand.
    .technical-impact = Heuristic language detection per text block; pages that are neither German nor English stay untested.

rule-language-part-unmarked = Check text in another language without lang
    .user-impact = Screen readers read the passage with the page language's pronunciation, which makes it hard to understand.
    .technical-impact = Heuristic language detection per text block; pages that are neither German nor English stay untested.

rule-language-abbreviation-unexpanded = Missing explanation of abbreviations (AAA)
    .user-impact = Users do not understand the content if they do not know the abbreviation.
    .technical-impact = Missing abbr tag with title attribute.

## Visual presentation / contrast

rule-contrast-text-insufficient = Insufficient color contrast
    .user-impact = Text is hard to read for people with low vision or under poor lighting conditions.
    .technical-impact = Contrast ratio below 4.5:1 for normal text.

rule-contrast-text-undetermined = Insufficient color contrast
    .user-impact = Text is hard to read for people with low vision or under poor lighting conditions.
    .technical-impact = Contrast ratio below 4.5:1 for normal text.

rule-contrast-text-enhanced = Enhanced color contrast not met (AAA)
    .user-impact = Text remains hard to read for people with substantially reduced vision.
    .technical-impact = Contrast ratio below 7:1 for normal text or 4.5:1 for large text.

rule-zoom-viewport-locked = Text not sufficiently resizable
    .user-impact = Users with low vision lose content when zooming in.
    .technical-impact = Fixed pixel values, overflow:hidden.

rule-zoom-viewport-missing = Missing viewport declaration
    .user-impact = On mobile devices the text appears below any readable size; zooming in forces horizontal scrolling. Desktop browsers are not affected.
    .technical-impact = No <meta name="viewport"> in the document.

rule-css-overflow-hidden = No reflow at 320 CSS pixels
    .user-impact = Users zooming to 400% must scroll in two directions — content becomes inaccessible.
    .technical-impact = Fixed widths or overflow:hidden prevent responsive reflow.

rule-contrast-control-insufficient = Insufficient contrast for UI elements
    .user-impact = Interactive elements are hard to recognize.
    .technical-impact = Contrast ratio of UI elements below 3:1.

rule-contrast-control-undetermined = Insufficient contrast for UI elements
    .user-impact = Interactive elements are hard to recognize.
    .technical-impact = Contrast ratio of UI elements below 3:1.

rule-text-spacing-clipped = Content clipped when text spacing is increased
    .user-impact = Users who increase line/letter/word spacing for readability lose content.
    .technical-impact = Fixed height/width with overflow:hidden on text containers; measured before and after overriding the text spacing, only newly clipped text counts.

rule-color-link-indistinct = Link distinguishable by color only
    .user-impact = Users with color vision deficiency cannot recognize in-text links.
    .technical-impact = No underline, bold weight or other non-color cue present.

rule-motion-reduced-motion-ignored = Animations without prefers-reduced-motion
    .user-impact = Users with vestibular disorders or photosensitivity cannot disable motion.
    .technical-impact = No @media (prefers-reduced-motion) in the stylesheets.

rule-patterns-tooltip-unreferenced = Inaccessible hover or focus content (tooltips)
    .user-impact = Users cannot read tooltip content, cannot dismiss it via keyboard, or it is ignored by screen readers.
    .technical-impact = Incomplete ARIA linkage (aria-describedby) or missing controls.

rule-media-audio-autoplay = Check autoplaying audio for a control
    .user-impact = Audio that starts on its own drowns out screen reader speech.
    .technical-impact = <audio autoplay> without muted; whether it plays longer than three seconds and can be paused is not in the markup.

rule-text-justified = Limited visual presentation of text (AAA)
    .user-impact = Users with visual impairments or dyslexia have difficulty reading.
    .technical-impact = Insufficient CSS styling for line spacing or justified-text formatting.

rule-text-line-height-tight = Limited visual presentation of text (AAA)
    .user-impact = Users with visual impairments or dyslexia have difficulty reading.
    .technical-impact = Insufficient CSS styling for line spacing or justified-text formatting.

rule-visual-presentation = Limited visual presentation of text (AAA)
    .user-impact = Users with visual impairments or dyslexia have difficulty reading.
    .technical-impact = Insufficient CSS styling for line spacing or justified-text formatting.

## Technical robustness

rule-keyboard-hidden-focusable = Focusable element hidden from screen readers
    .user-impact = Keyboard users reach elements that screen readers do not announce — invisible traps.
    .technical-impact = aria-hidden="true" on a focusable element or its ancestor.

rule-status-live-overridden = Status messages not programmatically determinable
    .user-impact = Screen readers do not announce status messages correctly.
    .technical-impact = Conflict between role (alert/status/log/timer) and aria-live value.

rule-aria-required-attribute-missing = Required ARIA attribute missing
    .user-impact = Assistive technologies cannot reliably expose the element state or value.
    .technical-impact = Role and ARIA attribute set are incomplete.

rule-aria-required-parent-missing = Element outside its required parent context
    .user-impact = Assistive technologies cannot convey the element's relationship to its group (e.g. a tab to its tab list).
    .technical-impact = Missing structural parent context for the ARIA role in use.

rule-inert-dialog-inert = Invalid modern interaction attributes
    .user-impact = Screen reader and keyboard users cannot reliably access dialogs, popovers, or disabled regions.
    .technical-impact = Broken popovertarget references, unnamed surfaces, or active focus inside inert regions.

rule-modern-attribute-misuse = Invalid modern interaction attributes
    .user-impact = Screen reader and keyboard users cannot reliably access dialogs, popovers, or disabled regions.
    .technical-impact = Broken popovertarget references, unnamed surfaces, or active focus inside inert regions.

rule-popover-target-invalid = Invalid modern interaction attributes
    .user-impact = Screen reader and keyboard users cannot reliably access dialogs, popovers, or disabled regions.
    .technical-impact = Broken popovertarget references, unnamed surfaces, or active focus inside inert regions.

rule-popover-target-missing = Invalid modern interaction attributes
    .user-impact = Screen reader and keyboard users cannot reliably access dialogs, popovers, or disabled regions.
    .technical-impact = Broken popovertarget references, unnamed surfaces, or active focus inside inert regions.

rule-aria-attribute-prohibited = Prohibited ARIA attributes
    .user-impact = Screen readers receive contradictory or invalid semantics.
    .technical-impact = ARIA attributes do not match the implicit or explicit role.

rule-aria-attribute-not-allowed = ARIA attribute not allowed for this role
    .user-impact = Assistive technologies ignore the attribute or interpret it incorrectly.
    .technical-impact = The attribute/role combination does not match the ARIA specification.

rule-aria-attribute-value-invalid = Invalid ARIA attribute value
    .user-impact = Assistive technologies cannot correctly convey the element's state or relationship.
    .technical-impact = Attribute value does not match the value type defined in the ARIA specification.

rule-aria-reference-missing = Empty reference in an ARIA relationship attribute
    .user-impact = Assistive technologies cannot resolve the intended relationship between the elements.
    .technical-impact = Relationship attribute without a valid ID reference.

rule-aria-role-abstract = Invalid ARIA role
    .user-impact = Assistive technologies cannot determine the element's function.
    .technical-impact = Role value is not defined in the ARIA specification.

rule-aria-role-invalid = Invalid ARIA role
    .user-impact = Assistive technologies cannot determine the element's function.
    .technical-impact = Role value is not defined in the ARIA specification.

rule-aria-attribute-unknown = Invalid ARIA attribute name
    .user-impact = Assistive technologies ignore the attribute entirely.
    .technical-impact = Attribute name does not exist in the ARIA specification.

rule-document-lang-mismatch = Conflicting language declarations
    .user-impact = Assistive technologies receive conflicting information about the content's language.
    .technical-impact = lang and xml:lang do not agree on the primary subtag.

rule-tables-headers-attr-invalid = Invalid headers reference in a table cell
    .user-impact = Screen readers cannot associate the cell with its correct column/row header, or none at all.
    .technical-impact = headers attribute does not reference a valid th cell in the same table.

rule-tables-data-undetermined = Table header without associated data cells
    .user-impact = Screen readers announce a header that structures no data.
    .technical-impact = Header cell with no associated data cells in the table.

rule-tables-header-without-data = Table header without associated data cells
    .user-impact = Screen readers announce a header that structures no data.
    .technical-impact = Header cell with no associated data cells in the table.

rule-aria-tabpanel-missing = Tab list without an associated tab panel
    .user-impact = Screen reader users cannot locate the content associated with a tab after activating it.
    .technical-impact = tablist without an accompanying tabpanel element.

rule-aria-tab-selected-missing = No tab marked as selected
    .user-impact = Screen reader users are not told which tab is active and which one the visible content belongs to.
    .technical-impact = tablist without a tab carrying aria-selected="true".

rule-patterns-accordion-controls-missing = Expanded button without a reference to its region
    .user-impact = Screen reader users are not told which region was expanded.
    .technical-impact = aria-expanded="true" without aria-controls; WAI-ARIA does not require it, the APG lists it as optional for the disclosure pattern.

rule-dialog-modal-unmarked = Dialog not marked as modal
    .user-impact = If the dialog is meant to be modal, screen readers keep reading the content behind it as if it were reachable.
    .technical-impact = role="dialog" without aria-modal="true"; whether the dialog is meant to be modal only shows in its behavior.

rule-aria-combobox-popup-missing = Combobox without an options list
    .user-impact = Screen reader users cannot tell what choices the combobox offers.
    .technical-impact = combobox without associated listbox/option elements.

rule-frames-name-missing = Iframe without accessible name
    .user-impact = Screen reader users cannot identify the purpose and content of the embedded frame.
    .technical-impact = Missing accessible name on iframe/frame.

rule-frames-cross-origin = Cross-origin iframe: review manually
    .user-impact = Embedded content (maps, videos, consent widgets) may be inaccessible to screen reader users.
    .technical-impact = Iframe content is unreachable via CDP — automatic rule checking is not possible.

rule-ids-duplicate = Referenced ID is assigned more than once
    .user-impact = Assistive technology cannot resolve the relationship unambiguously and announces the name or description of the wrong element.
    .technical-impact = The same ID occurs more than once in the DOM and is referenced by a for, headers or aria attribute.

rule-images-server-side-map = Server-side image map used
    .user-impact = Keyboard and screen reader users cannot target the individual image regions.
    .technical-impact = ismap attribute on an image without an equivalent client-side alternative.

rule-svg-name-missing = Missing alternative text on SVG graphics
    .user-impact = Screen reader users receive no information about the graphic's content.
    .technical-impact = SVG without a <title> child element, aria-label, or aria-labelledby.

rule-images-area-alt-missing = Missing alternative text on image map areas
    .user-impact = Screen reader users are not told where a clickable image region leads.
    .technical-impact = <area> element without an alt attribute.

rule-images-input-alt-missing = Missing alternative text on image buttons
    .user-impact = Screen reader users are not told what action the button triggers.
    .technical-impact = <input type="image"> without an alt attribute.

rule-objects-alt-missing = Missing text alternative on embedded objects
    .user-impact = Screen reader users receive no information about the embedded object's content.
    .technical-impact = <object> without text content or aria-label.

rule-html-content-model = HTML5 content model violation
    .user-impact = Screen readers cannot reliably parse the structure, even when it appears superficially valid.
    .technical-impact = Unexpected child element inside a structural container (e.g. <ul>, <table>, <select>).

rule-forms-group-missing = Missing grouping for related form fields
    .user-impact = Screen reader users cannot tell that several fields belong to one question/group.
    .technical-impact = Radio buttons/checkboxes without an enclosing <fieldset> with <legend>.

rule-landmarks-not-top-level = Landmark is nested
    .user-impact = Screen reader users who jump via landmark navigation land in an unexpected context.
    .technical-impact = <header>/<footer>/<main> or role="banner"/"contentinfo"/"main" is a descendant of another landmark.

rule-landmarks-banner-duplicate = Multiple banner landmarks
    .user-impact = Screen reader users cannot uniquely identify the page's 'header' region.
    .technical-impact = More than one element with role="banner" (implicit or explicit).

rule-landmarks-contentinfo-duplicate = Multiple contentinfo landmarks
    .user-impact = Screen reader users cannot uniquely identify the page's 'footer' region.
    .technical-impact = More than one element with role="contentinfo" (implicit or explicit).

rule-landmarks-main-duplicate = Multiple main landmarks
    .user-impact = Screen reader users who jump to the main content cannot tell which region is meant.
    .technical-impact = More than one element with role="main" (implicit or explicit).

rule-landmarks-banner-missing = Missing banner landmark
    .user-impact = Assistive technologies cannot identify the site header.
    .technical-impact = No element with role="banner" (implicit via <header> or explicit) present.

rule-landmarks-navigation-missing = Missing navigation landmark
    .user-impact = Screen reader users cannot jump straight to the navigation.
    .technical-impact = No element with role="navigation" (implicit via <nav> or explicit) present.

rule-landmarks-contentinfo-missing = Missing contentinfo landmark
    .user-impact = Screen reader users cannot jump straight to the footer with contact and legal information.
    .technical-impact = No element with role="contentinfo" (implicit via a <footer> outside <main>/<article>/<section>, or explicit) present.

rule-lists-item-outside-list = List items outside a list container
    .user-impact = Screen reader users receive no announcement of the list's length/structure.
    .technical-impact = <li> outside <ul>/<ol>.

rule-headings-skip-level = Skipped heading level
    .user-impact = Screen reader users navigating by headings assume that sections are missing.
    .technical-impact = Heading levels do not follow each other without gaps.

rule-headings-empty = Empty heading
    .user-impact = Screen reader users find an entry without content in the list of headings.
    .technical-impact = <h1>–<h6> or role="heading" without text and without aria-label.

rule-headings-h1-missing = Missing main heading (h1)
    .user-impact = Screen reader users cannot find the start of the main content via heading navigation.
    .technical-impact = No <h1> element in the document.

rule-headings-h1-multiple = Multiple main headings (h1)
    .user-impact = Screen reader users find it harder to tell the actual page title from the rest.
    .technical-impact = Several <h1> elements in the document.

rule-lists-invalid-structure = List with invalid child elements
    .user-impact = Screen reader users hear a wrong number of list items or lose content.
    .technical-impact = Direct children of <ul>/<ol>/role="list" are not <li>/role="listitem".

rule-lists-empty = Empty list
    .user-impact = Screen readers announce a list without content.
    .technical-impact = <ul>/<ol>/role="list" without <li>/role="listitem".

rule-lists-term-without-definition = Term without definition in a description list
    .user-impact = Screen reader users cannot recognize the term and its definition as belonging together.
    .technical-impact = <dt> without a following <dd> inside the <dl>.

rule-tables-header-missing = Data table without header cells
    .user-impact = Screen reader users hear cell values without their row or column label.
    .technical-impact = No <th> and no cell with role="columnheader"/"rowheader".

rule-tables-name-missing = Data table without a name
    .user-impact = Screen reader users are not told what the table is about when entering it.
    .technical-impact = No <caption> and no aria-label on the table.

rule-tables-presentational-with-headers = Layout table with header cells
    .user-impact = Screen reader users receive contradictory signals about whether this is a data table.
    .technical-impact = role="presentation"/"none" on a table with <th>.

rule-document-lang-invalid = Invalid page language
    .user-impact = Screen readers choose a wrong or the default pronunciation.
    .technical-impact = Invalid value in the html element's lang attribute.

rule-keyboard-positive-tabindex = Positive tabindex changes the focus order
    .user-impact = Keyboard users reach elements in an order that does not match the page layout.
    .technical-impact = tabindex value greater than 0.

rule-forms-title-only-label = Form field labeled only via the title attribute
    .user-impact = The title attribute is read inconsistently or not at all by many screen readers; sighted users only see it on hover.
    .technical-impact = Form element without <label>/aria-label, name derived only from title.

rule-zoom-viewport-scale-limited = Viewport meta tag prevents zooming to 500%
    .user-impact = Users with low vision cannot magnify the page content enough.
    .technical-impact = maximum-scale in the viewport meta tag set below 5.

rule-keyboard-click-handler-not-focusable = Click handler without keyboard equivalent
    .user-impact = Keyboard users cannot trigger the function.
    .technical-impact = onclick on a <div>/<span> without onkeydown/onkeyup, tabindex, or a matching role.

rule-keyboard-pointer-only = Click handler without keyboard equivalent
    .user-impact = Keyboard users cannot trigger the function.
    .technical-impact = onclick on a <div>/<span> without onkeydown/onkeyup, tabindex, or a matching role.

rule-landmarks-main-missing = No main-content region as a bypass target
    .user-impact = Keyboard users have no reliable target to bypass repeated navigation blocks.
    .technical-impact = No element with role="main" (implicit via <main> or explicit) present.

rule-keyboard-skip-link-missing = Missing skip link
    .user-impact = Keyboard users must tab through the entire navigation on every page change.
    .technical-impact = No link with a recognizable 'skip'/'to content' text or anchor to #main/#content at the top of the page.

rule-focus-indicator-missing = No visible focus indicator measured
    .user-impact = Keyboard users cannot see which element currently has focus.
    .technical-impact = Focus pass: the element's computed styles before and after focusing are equal, as far as they can serve as an indicator (outline, box-shadow, border, background, color, text-decoration).

rule-focus-outline-removed = Focus indicator removed via CSS
    .user-impact = Keyboard users cannot see which element currently has focus.
    .technical-impact = outline: none/0 in :focus rules without an alternative visible focus indicator.

rule-keyboard-scrollable-region-not-focusable = Scrollable region not reachable by keyboard
    .user-impact = Keyboard users cannot scroll the region and cannot read the hidden content.
    .technical-impact = Element with overflow: auto/scroll and overflowing content, without tabindex and without focusable children.

rule-forms-error-unidentified = Invalid field without an accessible error description
    .user-impact = Screen reader users learn that an error exists, but not what it is.
    .technical-impact = aria-invalid="true" without a description/aria-describedby pointing to a visible error element.

rule-aria-owns-conflict = ARIA reference points to an ambiguous ID
    .user-impact = Assistive technologies cannot reliably determine which element is referenced.
    .technical-impact = The referenced ID occurs more than once in the DOM.

rule-names-required-missing = Interactive element without a meaningful name
    .user-impact = Screen reader users are not told what the element does.
    .technical-impact = Missing or purely symbolic aria-label/aria-labelledby/text content.

rule-names-symbol-only = Interactive element without a meaningful name
    .user-impact = Screen reader users are not told what the element does.
    .technical-impact = Missing or purely symbolic aria-label/aria-labelledby/text content.

rule-names-title-only = Native title attribute as the only description
    .user-impact = The title attribute's tooltip only appears on mouse hover; with a keyboard and on touch devices the element's purpose stays invisible.
    .technical-impact = The accessible name comes only from the title attribute; no text content, no aria-label, no aria-labelledby.

rule-description-duplicates-name = Description repeats the name
    .user-impact = Screen readers announce the same text twice without adding information.
    .technical-impact = aria-describedby or title yields the same text as the name.

rule-aria-required-children-missing = Required child elements of an ARIA role are missing
    .user-impact = Assistive technologies cannot resolve the relationship between the widget's parts.
    .technical-impact = e.g. role="tablist" without corresponding role="tabpanel" elements.

rule-dialog-name-missing = Dialog without an accessible name
    .user-impact = Screen reader users are not told what the dialog is about when it opens.
    .technical-impact = Dialog element without aria-label or aria-labelledby pointing to the dialog title.

rule-buttons-name-missing = Form control, link, or button without an accessible name
    .user-impact = Screen reader users are not told what input is expected or where a link/button leads.
    .technical-impact = Form control/link/button without <label>, aria-label, aria-labelledby, or text content.

rule-links-name-missing = Form control, link, or button without an accessible name
    .user-impact = Screen reader users are not told what input is expected or where a link/button leads.
    .technical-impact = Form control/link/button without <label>, aria-label, aria-labelledby, or text content.

rule-links-used-as-button = Link used as a button substitute without a matching role
    .user-impact = Assistive technologies announce the element as a link even though it behaves like a button — keyboard users expect the wrong interaction.
    .technical-impact = <a href="#">/<a href="javascript:..."> with onclick, without role="button".

rule-aria-role-redundant = Redundant ARIA role
    .user-impact = No immediate user harm, but an unnecessary risk of inconsistency in future changes.
    .technical-impact = role attribute duplicates the tag's native implicit role.

rule-lists-role-redundant = Redundant ARIA role
    .user-impact = No immediate user harm, but an unnecessary risk of inconsistency in future changes.
    .technical-impact = role attribute duplicates the tag's native implicit role.

rule-summary-name-missing = Summary element without an accessible name
    .user-impact = Screen reader users are not told what the disclosure section contains before opening it.
    .technical-impact = <summary> without text content or aria-label.

rule-landmarks-content-outside = Content outside a landmark region
    .user-impact = Content outside a landmark region.
    .technical-impact = Content outside a landmark region.

rule-keyboard-focusable-no-role = Focusable element without an interactive role
    .user-impact = Focusable element without an interactive role.
    .technical-impact = Focusable element without an interactive role.

rule-display-toggle-missing = No display-mode toggle
    .user-impact = If there really is no toggle, visitors who cannot cope with motion or graphics cannot switch to a calm or text-only display.
    .technical-impact = No [data-display-toggle] element on a page with figure[data-viz].

rule-display-init-missing = Display mode set too late
    .user-impact = The page can briefly render in the wrong mode, for example with animation, before the chosen calm mode applies.
    .technical-impact = data-display not set by a blocking script in <head>.

rule-display-text-media-visible = Visualisations visible in text mode
    .user-impact = Visitors who chose text mode still get graphics or moving content.
    .technical-impact = [data-viz-live]/[data-viz-static] or figure media visible with html[data-display=text].

rule-display-text-not-visible = Visualisation text layer missing in text mode
    .user-impact = In text mode the statement of the visualisation is lost.
    .technical-impact = [data-viz-text] missing, empty or not visible with html[data-display=text].

rule-viz-text-missing = Visualisation text layer missing in text mode
    .user-impact = In text mode the statement of the visualisation is lost.
    .technical-impact = [data-viz-text] missing, empty or not visible with html[data-display=text].

rule-display-text-hidden = Visualisation text layer hidden from assistive technology
    .user-impact = Screen reader users do not get the statement of the visualisation.
    .technical-impact = hidden, aria-hidden=true, inert or display:none/visibility:hidden on [data-viz-text] or an ancestor.

rule-viz-caption-missing = Visualisation without a caption
    .user-impact = Nothing names the visualisation in a few words; those who cannot see it do not learn what it is about before reading through the text layer.
    .technical-impact = figure[data-viz] without a <figcaption> as a direct child.

rule-viz-static-missing = Moving visualisation without a still image
    .user-impact = In calm mode nothing replaces the motion; those who cannot cope with motion get either the motion or nothing.
    .technical-impact = figure[data-viz=3d|interactive] without a descendant carrying [data-viz-static].

rule-viz-table-missing = Chart without a values table
    .user-impact = The relations between the values are lost unless the text layer conveys them another way.
    .technical-impact = figure[data-viz=chart] without a <table> in its subtree.
