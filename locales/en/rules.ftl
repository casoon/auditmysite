# Rule texts: one message per rule id of `src/taxonomy/rules.rs` (`RULES`),
# in the order of that table. The message value is the title, shown for the
# rule everywhere (finding, `fix_guidance[]`, PDF, table). The attributes
# `.user-impact` and `.technical-impact` are the catalogue texts and exist for
# every rule. A rule that has an explanation entry under its own id (the
# entry key is the rule id, see `RULE_ENTRIES` in `src/output/explanations.rs`)
# also carries the explanation texts `.description`, `.explanation-user-impact`,
# `.typical-cause`, `.recommendation` and `.technical-note`.
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
    .description = An image has alternative text that probably does not describe its content: a file name, a filler word such as "image" or "logo", or just one or two characters.
    .explanation-user-impact = Screen reader users hear text that does not convey the image content to them.
    .typical-cause = The CMS copies the file name as alternative text, or the required field was filled with a placeholder.
    .recommendation = Check whether the text describes the image; otherwise replace it with a description, or with alt="" for decorative images.
    .technical-note = Detected heuristically, hence for review: alt="5" on an image of a five is correct.

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

rule-orientation-root-rotated = Page forces a screen orientation by rotating its root
    .user-impact = Users with fixed-mount screens (e.g. wheelchair-mounted) cannot read the page.
    .technical-impact = The computed transform (matrix) or rotate of html or body turns by a quarter turn, measured in the audited viewport.
    .description = The root element (html or body) is rotated by a quarter turn. The page is then usable in one orientation only, unless a specific orientation is essential.
    .explanation-user-impact = People whose device is mounted in a fixed position, for example on a wheelchair, cannot change the orientation and see the page rotated or cut off.
    .typical-cause = A transform: rotate(90deg) or rotate: 90deg on html or body that is meant to imitate landscape in portrait.
    .recommendation = Remove the rotation and make the layout usable in both orientations. Only fix an orientation when it is essential (a piano, a bank cheque).
    .technical-note = The computed transform matrix and the rotate property of html and body are measured in the audited viewport; a lock that applies in the other orientation only shows in the run of that viewport. Whether an orientation is essential is a human judgement.

rule-orientation-content-hidden = Restricted screen orientation
    .user-impact = Users with fixed-mount screens (e.g. wheelchair-mounted) cannot read the page.
    .technical-impact = CSS media queries lock the orientation.

## Navigation & operation

rule-keyboard-interactive-not-focusable = Content not operable by keyboard
    .user-impact = Keyboard users are excluded from functionality.
    .technical-impact = Click handler without keyboard alternative.

rule-keyboard-trap = Keyboard trap cannot be automatically tested
    .user-impact = The page becomes effectively unusable for keyboard users.
    .technical-impact = Missing focus management in modal elements.
    .description = Whether keyboard focus can become permanently trapped inside a dialog, carousel, or custom JavaScript widget cannot be determined automatically. It requires manually navigating the page using only the Tab key.
    .explanation-user-impact = People who navigate exclusively via keyboard or screen reader can become stuck inside a component and lose the ability to leave the page by keyboard.
    .typical-cause = Custom dialogs, carousels, or widgets with their own JavaScript focus management that provide no way back out via the Escape key, a reachable close button, or a documented keyboard shortcut.
    .recommendation = Ensure every focusable region has a way to leave it by keyboard: the Escape key, a visible close button reachable by Tab, or a documented keyboard shortcut.
    .technical-note = Navigate the page using only Tab/Shift+Tab and verify every dialog/widget can be exited. For custom focus management, implement an Escape handler and a reachable close button.

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
    .description = After focusing and scrolling, every sample point of a control lies under a fixed or sticky element such as a header bar or a cookie banner.
    .explanation-user-impact = Keyboard users tab to an element they cannot see and do not know where focus is.
    .typical-cause = A fixed or sticky bar that reaches lower than scroll-padding-top, or a banner that covers controls of the page.
    .recommendation = Set scroll-padding-top at least as large as the fixed bar and place banners so they do not cover controls.
    .technical-note = Measured after focusing with scrollIntoView({"{"} block: 'nearest' {"}"}), 3 by 3 points per control via elementsFromPoint. A real Tab may scroll differently, so the value is a hint to check, not proof. The hint about a bar that reaches lower than scroll-padding-top belongs to this id as well.

rule-focus-obscured-enhanced = Focused element partially obscured
    .user-impact = Keyboard users tab to an element whose visible area is partially covered by a sticky header, footer, cookie banner, or similar overlay.
    .technical-impact = Focus pass as for focus/obscured; at least one sample point lies under a position: fixed/sticky element.
    .description = After focusing and scrolling, at least one sample point of a control lies under a fixed or sticky element (level AAA).
    .explanation-user-impact = Keyboard users see a focused control only in part and may not recognise its label or state.
    .typical-cause = A sticky footer, chat button or cookie banner that covers an edge of the page.
    .recommendation = Keep room above and below fixed elements (scroll-padding, page margin) or make the elements smaller.
    .technical-note = The same measurement as for focus/obscured, a different threshold: every covered sample point is reported. Sampling only approximates "any part", so the result stays a hint to check.

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
    .description = A link opens a new window or tab (target="_blank") without saying so in its text. WCAG 3.2.5 (AAA) asks that such context changes happen only on request; announcing it in the link text is the usual way.
    .explanation-user-impact = Screen reader and magnifier users often miss the window change; the back button then does not return to the previous page.
    .typical-cause = External links or PDF downloads opened with target="_blank".
    .recommendation = Announce it in the link text, e.g. "(opens in a new window)", or drop target="_blank".
    .technical-note = A visible icon only counts with a text alternative in the accessible name (technique G201).

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
    .description = A password or one-time-code field cancels the paste event and does not insert the text itself. Password managers and the clipboard do not work there.
    .explanation-user-impact = Anyone who cannot memorise passwords or transcribe them accurately, for example because of cognitive or motor impairments, fails to sign in.
    .typical-cause = A paste handler with preventDefault() or onpaste="return false", usually meant as a security measure.
    .recommendation = Remove the handler. If the text is sanitised, insert the sanitised text into the field instead of forbidding the paste.
    .technical-note = A synthetic, cancelable paste event is dispatched on password and code fields, including same-origin frames and open shadow roots; the field value is restored afterwards. The event is not trusted (isTrusted is false): a handler that reacts only to real events or key presses stays unseen. autocomplete="off" is not a finding.

rule-auth-captcha = Review CAPTCHA in authentication form
    .user-impact = Text and puzzle CAPTCHAs lock people with cognitive impairments, dyslexia or low vision out of signing in when no alternative is offered.
    .technical-impact = Interactive CAPTCHA widget (reCAPTCHA, hCaptcha, image CAPTCHA) in a form with a password or one-time-code field.
    .description = A sign-in form contains a CAPTCHA. Whether it is an object-recognition test or an alternative without a cognitive test is offered needs to be checked.
    .explanation-user-impact = People who cannot read, transcribe or remember distorted characters cannot get past the sign-in.
    .typical-cause = A text or puzzle CAPTCHA against bots, without a second way in.
    .recommendation = Offer a sign-in without a cognitive test, e.g. a passkey, an e-mail link or a non-interactive bot check. An object-recognition test is allowed at level AA.
    .technical-note = Review hint: the kind of test and any alternatives are not in the markup. A CAPTCHA on a contact form is outside 3.3.8.

rule-forms-no-submit = Form without submit button
    .user-impact = Keyboard and screen reader users have a harder time completing the form.
    .technical-impact = Missing submit control in the form.
    .description = A form collects input but has no button or other control that explicitly submits it.
    .explanation-user-impact = Without a submit control it is unclear when the input takes effect. If the form submits when a field changes instead, the context changes without warning.
    .typical-cause = Filter and search forms submitted by script when a field changes, or a button outside the form without a form attribute.
    .recommendation = Add a button type="submit" to the form or associate one via the form attribute. If a form only changes content on the page, check that no change of context happens.
    .technical-note = Without an action and without a text field only script can submit the form; the finding is then a review hint (#728).

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
    .description = A passage appears to be written in another language than the page, without a lang attribute marking it.
    .explanation-user-impact = Screen readers read the passage with the page language's pronunciation, which makes it hard to understand.
    .typical-cause = Quotes, product names or teasers in another language without lang.
    .recommendation = Mark the passage with lang, for example <p lang="en">.
    .technical-note = Review hint: language detection is a heuristic and knows German and English. On pages in other languages the check is recorded as untested.

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

rule-zoom-viewport-locked = Zoom blocked by the viewport meta tag
    .user-impact = Users with low vision lose content when zooming in.
    .technical-impact = Fixed pixel values, overflow:hidden.
    .description = The viewport meta tag prevents zooming (user-scalable=no) or limits it to less than 200%.
    .explanation-user-impact = People with visual impairments cannot enlarge the page on mobile devices and lose content.
    .typical-cause = user-scalable=no or maximum-scale=1, set to prevent accidental zooming or to stabilize a layout.
    .recommendation = Remove user-scalable=no and maximum-scale from the viewport meta tag.
    .technical-note = A limit between 200% and 500% does not violate 1.4.4 and is reported separately.

rule-zoom-viewport-missing = Missing viewport declaration
    .user-impact = On mobile devices the text appears below any readable size; zooming in forces horizontal scrolling. Desktop browsers are not affected.
    .technical-impact = No <meta name="viewport"> in the document.
    .description = The document has no viewport meta tag. Mobile browsers then assume a desktop width and scale the page down.
    .explanation-user-impact = On mobile devices the text appears below any readable size; zooming in forces horizontal scrolling. Desktop browsers are not affected.
    .typical-cause = The template sets no viewport meta tag, as with older pages or pages built for desktop only.
    .recommendation = Add a viewport meta tag with width=device-width to the <head>.
    .technical-note = Desktop browsers ignore the tag; only the mobile rendering is affected. The layout must then also work in narrow viewports (1.4.10 Reflow).

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
    .description = An audio element starts by itself when the page loads and is not muted.
    .explanation-user-impact = Audio that starts on its own drowns out screen reader speech, which makes the control hard to find.
    .typical-cause = <audio autoplay> for background music or ads.
    .recommendation = Do not start audio automatically, or offer a control to pause or lower it at the start of the page.
    .technical-note = Review hint (WCAG 1.4.2): whether the audio plays longer than three seconds and can be paused is not in the markup.

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
    .description = An interactive element is hidden from screen readers via aria-hidden="true", but can still be focused using the keyboard (tab key).
    .explanation-user-impact = Keyboard users navigate to hidden elements (focus traps), while screen reader users receive no feedback about the element.
    .typical-cause = Using aria-hidden="true" to hide an element visually without setting tabindex="-1" or hiding the element via display:none.
    .recommendation = To hide an element completely, use display:none or visibility:hidden. If it should only be hidden from screen readers, make it unfocusable (tabindex="-1").
    .technical-note = Focusable elements (a, button, input) inside an aria-hidden="true" tree must either be removed from tab order or hidden completely.

rule-status-live-overridden = Status messages not programmatically determinable
    .user-impact = Screen readers do not announce status messages correctly.
    .technical-impact = Conflict between role (alert/status/log/timer) and aria-live value.

rule-aria-required-attribute-missing = Required ARIA attribute missing
    .user-impact = Assistive technologies cannot reliably expose the element state or value.
    .technical-impact = Role and ARIA attribute set are incomplete.
    .description = An element carries a role that announces a state (such as checkbox or slider) but not the attribute that carries this state.
    .explanation-user-impact = Screen reader users do not learn whether a switch is on or which value a slider has.
    .typical-cause = Custom controls set the role but maintain the state only visually.
    .recommendation = Set the required attribute and update it on every state change, or use a native element.
    .technical-note = For example aria-checked for checkbox/radio/switch, aria-expanded for combobox, aria-valuenow for slider/spinbutton/meter.

rule-aria-required-parent-missing = Element outside its required parent context
    .user-impact = Assistive technologies cannot convey the element's relationship to its group (e.g. a tab to its tab list).
    .technical-impact = Missing structural parent context for the ARIA role in use.
    .description = An element with a role such as tab, option, or listitem is not contained in an element with the required role.
    .explanation-user-impact = Screen readers cannot assign the element to a widget; position and count ("tab 2 of 3") are not announced.
    .typical-cause = An element with its own role in between, such as an <li> between role="tablist" and role="tab".
    .recommendation = Place the element directly in the required container, or neutralize the elements in between with role="presentation".
    .technical-note = Checked against the DOM. An <li> in a list that became something else via role stays a listitem, even when the browser flattens it in the accessibility tree.

rule-inert-dialog-inert = Invalid modern interaction attributes
    .user-impact = Screen reader and keyboard users cannot reliably access dialogs, popovers, or disabled regions.
    .technical-impact = Broken popovertarget references, unnamed surfaces, or active focus inside inert regions.

rule-modern-attribute-misuse = Broken dialog or popover control
    .user-impact = Screen reader and keyboard users cannot reliably access dialogs, popovers, or disabled regions.
    .technical-impact = Broken popovertarget references, unnamed surfaces, or active focus inside inert regions.
    .description = Modern HTML attributes for popovers, dialogs, or disabled regions are used incompletely or inconsistently.
    .explanation-user-impact = Keyboard and screen reader users may open, focus, or leave surfaces without understandable feedback.
    .typical-cause = popovertarget points to a missing element, a visible dialog has no accessible name, or an active region is marked inert at the same time.
    .recommendation = Check popover targets, name visible dialogs and menus, and use inert only for regions that are truly inactive.
    .technical-note = popovertarget must reference an element with popover. Open dialogs, menus, and popovers need aria-label, aria-labelledby, or another suitable name.

rule-popover-target-invalid = Invalid modern interaction attributes
    .user-impact = Screen reader and keyboard users cannot reliably access dialogs, popovers, or disabled regions.
    .technical-impact = Broken popovertarget references, unnamed surfaces, or active focus inside inert regions.

rule-popover-target-missing = Invalid modern interaction attributes
    .user-impact = Screen reader and keyboard users cannot reliably access dialogs, popovers, or disabled regions.
    .technical-impact = Broken popovertarget references, unnamed surfaces, or active focus inside inert regions.

rule-aria-attribute-prohibited = Prohibited ARIA attributes
    .user-impact = Screen readers receive contradictory or invalid semantics.
    .technical-impact = ARIA attributes do not match the implicit or explicit role.
    .description = Attributes are used on HTML elements or ARIA roles that are not permitted or meaningful for that specific role.
    .explanation-user-impact = Screen readers receive conflicting or incorrect markup signals, which can impair accessibility.
    .typical-cause = Using ARIA attributes (like aria-label or aria-expanded) on elements that do not have an interactive role (e.g. span or div without role).
    .recommendation = Check the ARIA specification for the respective role. Only use ARIA attributes on appropriate elements and roles.
    .technical-note = Prohibited attributes on generic elements: <span aria-label="text"> is invalid. Use native elements or valid roles (e.g. role="button").

rule-aria-attribute-not-allowed = ARIA attribute not allowed for this role
    .user-impact = Assistive technologies ignore the attribute or interpret it incorrectly.
    .technical-impact = The attribute/role combination does not match the ARIA specification.
    .description = An element carries an ARIA attribute its role does not support, or one that must not be used alongside a native attribute.
    .explanation-user-impact = Screen readers ignore the attribute or receive conflicting information about state and value.
    .typical-cause = States such as aria-expanded or aria-checked on elements without a matching role, such as a div.
    .recommendation = Move the attribute to the element with the matching role, set a matching role, or remove the attribute.
    .technical-note = The explicit or implicit role is judged. Also covered are the cases from ARIA in HTML, such as aria-checked on a native checkbox.

rule-aria-attribute-value-invalid = Invalid ARIA attribute value
    .user-impact = Assistive technologies cannot correctly convey the element's state or relationship.
    .technical-impact = Attribute value does not match the value type defined in the ARIA specification.
    .description = An ARIA attribute has a value outside its allowed range, such as aria-expanded="yes".
    .explanation-user-impact = Browsers fall back to the default value; the state is announced incorrectly or not at all to screen reader users.
    .typical-cause = Values from other conventions (yes/no, 1/0) or typos in token values.
    .recommendation = Only use the values the ARIA specification defines for the attribute, such as true or false.
    .technical-note = Unknown values of aria-current and aria-invalid count as true under ARIA 1.2 and are not reported.

rule-aria-reference-missing = Empty reference in an ARIA relationship attribute
    .user-impact = Assistive technologies cannot resolve the intended relationship between the elements.
    .technical-impact = Relationship attribute without a valid ID reference.
    .description = An ARIA relationship attribute (such as aria-controls or aria-labelledby) is empty or references an ID that does not exist on the page.
    .explanation-user-impact = Screen readers cannot establish the relationship: the label, description, or controlled region is missing.
    .typical-cause = The target's ID was renamed or is created later by script; components set the attribute before the target exists.
    .recommendation = Point the attribute at the ID of an existing element or remove it.
    .technical-note = Checked are aria-labelledby, aria-describedby, aria-controls, aria-owns, and aria-activedescendant.

rule-aria-role-abstract = Invalid ARIA role
    .user-impact = Assistive technologies cannot determine the element's function.
    .technical-impact = Role value is not defined in the ARIA specification.

rule-aria-role-invalid = Invalid ARIA role
    .user-impact = Assistive technologies cannot determine the element's function.
    .technical-impact = Role value is not defined in the ARIA specification.
    .description = A role attribute contains a value that does not exist in WAI-ARIA, or an abstract role that must not be used in markup.
    .explanation-user-impact = Browsers ignore the role; screen readers announce the element differently than intended.
    .typical-cause = Typos, invented roles, or abstract roles such as widget or landmark.
    .recommendation = Use a role from the ARIA specification, or remove the role attribute and use a native element.
    .technical-note = Roles are case-sensitive: role="Button" is invalid.

rule-aria-attribute-unknown = Invalid ARIA attribute name
    .user-impact = Assistive technologies ignore the attribute entirely.
    .technical-impact = Attribute name does not exist in the ARIA specification.
    .description = An element carries an aria-* attribute that does not exist in WAI-ARIA, usually a typo.
    .explanation-user-impact = The browser ignores the attribute; the intended label, relationship, or state is missing.
    .typical-cause = Misspelled attributes such as aria-labeledby instead of aria-labelledby.
    .recommendation = Check the name against the ARIA specification and correct it, or remove the attribute.
    .technical-note = Attributes from the WAI-ARIA 1.3 draft that browsers already support (such as aria-description) count as known.

rule-document-lang-mismatch = lang and xml:lang disagree
    .user-impact = Assistive technologies receive conflicting information about the content's language.
    .technical-impact = lang and xml:lang do not agree on the primary subtag.
    .description = The <html> element carries lang and xml:lang with different languages.
    .explanation-user-impact = Screen readers pick a different language depending on the attribute and may mispronounce the page.
    .typical-cause = Templates carried over from XHTML where only one attribute was updated.
    .recommendation = Remove xml:lang or set it to the same value as lang.
    .technical-note = The primary language of both attributes is compared.

rule-tables-headers-attr-invalid = Invalid headers reference in a table cell
    .user-impact = Screen readers cannot associate the cell with its correct column/row header, or none at all.
    .technical-impact = headers attribute does not reference a valid th cell in the same table.
    .description = A table cell references an ID via headers that is not a header cell of the same table.
    .explanation-user-impact = Screen readers announce the wrong header or none at all for the cell.
    .typical-cause = Renamed or copied IDs, references to cells of other tables.
    .recommendation = Point headers only at IDs of <th> in the same table, or use scope for simple tables.
    .technical-note = Only cells are checked (td, th and their roles); HTML defines the attribute only there.

rule-tables-data-undetermined = Table header without associated data cells
    .user-impact = Screen readers announce a header that structures no data.
    .technical-impact = Header cell with no associated data cells in the table.

rule-tables-header-without-data = Table header without data cells
    .user-impact = Screen readers announce a header that structures no data.
    .technical-impact = Header cell with no associated data cells in the table.
    .description = A header cell (<th>) applies to no data cell – the table has headers but no data below or beside them.
    .explanation-user-impact = Screen readers cannot relate data cells to a header; the table reads as empty or wrongly built.
    .typical-cause = Tables used for layout only, or data rows that a script adds later.
    .recommendation = Use header cells only where data follows; build layout without <table>.
    .technical-note = Empty data cells, cells with aria-hidden content and column headers over row headers are no error. A pool of rows not yet rendered gives an untested note instead of a violation.

rule-aria-tabpanel-missing = Tab list without an associated tab panel
    .user-impact = Screen reader users cannot locate the content associated with a tab after activating it.
    .technical-impact = tablist without an accompanying tabpanel element.

rule-aria-tab-selected-missing = No tab marked as selected
    .user-impact = Screen reader users are not told which tab is active and which one the visible content belongs to.
    .technical-impact = tablist without a tab carrying aria-selected="true".
    .description = None of the tabs in a tab list carries aria-selected="true".
    .explanation-user-impact = Screen reader users are not told which tab is active and which one the visible content belongs to.
    .typical-cause = The active tab is highlighted by a CSS class only.
    .recommendation = Set aria-selected="true" on the active tab and update it when the tab changes.
    .technical-note = For review: ARIA 1.2 says SHOULD. Tabs without the attribute count as not selected, which is correct for the inactive ones.

rule-patterns-accordion-controls-missing = Expanded button without a reference to its region
    .user-impact = Screen reader users are not told which region was expanded.
    .technical-impact = aria-expanded="true" without aria-controls; WAI-ARIA does not require it, the APG lists it as optional for the disclosure pattern.
    .description = An expanded button (aria-expanded="true") does not reference the region it controls with aria-controls.
    .explanation-user-impact = Screen reader users are not told which region was expanded and cannot jump straight to it.
    .typical-cause = Accordion or disclosure scripts that only toggle aria-expanded.
    .recommendation = Set aria-controls on the button to the ID of the controlled region.
    .technical-note = Review hint: WAI-ARIA does not require aria-controls, the APG lists it as optional for the disclosure pattern. A <summary> does not need it.

rule-dialog-modal-unmarked = Dialog not marked as modal
    .user-impact = If the dialog is meant to be modal, screen readers keep reading the content behind it as if it were reachable.
    .technical-impact = role="dialog" without aria-modal="true"; whether the dialog is meant to be modal only shows in its behavior.
    .description = An element with role="dialog" does not carry aria-modal="true". Whether the dialog is meant to be modal needs to be checked.
    .explanation-user-impact = If the dialog is meant to be modal, screen readers keep reading the content behind it as if it were reachable.
    .typical-cause = A custom dialog built from a div with role="dialog", without aria-modal.
    .recommendation = Mark a modal dialog with aria-modal="true", or use a native <dialog> opened with showModal(). A non-modal dialog does not need the attribute.
    .technical-note = ARIA does not require aria-modal; this is a review hint, not a violation.

rule-aria-combobox-popup-missing = Expanded combobox without a reference to its popup
    .user-impact = Screen reader users cannot tell what choices the combobox offers.
    .technical-impact = combobox without associated listbox/option elements.
    .description = A combobox is marked as expanded but does not reference its list of options with aria-controls.
    .explanation-user-impact = Screen reader users are not told which options the combobox offers and cannot reach them.
    .typical-cause = The list sits next to the combobox instead of inside it, and aria-controls is missing.
    .recommendation = Set aria-controls on the combobox to the ID of the list.
    .technical-note = ARIA 1.2 requires aria-controls (MUST). A popup in the combobox's own subtree or via aria-owns (ARIA 1.1) is still accepted.

rule-frames-name-missing = Iframe without accessible name
    .user-impact = Screen reader users cannot identify the purpose and content of the embedded frame.
    .technical-impact = Missing accessible name on iframe/frame.
    .description = An <iframe> has no accessible name (title, aria-label or aria-labelledby).
    .explanation-user-impact = Screen readers announce just "frame" when entering it; users cannot tell what it contains or skip it on purpose.
    .typical-cause = Embedded maps, videos, widgets or ads without a title attribute.
    .recommendation = Give every visible <iframe> a title that describes its content.
    .technical-note = WCAG technique H64. Frames with role="none"/"presentation", hidden, aria-hidden or width and height 0/1 do not count.

rule-frames-cross-origin = Cross-origin iframe: review manually
    .user-impact = Embedded content (maps, videos, consent widgets) may be inaccessible to screen reader users.
    .technical-impact = Iframe content is unreachable via CDP — automatic rule checking is not possible.
    .description = The document inside an iframe could not be read by the testing tool, so it was not tested. The frame has a foreign origin, or its navigation had not finished.
    .explanation-user-impact = Embedded content such as maps, booking widgets or consent dialogs may be inaccessible to screen reader and keyboard users without the report showing it.
    .typical-cause = Third-party content in an iframe with a foreign origin; the page cannot read its document.
    .recommendation = Check the frame content by hand or test its address on its own; ask the provider for an accessible embed.
    .technical-note = A coverage note, not a conformance check and tied to no success criterion. It measures whether the frame's contentDocument is readable; presentational, hidden or 1 px frames are skipped. Without the measurement the rule is reported as not run.

rule-ids-duplicate = Referenced ID is assigned more than once
    .user-impact = Assistive technology cannot resolve the relationship unambiguously and announces the name or description of the wrong element.
    .technical-impact = The same ID occurs more than once in the DOM and is referenced by a for, headers or aria attribute.
    .description = Multiple elements on the page share the same id, and another element references it (e.g. label for, aria-labelledby). Which of them is meant can no longer be determined.
    .explanation-user-impact = Screen readers resolve the relationship to the wrong element and announce the wrong name or description.
    .typical-cause = Repeatedly embedded components or templates (e.g. menu items, widgets) that use a static ID instead of a per-instance unique ID.
    .recommendation = Assign each ID only once per document. For reused components, generate the ID dynamically (e.g. with an index or slug).
    .technical-note = Make IDs unique, e.g. by appending an index or a unique identifier per component instance.

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
    .description = HTML5 content model violation. Screen readers cannot reliably parse the structure, even when it appears superficially valid.
    .explanation-user-impact = Screen readers cannot reliably parse the structure, even when it appears superficially valid.
    .typical-cause = Unexpected child element inside a structural container (e.g. <ul>, <table>, <select>).
    .recommendation = Fix the markup so it matches the HTML5 content model for the element (e.g. a <dl> may only directly contain <dt>/<dd> groups, <div>, <script>, or <template>). Invalid children can be silently "healed" by the browser when building the accessibility tree, hiding the defect from role-based checks while still confusing assistive technology.
    .technical-note = Unexpected child element inside a structural container (e.g. <ul>, <table>, <select>).

rule-forms-group-missing = Missing grouping for related form fields
    .user-impact = Screen reader users cannot tell that several fields belong to one question/group.
    .technical-impact = Radio buttons/checkboxes without an enclosing <fieldset> with <legend>.

rule-landmarks-not-top-level = Landmark is nested
    .user-impact = Screen reader users who jump via landmark navigation land in an unexpected context.
    .technical-impact = <header>/<footer>/<main> or role="banner"/"contentinfo"/"main" is a descendant of another landmark.
    .description = A header, footer or main content region (banner, contentinfo, main) sits inside another landmark instead of at the top level.
    .explanation-user-impact = Screen reader users who jump to the header, footer or main content via landmark navigation land in an unexpected context.
    .typical-cause = role="banner", role="contentinfo" or <main> inside <main>, <nav>, <aside> or a named region.
    .recommendation = Move the header, footer and main content region to the top level of the page, outside any other landmark.
    .technical-note = A <header>/<footer> inside <main>, <article> or <section> is no banner/contentinfo anyway; reported are explicit roles and <main>. The affected role is named in the finding.

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

rule-lists-item-outside-list = List item outside a list
    .user-impact = Screen reader users receive no announcement of the list's length/structure.
    .technical-impact = <li> outside <ul>/<ol>.
    .description = A list item (<li> or role="listitem") is not inside a list.
    .explanation-user-impact = Screen readers announce a list item without a list that gives its length and position.
    .typical-cause = The enclosing <ul>/<ol> is missing, for example because a component renders only the items, or role="listitem" sits on an arbitrary element.
    .recommendation = Wrap list items in a <ul> or <ol> (or role="list"), or remove the list semantics if it is not a list.
    .technical-note = An <li> needs a <ul>, <ol> or <menu> parent; role="listitem" needs a role="list" ancestor.

rule-headings-skip-level = Skipped heading level
    .user-impact = Screen reader users navigating by headings assume that sections are missing.
    .technical-impact = Heading levels do not follow each other without gaps.
    .description = Headings skip a level, for example from h2 straight to h4. The page outline has a gap.
    .explanation-user-impact = Screen reader users navigate via the list of headings and suspect missing content where a level is skipped.
    .typical-cause = Heading levels chosen for their font size rather than the outline, or components that ship with a fixed level.
    .recommendation = Nest headings without gaps (h1, h2, h3 ...) and control their look with CSS, not with the level.
    .technical-note = A heading of level n is followed by at most level n+1. Components should take the level as a parameter.

rule-headings-empty = Empty heading
    .user-impact = Screen reader users find an entry without content in the list of headings.
    .technical-impact = <h1>–<h6> or role="heading" without text and without aria-label.
    .description = A heading element has no text. It appears in the page outline but says nothing.
    .explanation-user-impact = Screen reader users navigate via the list of headings and hit an entry without a name.
    .typical-cause = Heading tags used as spacers or for icons, or CMS fields left empty while the tag is still rendered.
    .recommendation = Remove empty headings or give them meaningful text; create spacing with CSS.
    .technical-note = Render the tag only when the title exists. An icon alone does not name a heading.

rule-headings-h1-missing = Missing main heading (h1)
    .user-impact = Screen reader users cannot find the start of the main content via heading navigation.
    .technical-impact = No <h1> element in the document.
    .description = The page has no level-1 heading that names its topic.
    .explanation-user-impact = Screen reader users jump to the h1 to learn what the page is about and find none.
    .typical-cause = The logo or an image stands in for the heading, or the template renders the main heading as an h2 or as styled text.
    .recommendation = Give every page one h1 that states its topic, at the start of the main content.
    .technical-note = The h1 belongs inside <main>, not in the header logo.

rule-headings-h1-multiple = Multiple main headings (h1)
    .user-impact = Screen reader users find it harder to tell the actual page title from the rest.
    .technical-impact = Several <h1> elements in the document.
    .description = The page has more than one level-1 heading. HTML allows this, but it should be intentional.
    .explanation-user-impact = Screen reader users find it harder to tell the actual page title from other sections.
    .typical-cause = The logo, teasers or a sidebar also use h1, often from a shared template.
    .recommendation = Check whether each h1 really names a main topic of the page; move all others to h2 or below.
    .technical-note = A review note, not a violation: whether it is a problem depends on the page structure.

rule-lists-invalid-structure = List with invalid child elements
    .user-impact = Screen reader users hear a wrong number of list items or lose content.
    .technical-impact = Direct children of <ul>/<ol>/role="list" are not <li>/role="listitem".
    .description = A list (<ul>, <ol> or role="list") has direct children that are not list items.
    .explanation-user-impact = Screen readers announce the wrong number of items or read parts of the list outside its list context.
    .typical-cause = Wrapper <div>s, separators or headings placed directly in the list, often by a slider or menu component.
    .recommendation = Put all list content inside <li> elements; move wrappers and separators into the items or create them with CSS.
    .technical-note = Direct children of <ul>/<ol> may only be <li>, <script> and <template>.

rule-lists-empty = Empty list
    .user-impact = Screen readers announce a list without content.
    .technical-impact = <ul>/<ol>/role="list" without <li>/role="listitem".
    .description = A list contains no list items at all.
    .explanation-user-impact = Screen readers announce a list that turns out to have nothing in it.
    .typical-cause = Lists filled by script or from the CMS that stay empty, or list markup used as a mere layout container.
    .recommendation = Do not output empty lists, or render the list element only once there are items.
    .technical-note = Check for items before rendering; a layout container does not need a <ul>.

rule-lists-term-without-definition = Term without definition in a description list
    .user-impact = Screen reader users cannot recognize the term and its definition as belonging together.
    .technical-impact = <dt> without a following <dd> inside the <dl>.
    .description = A description list (<dl>) contains a term (<dt>) without a matching description (<dd>).
    .explanation-user-impact = Screen readers read out a term that is followed by no description.
    .typical-cause = Empty CMS fields, or a <dl> used as a label/value layout where the value is missing.
    .recommendation = Output at least one <dd> for every <dt>, or leave the term out when there is no description.
    .technical-note = Checked within the same parent, including <div> groups inside the <dl>.

rule-tables-header-missing = Data table without header cells
    .user-impact = Screen reader users hear cell values without their row or column label.
    .technical-impact = No <th> and no cell with role="columnheader"/"rowheader".
    .description = A table has no header cells that label its rows or columns.
    .explanation-user-impact = Screen reader users hear only values when moving through the cells, without learning which column or row they belong to.
    .typical-cause = The header row is built from bold <td> cells instead of <th>, or the table is used for layout only.
    .recommendation = Mark header cells as <th> (with scope="col" or scope="row"). If the table is only for layout, replace it with CSS layout or set role="presentation".
    .technical-note = Header cells are <th> or elements with role="columnheader"/"rowheader" anywhere in the table.

rule-tables-name-missing = Data table without a name
    .user-impact = Screen reader users are not told what the table is about when entering it.
    .technical-impact = No <caption> and no aria-label on the table.
    .description = A table has neither a <caption> nor a name via aria-label or aria-labelledby.
    .explanation-user-impact = Users who list the tables on a page see only "table" and have to open each one to find the right one.
    .typical-cause = The caption is a heading or paragraph above the table that is not linked to it.
    .recommendation = Give the table a <caption>, or link the existing heading with aria-labelledby. Whether a name is needed depends on the context.
    .technical-note = A review note, not a violation: <caption> as the first child of the <table>, or aria-labelledby pointing to the heading.

rule-tables-presentational-with-headers = Layout table with header cells
    .user-impact = Screen reader users receive contradictory signals about whether this is a data table.
    .technical-impact = role="presentation"/"none" on a table with <th>.
    .description = A table is marked as layout only (role="presentation" or "none") but contains header cells.
    .explanation-user-impact = Assistive technology receives contradictory information: no table according to the role, table structure according to the header cells.
    .typical-cause = role="presentation" was applied to tables across the board, including real data tables.
    .recommendation = Remove role="presentation" from data tables. In layout tables, replace <th> with <td>.
    .technical-note = A table with <th> or role="columnheader"/"rowheader" must not carry role="presentation"/"none".

rule-document-lang-invalid = Invalid page language
    .user-impact = Screen readers choose a wrong or the default pronunciation.
    .technical-impact = Invalid value in the html element's lang attribute.
    .description = The page's lang attribute is set but does not hold a valid language code (for example "english" instead of "en").
    .explanation-user-impact = Screen readers do not recognize the language and read the text with the wrong or the default pronunciation.
    .typical-cause = A language name instead of a code, an underscore instead of a hyphen (de_DE), or an empty value from the template.
    .recommendation = Use a valid BCP 47 code, such as lang="en" or lang="en-GB".
    .technical-note = A primary subtag of two or three letters, followed only by alphanumeric subtags separated by hyphens.

rule-keyboard-positive-tabindex = Positive tabindex changes the focus order
    .user-impact = Keyboard users reach elements in an order that does not match the page layout.
    .technical-impact = tabindex value greater than 0.
    .description = An element has a tabindex greater than 0 and therefore receives focus before all other elements, regardless of where it sits on the page.
    .explanation-user-impact = Keyboard users jump around the page unpredictably when tabbing and lose their bearings.
    .typical-cause = Attempts to bring single elements forward with tabindex="1", often in form or search widgets.
    .recommendation = Remove positive tabindex values and set the order through the order in the HTML; tabindex="0" or "-1" is enough.
    .technical-note = Use only tabindex="0" (focusable in natural order) and "-1" (focusable by script only).

rule-forms-title-only-label = Form field labeled only via the title attribute
    .user-impact = The title attribute is read inconsistently or not at all by many screen readers; sighted users only see it on hover.
    .technical-impact = Form element without <label>/aria-label, name derived only from title.

rule-zoom-viewport-scale-limited = Viewport meta tag prevents zooming to 500%
    .user-impact = Users with low vision cannot magnify the page content enough.
    .technical-impact = maximum-scale in the viewport meta tag set below 5.
    .description = The viewport meta tag limits maximum-scale to a value below 5, so users cannot magnify the page to 500% on mobile devices.
    .explanation-user-impact = Users with low vision cannot magnify the page content enough.
    .typical-cause = maximum-scale was set to prevent accidental zooming or to stabilize a layout.
    .recommendation = Remove maximum-scale or set it to at least 5.
    .technical-note = Best practice: 1.4.4 requires 200% and is met with this limit. A limit below 2, or user-scalable=no, is reported as a separate 1.4.4 violation.

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

rule-aria-owns-conflict = Element owned by several elements via aria-owns
    .user-impact = Assistive technologies cannot reliably determine which element is referenced.
    .technical-impact = The referenced ID occurs more than once in the DOM.
    .description = The same ID appears in the aria-owns of several elements. Which element is the owner remains undetermined.
    .explanation-user-impact = Assistive technologies cannot assign the element to one widget; name, role, and value become unreliable.
    .typical-cause = Components included several times that point to the same list, or a duplicated ID.
    .recommendation = List each element in the aria-owns of a single element only.
    .technical-note = WAI-ARIA 1.2: an ID must not be specified in more than one other element's aria-owns at any time.

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
    .description = An element's accessible name and accessible description are identical. The element has a name; the repeated announcement is best practice, not a missing name.
    .explanation-user-impact = Screen readers announce the same text twice without adding information.
    .typical-cause = title or aria-describedby repeats the visible text or the aria-label, e.g. title="Close" on a "Close" button.
    .recommendation = Remove the description, or fill it with information that goes beyond the name.
    .technical-note = Best practice, anchored to WCAG 4.1.2. Compares name and description from the browser's accessibility tree.

rule-aria-required-children-missing = Required owned elements of an ARIA role are missing
    .user-impact = Assistive technologies cannot resolve the relationship between the widget's parts.
    .technical-impact = e.g. role="tablist" without corresponding role="tabpanel" elements.
    .description = A container such as role="tablist", "list", or "menu" contains none of the elements its role requires.
    .explanation-user-impact = Screen readers announce an empty or broken container; users cannot find its items.
    .typical-cause = The items carry no role or a different one, or an element with its own role sits between the container and its items.
    .recommendation = Give the items the required role, such as role="tab" in a tablist, or remove the container's role.
    .technical-note = Only containers with an explicit role are checked; native tables and lists define their parts through HTML.

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
    .description = The page shows visualisations, but no toggle marked [data-display-toggle] was found.
    .explanation-user-impact = If there really is no toggle, visitors who cannot cope with motion, 3D or graphics are stuck with the full display.
    .typical-cause = Visualisations marked up per the convention, but the toggle was not added or not marked with [data-display-toggle].
    .recommendation = Offer an operable toggle on every page with visualisations that switches between visual, calm and text and stores the choice.
    .technical-note = Best practice from the BarrierLab display-mode convention (draft v0), not a WCAG requirement; anchored to WCAG 2.2.2 (Pause, Stop, Hide). Checks for a [data-display-toggle] element in the rendered DOM.

rule-display-init-missing = Display mode set too late
    .user-impact = The page can briefly render in the wrong mode, for example with animation, before the chosen calm mode applies.
    .technical-impact = data-display not set by a blocking script in <head>.
    .description = The chosen display is not applied before the page is first shown.
    .explanation-user-impact = The page can briefly show animation or graphics even though a calm display was chosen.
    .typical-cause = data-display is set in a deferred or module script instead of a small blocking script in <head>.
    .recommendation = Set the mode in a small blocking script in <head>, before the page body starts.
    .technical-note = Best practice from the BarrierLab display-mode convention (draft v0), not a WCAG requirement; anchored to WCAG 2.2.2. Measured via CDP: an observer injected before navigation records whether html[data-display] was already set when <body> was inserted.

rule-display-text-media-visible = Visualisations visible in text mode
    .user-impact = Visitors who chose text mode still get graphics or moving content.
    .technical-impact = [data-viz-live]/[data-viz-static] or figure media visible with html[data-display=text].
    .description = In text mode a visualisation still shows graphics, canvas or video.
    .explanation-user-impact = Visitors who chose text mode still get graphics or moving content.
    .typical-cause = The CSS for html[data-display="text"] does not hide [data-viz-live] or [data-viz-static].
    .recommendation = In text mode hide the live and static parts of the visualisation and show only the text layer.
    .technical-note = Best practice from the BarrierLab display-mode convention (draft v0), not a WCAG requirement; anchored to WCAG 2.2.2. Checks the rendered layout in text mode; elements up to 32x32 px count as icons.

rule-display-text-not-visible = Text layer missing in text mode
    .user-impact = In text mode the statement of the visualisation is lost.
    .technical-impact = [data-viz-text] missing, empty or not visible with html[data-display=text].
    .description = In text mode a visualisation shows no readable statement.
    .explanation-user-impact = Visitors who chose text mode do not learn what the visualisation says.
    .typical-cause = [data-viz-text] is missing, empty or stays hidden in text mode.
    .recommendation = Give every visualisation a text layer (key statement, for charts the values as a table, source) and show it in text mode.
    .technical-note = Best practice from the BarrierLab display-mode convention (draft v0), not a WCAG requirement; anchored to WCAG 1.1.1 (Non-text Content). Checks the rendered layout in text mode.

rule-viz-text-missing = Visualisation without a text layer
    .user-impact = In text mode the statement of the visualisation is lost.
    .technical-impact = [data-viz-text] missing, empty or not visible with html[data-display=text].
    .description = A visualisation has no text layer marked [data-viz-text], or only an empty one.
    .explanation-user-impact = Those who cannot see the graphic do not get its statement; text mode has nothing to show in its place.
    .typical-cause = The description sits in an element without the [data-viz-text] mark, or the visualisation was built without a text layer.
    .recommendation = Give every figure[data-viz] a text layer with the statement (for charts the values, preferably as a <table>) and name the source.
    .technical-note = Best practice from the BarrierLab display-mode convention (draft v0), not a WCAG requirement; anchored to WCAG 1.1.1. Only checks that text is there, not whether it conveys the key message. From the markup, in every mode.

rule-display-text-hidden = Visualisation text layer hidden from assistive technology
    .user-impact = Screen reader users do not get the statement of the visualisation.
    .technical-impact = hidden, aria-hidden=true, inert or display:none/visibility:hidden on [data-viz-text] or an ancestor.
    .description = The text layer of a visualisation is hidden from screen readers.
    .explanation-user-impact = Screen reader users do not get the statement of the visualisation.
    .typical-cause = hidden, aria-hidden="true", inert or display:none on [data-viz-text] or a surrounding element, usually to hide it in the graphical display.
    .recommendation = Hide the text layer only visually (visually-hidden), never from assistive technology.
    .technical-note = Best practice from the BarrierLab display-mode convention (draft v0), not a WCAG requirement; anchored to WCAG 1.1.1. Checked in the rendered DOM in every mode, including computed styles.

rule-viz-caption-missing = Visualisation without a caption
    .user-impact = Nothing names the visualisation in a few words; those who cannot see it do not learn what it is about before reading through the text layer.
    .technical-impact = figure[data-viz] without a <figcaption> as a direct child.
    .description = A visualisation has no <figcaption> naming it in a few words.
    .explanation-user-impact = Nothing names the visualisation briefly; those who cannot see it have to read the text layer to know what it is about.
    .typical-cause = The <figcaption> is missing or sits outside the <figure>.
    .recommendation = Add a <figcaption> as a direct child of the figure[data-viz].
    .technical-note = Best practice from the BarrierLab display-mode convention (draft v0), not a WCAG requirement; anchored to WCAG 1.1.1. Checks presence, not content.

rule-viz-static-missing = Moving visualisation without a still image
    .user-impact = In calm mode nothing replaces the motion; those who cannot cope with motion get either the motion or nothing.
    .technical-impact = figure[data-viz=3d|interactive] without a descendant carrying [data-viz-static].
    .description = A 3D or interactive visualisation has no still image for calm mode.
    .explanation-user-impact = Those who cannot cope with motion get nothing in calm mode that replaces the motion.
    .typical-cause = Only the moving version was built, without a still image marked [data-viz-static].
    .recommendation = Include a still image marked [data-viz-static] in the visualisation and show it instead of the motion in calm mode.
    .technical-note = Best practice from the BarrierLab display-mode convention (draft v0), not a WCAG requirement; anchored to WCAG 2.2.2. Applies to data-viz="3d" and "interactive".

rule-viz-table-missing = Chart without a values table
    .user-impact = The relations between the values are lost unless the text layer conveys them another way.
    .technical-impact = figure[data-viz=chart] without a <table> in its subtree.
    .description = A chart has no table with its values. Check whether the text layer conveys them without one.
    .explanation-user-impact = The relations between the values can be lost when they exist only as running text or not at all.
    .typical-cause = The values exist only in the graphic or in one sentence, often deliberately for sparklines.
    .recommendation = Put the values into the text layer as a <table> unless a sentence carries them just as completely.
    .technical-note = Best practice from the BarrierLab display-mode convention (draft v0), not a WCAG requirement; anchored to WCAG 1.3.1. A review note, not a failure: only a person can judge whether a sentence is enough.

# Explanation texts of the entries of `EXPLANATIONS` in
# `src/output/explanations.rs` that are keyed by a WCAG criterion number or an
# axe id, in the order of that table: one message per entry. The message value
# is the customer title, the attributes carry the description, the user
# impact, the typical cause, the recommendation and the technical note.
# The key is `explanation-` plus the table key with `.`, `/` and `_` written
# as `-` (`output::explanations::explanation_text_key`). An entry keyed by a
# rule id has no message here: its texts are attributes of the rule message
# above.

## 1. Perceivable

explanation-1-1-1 = Missing alternative text for images
    .description = Images on the website have no descriptive alternative text. As a result, screen readers cannot convey the image content to blind or visually impaired users.
    .user-impact = People with visual impairments receive no or only incomplete information at these points. The image content is entirely lost to them.
    .typical-cause = Teaser images, sliders, or editorially uploaded media without a required field in the CMS, or decorative images that are not marked as such.
    .recommendation = Provide a descriptive alt text for informative images that conveys the image content or purpose. Mark purely decorative images with an empty alt attribute (alt="").
    .technical-note = Informative images: <img alt="description">. Decorative images: <img alt=""> or role="presentation". Configure alt-text fields in the CMS as required.

explanation-1-2-1 = Video/audio without an alternative not automatically verified
    .description = A video-only or audio-only element was found on the page. Whether an equivalent text alternative (e.g. a transcript) exists cannot be reliably determined from the HTML alone — this requires a manual content check.
    .user-impact = People who cannot see the video or hear the audio have no access to the conveyed content without a text alternative.
    .typical-cause = Embedded video/audio players (native or third-party) with no linked transcript page or text equivalent.
    .recommendation = For every video-only or audio-only element, manually verify that a transcript or equivalent text alternative exists and is complete.
    .technical-note = Place a text link to the transcript near the media element, e.g. directly below the player.

explanation-1-2-2 = Captions on video not confirmed
    .description = For a video on the page, a resolving caption/subtitle file (`<track kind="captions">`) could not be technically confirmed, or the video is a third-party embedded player whose caption settings cannot be checked from the outside.
    .user-impact = Deaf and hard-of-hearing users have no access to the video's spoken content without captions.
    .typical-cause = Video without a `<track kind="captions">` element, a track whose file is unreachable, or an embedded player (YouTube, Vimeo, etc.) whose caption status is configured by the platform, not the audited page.
    .recommendation = Ensure every prerecorded video with audio has synchronized captions. For embedded players, manually verify the caption settings within the player itself.
    .technical-note = Native `<video>`: add `<track kind="captions" src="..." srclang="en">`, the file must resolve. For embeds, enable the platform's caption feature.

explanation-1-2-3 = Audio description or text alternative not automatically verified
    .description = Whether a prerecorded video with visually conveyed information (e.g. actions, scene changes) has an audio description or a complete text alternative cannot be determined from the HTML alone.
    .user-impact = Blind and visually impaired users miss visually conveyed information that is not mentioned in the video's regular audio track.
    .typical-cause = Video with important visual content (diagrams, actions, on-screen text) without a separate audio description track and without a text alternative.
    .recommendation = Manually check videos with relevant visual content: does an audio description or a complete text alternative exist that covers this content?
    .technical-note = Add an audio description via `<track kind="descriptions">` or a separate audio track, or alternatively link a complete text alternative.

explanation-1-2-8 = Full text alternative for video not confirmed
    .description = For prerecorded video/audio content, it could not be automatically confirmed that a complete text alternative (a transcript including relevant visual information) exists.
    .user-impact = Deafblind users and users who can perceive neither image nor sound depend on a complete text alternative to access the content.
    .typical-cause = Video/audio element with no linked transcript, or a nearby link suggests a transcript exists but its completeness cannot be verified automatically.
    .recommendation = Provide a complete transcript for every prerecorded video/audio element and manually verify its completeness (including spoken dialogue and relevant visual information).
    .technical-note = Provide the transcript as text on the page or via a link, add `<track kind="descriptions">` for audio description where applicable.

explanation-1-3-1 = Missing semantic structure
    .description = Content is visually structured (e.g. by size or color), but the structure is not encoded in the HTML. Screen readers and other assistive technologies cannot recognize the relationships between pieces of content.
    .user-impact = Screen reader users cannot navigate tables, lists, and form groups correctly. The logical structure of the page is lost.
    .typical-cause = Tables without proper table markup, missing fieldsets in forms, lists implemented as divs instead of ul/ol, missing landmarks.
    .recommendation = Mark up content with correct semantics: tables with <table>, <th>, <td>; lists with <ul>/<ol>; group form fields with <fieldset> and <legend>.
    .technical-note = Use HTML5 semantics: <nav>, <main>, <aside>, <header>, <footer>. Tables: use scope attributes on header cells. Use ARIA roles only as a supplement.

explanation-region = Content outside a landmark region
    .description = An element with visible content — often a link — sits outside every landmark region of the page (main, nav, header, footer, aside). Screen reader users who jump between page regions via landmark navigation will not encounter this element.
    .user-impact = Screen reader users who switch between main content, navigation, and footer via landmark navigation skip over this element without noticing — it does not appear in any of the regions they navigate to. It stays reachable in reading mode and by Tab.
    .typical-cause = Markup inserted outside the main layout via JavaScript, leftover third-party widget content, or content placed before the first landmark without being embedded inside main/nav/header/footer itself.
    .recommendation = Move the element into the appropriate landmark region (e.g. a link into <nav> or <main>). If no existing region fits, add an additional region with role="region" and a descriptive aria-label.
    .technical-note = Use HTML5 landmarks consistently: <header>, <nav>, <main>, <aside>, <footer>. Skip links placed before the first landmark are deliberately exempt. For areas without a matching semantic element, use role="region" plus aria-label.

explanation-landmark-unique = Landmarks are not uniquely named
    .description = Multiple regions of the page with the same role (e.g. two navigation areas or two aside regions) share no distinguishing name. Screen readers announce both regions with the same generic name.
    .user-impact = Screen reader users who jump between regions via landmark navigation hear, for example, 'Navigation' announced twice and cannot tell the regions apart without further exploration.
    .typical-cause = Multiple <nav>, <aside>, or role="region" elements on the same page (e.g. main navigation and breadcrumb navigation) without an aria-label or aria-labelledby to distinguish them.
    .recommendation = Give each landmark of the same role its own descriptive aria-label, e.g. aria-label="Main navigation" and aria-label="Breadcrumb".
    .technical-note = Add aria-label or aria-labelledby to each of the repeated same-role landmarks. The names must actually differ — repeating the identical label does not satisfy this check.

explanation-1-3-5 = Missing input purpose identification
    .description = Form fields lack machine-readable identification of their purpose. As a result, browsers and assistive technologies cannot offer autofill suggestions or help users complete forms faster.
    .user-impact = People with motor or cognitive impairments cannot benefit from automatic form filling. Completing forms takes longer and is more error-prone.
    .typical-cause = Missing autocomplete attributes on form fields for personal data (name, email, address, phone).
    .recommendation = Add the appropriate autocomplete attribute to all form fields for personal data (e.g. autocomplete="email", autocomplete="given-name").
    .technical-note = Use autocomplete values per the HTML specification: name, email, tel, street-address, postal-code, country, etc.

explanation-1-4-1 = Links distinguishable by color alone
    .description = Links within body text are distinguishable from regular text only by their color. For people with color vision deficiency, these links are not recognizable as clickable.
    .user-impact = Users with red-green deficiency or other color vision impairments cannot recognize links in text and miss important navigation options.
    .typical-cause = CSS sets `text-decoration: none` on inline links without an alternative non-color distinguishing feature such as underline, bold, or an icon.
    .recommendation = Mark inline links with a non-color cue: underline (standard and recommended), bold text, or a small icon. Underline is the strongest convention.
    .technical-note = CSS: `a {"{"} text-decoration: underline; {"}"}` or, if `text-decoration: none`, at least `font-weight: bold` or `border-bottom`. Applies only to inline links in body text, not to navigation links.

explanation-1-4-3 = Insufficient color contrast
    .description = Text on the website has insufficient contrast against its background. Under poor lighting conditions or for people with low vision, the text is hard to read.
    .user-impact = People with visual impairments, older users, and any user under poor screen conditions (sunlight, low-quality displays) can read the text only with difficulty or not at all.
    .typical-cause = Light font color on a light background, gray text on white, design decisions made without contrast checking.
    .recommendation = Adjust font colors so normal text has a contrast ratio of at least 4.5:1 and large text at least 3:1 against the background. Verify contrast values with tools such as the WebAIM Contrast Checker.
    .technical-note = WCAG 2.1 Level AA: 4.5:1 for normal text (<18pt / <14pt bold), 3:1 for large text (≥18pt / ≥14pt bold). Use CSS variables for a consistent color palette.

explanation-1-4-4 = Text cannot be sufficiently resized
    .description = Text on the website cannot be enlarged to 200% without loss of content or functionality. Users who depend on larger text lose information.
    .user-impact = People with visual impairments who increase the browser font size encounter clipped text, overlapping elements, or non-scrollable areas.
    .typical-cause = Fixed pixel values for font sizes and containers, overflow: hidden on text containers, missing responsive adjustments.
    .recommendation = Specify font sizes in relative units (rem/em). Design containers so they grow with enlarged text. Test browser zoom at 200%.
    .technical-note = Use font-size in rem/em instead of px. Use min-height on containers instead of height. Prefer overflow: auto over hidden. Use @media queries for different zoom levels.

explanation-1-4-10 = No automatic text wrapping (Reflow)
    .description = The website does not adapt flexibly to high zoom levels or small screens (320 CSS pixels). Users have to scroll horizontally to read the text.
    .user-impact = People with visual impairments who zoom in heavily (up to 400%) must scroll in two directions. Reading text becomes extremely tedious or impossible.
    .typical-cause = Using fixed pixel widths (width: 960px) or overflow: hidden, which prevents text from wrapping properly.
    .recommendation = Use responsive design with relative units (%, vw, rem). Avoid fixed widths on containers holding text. Test zooming up to 400%.
    .technical-note = WCAG 2.1 Level AA: No loss of content or function at width 320px (horizontal scroll must be avoided for text-flow containers). Use max-width: 100% and flexbox/grid.

explanation-1-4-11 = Insufficient contrast on UI elements
    .description = Interactive elements such as buttons, input fields, or icons have insufficient contrast against the background. They are therefore hard to recognize for some users.
    .user-impact = Users with limited vision cannot reliably perceive and operate interactive elements.
    .typical-cause = Subtle border colors on form elements, low-contrast icons, focus indicators that are too inconspicuous.
    .recommendation = Design all interactive elements and their states (default, hover, focus) with a contrast ratio of at least 3:1 against the background.
    .technical-note = 3:1 contrast ratio for UI components and graphical objects. Applies to borders, icons, sliders, checkboxes, etc.

explanation-1-4-12 = Content is clipped when text spacing is increased
    .description = When users increase line, word, or letter spacing for better readability, a container with a fixed height or overflow:hidden clips parts of the text.
    .user-impact = Users with reading difficulties or visual impairments who increase spacing for readability lose content or can no longer read it fully.
    .typical-cause = Fixed heights or widths with overflow:hidden on text containers, e.g. cards, buttons, or navigation elements with tightly sized space.
    .recommendation = Design text containers flexibly (min-height instead of a fixed height, no overflow:hidden on text areas) and test against the WCAG minimum spacing values.
    .technical-note = Simulate the WCAG 1.4.12 minimum values (line height 1.5x, paragraph spacing 2x, letter spacing 0.12x, word spacing 0.16x of font size) and check whether containers clip content instead of growing.

explanation-1-4-13 = Suboptimal or inaccessible hover/focus content (tooltips)
    .description = Additional content appearing on hover or focus (such as tooltips) is not accessible or not properly linked in the HTML code.
    .user-impact = Screen reader users do not hear tooltips, keyboard users cannot dismiss them, and they are often completely unusable on touch devices.
    .typical-cause = Missing aria-describedby associations, or lack of keyboard control on custom tooltips.
    .recommendation = Use visible labels or modern, accessible tooltips (dismissible with Escape, linked via aria-describedby, and hoverable).
    .technical-note = Additional content must be (1) hoverable, (2) dismissible (Esc), and (3) persistent. Associate tooltips carrying role="tooltip" with their trigger via aria-describedby.

## 2. Operable

explanation-2-1-1 = Content not operable by keyboard
    .description = Certain features of the website can only be operated with a mouse. Users who rely on the keyboard cannot reach or use these features.
    .user-impact = People with motor impairments who cannot use a mouse, as well as screen reader users, are excluded from these features.
    .typical-cause = Click handlers attached to div/span instead of interactive elements, missing tabindex attributes, JavaScript widgets without keyboard support.
    .recommendation = Implement all interactive elements with native HTML elements (button, a, input). For custom widgets, implement keyboard navigation (Tab, Enter, Escape, arrow keys).
    .technical-note = Prefer native HTML elements. For custom widgets: tabindex="0", keydown/keyup handlers, ARIA roles and states.

explanation-focusable-no-role = Focusable element without an interactive role
    .description = An element can receive keyboard focus, but carries no (or a non-interactive) ARIA role. Assistive technologies therefore cannot tell what is possible once the element receives focus.
    .user-impact = Screen reader users hear no role, or a misleading one (e.g. "region" instead of "dialog" or "button"), when the element receives focus, and cannot tell what interaction is expected.
    .typical-cause = A container — such as a consent banner or a third-party widget — technically receives keyboard focus, without a matching ARIA role or native interactive element behind it.
    .recommendation = Give the element an ARIA role that matches its actual function, or replace it with a native interactive HTML element. Making an element merely focusable without exposing its function to screen readers does not fix the underlying issue.
    .technical-note = Check why the element is focusable (tabindex, third-party script). If it is genuinely interactive: add a matching role (e.g. role="dialog", role="button") and an accessible name. If it has no interaction of its own: remove focusability (tabindex="-1" or delete the tabindex attribute).

explanation-2-1-2 = Keyboard trap — focus cannot leave the element
    .description = At certain points on the website, keyboard focus gets stuck. Users can no longer navigate forward or backward with the keyboard.
    .user-impact = Keyboard users are trapped at this point and cannot reach the rest of the page — the page effectively becomes unusable.
    .typical-cause = Modal dialogs without proper focus management, widgets that capture focus, missing Escape key to dismiss.
    .recommendation = Ensure focus can leave every element using Tab and Shift+Tab. Make modal dialogs dismissible with Escape and restore focus correctly afterwards.
    .technical-note = Use focus trapping only in modals with a correct implementation. Allow Escape to close. Return focus to the triggering element.

explanation-2-4-1 = Missing skip navigation
    .description = The website provides no mechanism to skip recurring blocks of content (e.g. navigation, header) and jump directly to the main content.
    .user-impact = Keyboard and screen reader users must tab through the entire navigation again on every page change before they reach the content.
    .typical-cause = Missing skip link as the first element on the page. WCAG 2.4.1 requires a mechanism to bypass repeated blocks — not necessarily landmarks, but a visible skip link is the most direct solution.
    .recommendation = Add a visible skip link ('Skip to main content') as the first interactive element on the page. HTML5 landmarks (<nav>, <main>) are a useful addition but do not replace the skip link.
    .technical-note = Skip link: <a href="#main" class="skip-link">Skip to main content</a>. Make it visible on focus via CSS :focus {"{"} clip: auto; position: static; {"}"}. Landmarks (<nav>, <main id="main">) additionally improve screen reader navigation.

explanation-2-4-2 = Missing or insufficient page title
    .description = The page has no meaningful title in the browser tab. Users cannot tell which page they are on.
    .user-impact = Screen reader users hear the page title first — without a clear title, they have no orientation. With many tabs open, the page is also hard to identify.
    .typical-cause = Empty <title> tag, generic title ('Home', 'Untitled'), identical title on every page.
    .recommendation = Give each page a unique, descriptive title that clearly states the page content and site context (e.g. 'Contact — Company Name').
    .technical-note = <title>Page content — Site</title>. Make the title a required field in the CMS. Pattern: page-specific + site name.

explanation-2-4-3 = Illogical focus order
    .description = The order in which interactive elements are reached via keyboard does not match the visual or logical order of the page.
    .user-impact = Keyboard users experience confusing navigation — focus jumps back and forth between unrelated areas.
    .typical-cause = Visual layout reordered via CSS without adjusting DOM order, positive tabindex values, dynamically inserted elements placed in the wrong spot.
    .recommendation = Align DOM order with the visual order. Avoid positive tabindex values. Manage focus programmatically for dynamic content.
    .technical-note = Use only tabindex 0 or -1. Do not use CSS order for content-relevant reordering. DOM order = reading order.

explanation-2-4-4 = Unclear or generic link text
    .description = Links on the page use text like 'here', 'more', 'read more' that is not understandable out of context. Users cannot tell where a link leads.
    .user-impact = Screen reader users often navigate via a list of links — generic text like 'more' is meaningless there. Orientation is also harder for all other users.
    .typical-cause = Editorial habit of using 'click here' or 'learn more' as link text. Teaser components with a generic 'Read more'.
    .recommendation = Phrase link text so it is understandable without surrounding context (e.g. 'Web development services' instead of 'learn more').
    .technical-note = Use descriptive link text. If a visually short text is required, use aria-label or aria-labelledby for an extended description.

explanation-link-name-context = Generic link text with possible contextual meaning
    .description = The link text alone is unspecific (e.g. "Start", "next"), but sits next to descriptive text that may explain the link's purpose. WCAG 2.4.4 explicitly allows a link's purpose to be derived from its text together with its context — whether that is the case here can only be confirmed by manual review.
    .user-impact = Whether the link's purpose is understandable to screen reader users depends on whether the surrounding text is actually read out before the link is reached, and whether the connection is unambiguous. Without manual review it is unclear whether users are actually affected.
    .typical-cause = Teaser or card components where a generic button or link text ("Start", "next") directly follows a heading or description text.
    .recommendation = Manually verify whether the surrounding text makes the link's purpose unambiguous both visually and for screen readers. If so, no change is needed. Otherwise, phrase the link text itself descriptively, or supplement it via aria-label/aria-labelledby.
    .technical-note = Context only counts if it is programmatically associated with the link or immediately precedes it (same sentence, same list cell, same heading). When in doubt, use aria-labelledby pointing at the context element to make the association explicit.

explanation-2-4-6 = Missing or insufficient heading structure
    .description = The page uses no headings or has an illogical heading structure. The content outline is not recognizable to assistive technologies.
    .user-impact = Screen reader users cannot navigate the page by headings. The page becomes an undifferentiated block of text — quickly finding relevant sections is impossible.
    .typical-cause = No headings used, only visual formatting (bold, large). Heading hierarchy skipped (e.g. H1 → H3). Multiple or missing H1.
    .recommendation = Build a logical heading hierarchy: exactly one H1 per page, H2 for main sections, H3 for sub-sections. Do not skip levels.
    .technical-note = H1 = page title (once). H2–H6 nested hierarchically. Do not skip levels. Verify the heading outline with a browser extension.

explanation-2-4-7 = Focus indicator not visible
    .description = When navigating with the keyboard, it is not visible which element currently has focus. The visual focus outline is missing or invisible.
    .user-impact = Keyboard users lose orientation — they cannot tell where they are on the page or which element they would activate.
    .typical-cause = CSS reset removes the outline (outline: none / outline: 0). No custom focus style defined. Focus style too inconspicuous.
    .recommendation = Design a clearly visible focus indicator for all interactive elements. At least a 2px outline with sufficient contrast.
    .technical-note = Use :focus-visible instead of :focus for keyboard-only focus. outline: 2px solid #005fcc; outline-offset: 2px; Never use outline: none without an alternative.

explanation-2-4-10 = Missing section headings
    .description = Long content is not broken up by section headings. The page reads as one continuous block with no recognizable structure.
    .user-impact = Users cannot jump to specific content and have to read long passages in full to find relevant information.
    .typical-cause = Long content pages without sub-headings. Editorial content presented as running text with no structure.
    .recommendation = Break up long content with meaningful sub-headings. Add a heading every 2–3 paragraphs.
    .technical-note = Use correct heading hierarchy (H2, H3...) within sections. WAI-ARIA: use role="heading" only as a fallback; prefer native HTML.

explanation-2-5-3 = Visible text does not match the accessible name
    .description = The visible text of a control does not match the name announced by assistive technologies. Voice-control users cannot address the element.
    .user-impact = Users relying on voice control cannot activate elements by voice command because the announced name does not match the visible text.
    .typical-cause = aria-label overrides the visible text with different wording. Visible text and accessible name are phrased differently.
    .recommendation = Make sure the accessible name (aria-label) contains the visible text or is identical to it.
    .technical-note = aria-label must include the visible text as a substring. Better: omit aria-label entirely when the visible text is sufficient.

explanation-2-5-8 = Insufficient click target size
    .description = Interactive elements such as buttons, links, or icons have a clickable area smaller than 24×24 CSS pixels.
    .user-impact = Users with motor impairments or on touchscreens easily miss small target areas and accidentally trigger the wrong action.
    .typical-cause = Icon buttons without extra padding, tightly packed link or icon lists, small form controls in the mobile layout.
    .recommendation = Increase clickable areas to at least 24×24 CSS pixels, e.g. via padding, or ensure sufficient spacing to neighboring targets.
    .technical-note = Exceptions apply to, among others, inline links within body text and cases where an equivalent, larger target for the same function exists.

## 3. Understandable

explanation-3-1-1 = Missing page language declaration
    .description = The primary language of the page is not declared in the HTML. Screen readers and translation tools cannot detect the language automatically.
    .user-impact = Screen readers read out the text with the wrong pronunciation — e.g. German content is read with English phonetics, which is unintelligible.
    .typical-cause = Missing lang attribute on the <html> tag. Often forgotten in templates.
    .recommendation = Set the lang attribute on the <html> tag (e.g. lang="en" for English). On multilingual pages, additionally mark sections with a lang attribute.
    .technical-note = <html lang="en"> for English-language pages. Use ISO 639-1 language codes. For foreign-language sections: <span lang="en">...</span>.

explanation-3-2-1 = Unexpected change of context on focus
    .description = When an element receives keyboard focus, the context changes unexpectedly (e.g. the page changes, a new window opens).
    .user-impact = Keyboard and screen reader users experience unpredictable page changes. This is confusing and disrupts their workflow.
    .typical-cause = JavaScript events on onfocus that trigger navigation or DOM changes. Select elements that perform an action on focus.
    .recommendation = Trigger context changes only via explicit user actions (click, Enter), not by focus change alone.
    .technical-note = No onchange/onfocus handlers for navigation. Use a submit button with select menus instead of auto-submit.

explanation-3-2-2 = Unexpected change of context on input
    .description = When a user enters data into a form element or makes a selection, the page context changes unexpectedly.
    .user-impact = Users experience confusing changes while filling out forms. Unannounced changes are especially problematic for screen reader users.
    .typical-cause = Auto-submit on form changes. Page redirect on select choice without confirmation.
    .recommendation = Process form changes only after an explicit confirmation (submit button). Announce up front when an input triggers immediate changes.
    .technical-note = No auto-submit on onchange. Use explicit submit buttons. Alternatively: an ARIA live region for an advance notice.

explanation-3-3-2 = Missing labels and instructions on forms
    .description = Form fields have no visible label or instruction. Users do not know what input is expected.
    .user-impact = All users, especially people with cognitive impairments and screen reader users, cannot complete forms correctly.
    .typical-cause = Missing <label> elements, placeholder text used instead of visible labels, no indication of required fields or expected input format.
    .recommendation = Provide every form field with a visible label linked via <label>. Mark required fields. For special formats (e.g. date), state the expected format.
    .technical-note = <label for="id"> on all form fields. Required fields: required + aria-required="true". Do not use placeholders as the only label.

## 4. Robust

explanation-4-1-2 = Missing name/role on controls
    .description = Interactive elements (buttons, links, form fields) have no accessible name or recognizable role. Assistive technologies cannot convey what the element is.
    .user-impact = Screen reader users hear, for example, only 'button' with no description of its function, or a clickable element is not recognized as interactive at all.
    .typical-cause = Buttons or links with no text or aria-label. Icon-only buttons without an accessible name. Custom widgets without ARIA roles.
    .recommendation = Give every interactive element a clear, accessible name. Prefer native HTML elements. For icon buttons: use aria-label.
    .technical-note = Buttons: visible text or aria-label. Links: descriptive link text. Inputs: associated <label>. Custom widgets: role + aria-label + ARIA states.

## SEO rules (no WCAG criterion)

explanation-seo-headings-long-heading = Heading too long
    .description = Headings on the page exceed the recommended length of 70 characters. Long headings are truncated in search results and harder for readers to scan.
    .user-impact = In search results and social media the headings are cut off, reducing click-through appeal. Readers need more cognitive effort to understand the content at a glance.
    .typical-cause = Editorial text is used as a heading without trimming. CMS heading fields have no character limit enforced.
    .recommendation = Shorten headings to under 70 characters and lead with the core message. Longer descriptions belong in the body text.
    .technical-note = Google typically displays 50–60 characters in the title tag. There is no hard limit for H2/H3, but readability drops noticeably beyond 70 characters. CMS-level validation is recommended.

explanation-seo-headings-skipped-level = Skipped heading level
    .description = The headings on the page skip a level — for example, an H1 is directly followed by an H3 without an H2 in between. This breaks the logical hierarchy that search engines and screen readers use for navigation.
    .user-impact = Screen reader users navigate pages via headings. Missing levels interrupt this path and make orientation harder. Search engines rate pages with consistent heading structure as higher quality content.
    .typical-cause = CMS templates or components hard-code heading levels (e.g. always H3 in footer columns) regardless of page context. Developers choose levels for visual style rather than hierarchy.
    .recommendation = Maintain heading hierarchy from H1 downward without gaps: H1 → H2 → H3. To make a heading look smaller visually, use CSS classes rather than changing the HTML level.
    .technical-note = WCAG 1.3.1 (Info and Relationships, Level A) requires structure to be programmatically determinable. Skipped levels violate this requirement. Screen readers like NVDA/JAWS jump with H-keys — gaps create dead spots.

explanation-seo-headings-missing-h1 = Missing H1 heading
    .description = The page has no H1 heading. Without one, its purpose isn't immediately recognizable to search engines or screen reader users.
    .user-impact = Screen reader users who orient via the heading list find no entry point naming the page's main topic.
    .typical-cause = The page template doesn't emit an H1, or a component that normally supplies it was removed or misconfigured.
    .recommendation = Give every page exactly one H1 naming the main topic, placed near the top of the content.
    .technical-note = Not a WCAG failure on its own: WCAG 2.2 asks in 1.3.1 (technique H42) and 2.4.6 that headings are marked up as such and describe their topic, not for an H1. One H1 naming the page topic is a common recommendation for orientation and search engines.

explanation-seo-headings-multiple-h1 = Multiple H1 headings
    .description = The page contains more than one H1 heading. That is allowed, but it can blur the page's main focus when it is unclear which H1 names the page topic.
    .user-impact = Users navigating by the heading list see several top-level headings. Whether that gets in the way depends on whether they name the page topic and meaningful main sections — only a manual check can tell.
    .typical-cause = A reusable component (e.g. a hero or logo block) emits its own H1 regardless of whether the page already has one.
    .recommendation = Check that the headings structure the content meaningfully. One H1 for the page topic is recommended; reusable components should make their heading level configurable rather than hard-coding H1.
    .technical-note = Not a WCAG failure on its own: WCAG 2.2 (1.3.1, technique H42; 2.4.6) and HTML allow several H1s. A failure arises only when the hierarchy misstates the structure. One H1 per page is a recommendation for orientation and search engines.

explanation-seo-headings-empty-heading = Empty heading
    .description = A heading on the page contains no text. Empty headings cause navigation problems for screen reader users and are treated as a weak signal by search engines.
    .user-impact = Screen reader users jumping by heading land on an unnamed point with no recognizable content.
    .typical-cause = A component renders its heading element regardless of whether the CMS title field was filled in, or the text is conveyed only via a background image instead of real text.
    .recommendation = Remove empty heading elements or fill them with real, descriptive text. Components should only render the heading element when a title is present.
    .technical-note = WCAG 1.3.1 (Info and Relationships, Level A) and 2.4.6 (Headings and Labels, Level AA) require headings to carry recognizable, programmatically available text.

## Rule-specific texts keyed by an axe id

explanation-server-side-image-map = Server-side image map used
    .description = Server-side image map used. Keyboard and screen reader users cannot target the individual image regions.
    .user-impact = Keyboard and screen reader users cannot target the individual image regions.
    .typical-cause = ismap attribute on an image without an equivalent client-side alternative.
    .recommendation = Replace the server-side image map with a client-side image map (<map>/<area>) or plain text links.
    .technical-note = ismap attribute on an image without an equivalent client-side alternative.

explanation-svg-img-alt = Missing alternative text on SVG graphics
    .description = Missing alternative text on SVG graphics. Screen reader users receive no information about the graphic's content.
    .user-impact = Screen reader users receive no information about the graphic's content.
    .typical-cause = SVG without a <title> child element, aria-label, or aria-labelledby.
    .recommendation = Add a <title> as the first child of the <svg>, use aria-label, or hide purely decorative SVGs with aria-hidden="true".
    .technical-note = SVG without a <title> child element, aria-label, or aria-labelledby.

explanation-area-alt = Missing alternative text on image map areas
    .description = Missing alternative text on image map areas. Screen reader users are not told where a clickable image region leads.
    .user-impact = Screen reader users are not told where a clickable image region leads.
    .typical-cause = <area> element without an alt attribute.
    .recommendation = Add an alt attribute to the <area> element describing its destination.
    .technical-note = <area> element without an alt attribute.

explanation-input-image-alt = Missing alternative text on image buttons
    .description = Missing alternative text on image buttons. Screen reader users are not told what action the button triggers.
    .user-impact = Screen reader users are not told what action the button triggers.
    .typical-cause = <input type="image"> without an alt attribute.
    .recommendation = Add an alt attribute to the <input type="image"> element.
    .technical-note = <input type="image"> without an alt attribute.

explanation-object-alt = Missing text alternative on embedded objects
    .description = Missing text alternative on embedded objects. Screen reader users receive no information about the embedded object's content.
    .user-impact = Screen reader users receive no information about the embedded object's content.
    .typical-cause = <object> without text content or aria-label.
    .recommendation = Provide a text alternative inside the <object> element or via aria-label.
    .technical-note = <object> without text content or aria-label.

explanation-form-field-group = Missing grouping for related form fields
    .description = Missing grouping for related form fields. Screen reader users cannot tell that several fields belong to one question/group.
    .user-impact = Screen reader users cannot tell that several fields belong to one question/group.
    .typical-cause = Radio buttons/checkboxes without an enclosing <fieldset> with <legend>.
    .recommendation = Wrap related radio buttons or checkboxes in a <fieldset> with a <legend>, or use role="group" with aria-labelledby.
    .technical-note = Radio buttons/checkboxes without an enclosing <fieldset> with <legend>.

explanation-landmark-banner-is-top-level = Banner landmark is nested
    .description = Banner landmark is nested. Screen reader users who jump to the header via landmark navigation land in an unexpected context.
    .user-impact = Screen reader users who jump to the header via landmark navigation land in an unexpected context.
    .typical-cause = <header>/role="banner" is a descendant of another landmark (e.g. <main>).
    .recommendation = Move the <header> / role="banner" element to the top level of the page, outside any other landmark.
    .technical-note = <header>/role="banner" is a descendant of another landmark (e.g. <main>).

explanation-landmark-contentinfo-is-top-level = Contentinfo landmark is nested
    .description = Contentinfo landmark is nested. Screen reader users who jump to the footer via landmark navigation land in an unexpected context.
    .user-impact = Screen reader users who jump to the footer via landmark navigation land in an unexpected context.
    .typical-cause = <footer>/role="contentinfo" is a descendant of another landmark.
    .recommendation = Move the <footer> / role="contentinfo" element to the top level of the page, outside any other landmark.
    .technical-note = <footer>/role="contentinfo" is a descendant of another landmark.

explanation-landmark-main-is-top-level = Main landmark is nested
    .description = Main landmark is nested. Screen reader users who jump to the main content via landmark navigation land in an unexpected context.
    .user-impact = Screen reader users who jump to the main content via landmark navigation land in an unexpected context.
    .typical-cause = <main>/role="main" is a descendant of another landmark.
    .recommendation = Move the <main> / role="main" element to the top level of the page, outside any other landmark.
    .technical-note = <main>/role="main" is a descendant of another landmark.

explanation-landmark-no-duplicate-banner = Multiple banner landmarks
    .description = Multiple banner landmarks. Screen reader users cannot uniquely identify the page's 'header' region.
    .user-impact = Screen reader users cannot uniquely identify the page's 'header' region.
    .typical-cause = More than one element with role="banner" (implicit or explicit).
    .recommendation = Ensure the page has at most one <header> / role="banner" element.
    .technical-note = More than one element with role="banner" (implicit or explicit).

explanation-landmark-no-duplicate-contentinfo = Multiple contentinfo landmarks
    .description = Multiple contentinfo landmarks. Screen reader users cannot uniquely identify the page's 'footer' region.
    .user-impact = Screen reader users cannot uniquely identify the page's 'footer' region.
    .typical-cause = More than one element with role="contentinfo" (implicit or explicit).
    .recommendation = Ensure the page has at most one <footer> / role="contentinfo" element.
    .technical-note = More than one element with role="contentinfo" (implicit or explicit).

explanation-landmark-no-duplicate-main = Multiple main landmarks
    .description = Multiple main landmarks. Screen reader users who jump to the main content cannot tell which region is meant.
    .user-impact = Screen reader users who jump to the main content cannot tell which region is meant.
    .typical-cause = More than one element with role="main" (implicit or explicit).
    .recommendation = Ensure the page has exactly one <main> / role="main" element.
    .technical-note = More than one element with role="main" (implicit or explicit).

explanation-landmark-banner-present = Missing banner landmark
    .description = Missing banner landmark. Assistive technologies cannot identify the site header.
    .user-impact = Assistive technologies cannot identify the site header.
    .typical-cause = No element with role="banner" (implicit via <header> or explicit) present.
    .recommendation = Add a top-level <header> element (or an element with role="banner") that wraps the site header content.
    .technical-note = No element with role="banner" (implicit via <header> or explicit) present.

explanation-label-title-only = Form field labeled only via the title attribute
    .description = Form field labeled only via the title attribute. The title attribute is read inconsistently or not at all by many screen readers; sighted users only see it on hover.
    .user-impact = The title attribute is read inconsistently or not at all by many screen readers; sighted users only see it on hover.
    .typical-cause = Form element without <label>/aria-label, name derived only from title.
    .recommendation = Add a visible <label> element or aria-label instead of relying on the title attribute.
    .technical-note = Form element without <label>/aria-label, name derived only from title.

explanation-click-events-have-key-events = Click handler without keyboard equivalent
    .description = Click handler without keyboard equivalent. Keyboard users cannot trigger the function.
    .user-impact = Keyboard users cannot trigger the function.
    .typical-cause = onclick on a <div>/<span> without onkeydown/onkeyup, tabindex, or a matching role.
    .recommendation = Add a keydown/keyup handler that responds to Enter/Space, add tabindex="0" and an appropriate ARIA role, or use a native <button> element instead.
    .technical-note = onclick on a <div>/<span> without onkeydown/onkeyup, tabindex, or a matching role.

explanation-landmark-one-main = No main-content region as a bypass target
    .description = No main-content region as a bypass target. Keyboard users have no reliable target to bypass repeated navigation blocks.
    .user-impact = Keyboard users have no reliable target to bypass repeated navigation blocks.
    .typical-cause = No element with role="main" (implicit via <main> or explicit) present.
    .recommendation = Add a <main> element or an element with role="main" so keyboard users have a reliable bypass target.
    .technical-note = No element with role="main" (implicit via <main> or explicit) present.

explanation-skip-link = Missing skip link
    .description = Missing skip link. Keyboard users must tab through the entire navigation on every page change.
    .user-impact = Keyboard users must tab through the entire navigation on every page change.
    .typical-cause = No link with a recognizable 'skip'/'to content' text or anchor to #main/#content at the top of the page.
    .recommendation = Add a visually hidden or visible link at the top of the page pointing to the main content anchor, e.g. <a href="#main">Skip to main content</a>.
    .technical-note = No link with a recognizable 'skip'/'to content' text or anchor to #main/#content at the top of the page.

explanation-focus-visible-outline-none = Focus indicator removed via CSS
    .description = Focus indicator removed via CSS. Keyboard users cannot see which element currently has focus.
    .user-impact = Keyboard users cannot see which element currently has focus.
    .typical-cause = outline: none/0 in :focus rules without an alternative visible focus indicator.
    .recommendation = Do not remove the default focus outline (outline: none/0) without providing an equally visible replacement, e.g. a custom outline or box-shadow on :focus-visible.
    .technical-note = outline: none/0 in :focus rules without an alternative visible focus indicator.

explanation-scrollable-region-focusable = Scrollable region not reachable by keyboard
    .description = A region with its own scrollbar can only be scrolled with a mouse or by touch: it is not focusable and contains nothing that brings focus into it.
    .user-impact = Keyboard users cannot reach the content below the visible part.
    .typical-cause = A fixed height with overflow: auto on menu, text or table containers that hold only text.
    .recommendation = Make the region focusable with tabindex="0" and give it a name (role="region" with aria-label), or remove the fixed height.
    .technical-note = Like axe-core scrollable-region-focusable: overflow of more than 13 px with overflow: auto/scroll, visible text content, neither the element nor a descendant keyboard-focusable. Chrome makes such regions focusable on its own since version 130, other browsers do not.

explanation-input-error-message = Invalid field without an accessible error description
    .description = Invalid field without an accessible error description. Screen reader users learn that an error exists, but not what it is.
    .user-impact = Screen reader users learn that an error exists, but not what it is.
    .typical-cause = aria-invalid="true" without a description/aria-describedby pointing to a visible error element.
    .recommendation = Add aria-describedby pointing to an element containing the error message, or use aria-errormessage.
    .technical-note = aria-invalid="true" without a description/aria-describedby pointing to a visible error element.

explanation-duplicate-id-aria = ARIA reference points to an ambiguous ID
    .description = ARIA reference points to an ambiguous ID. Assistive technologies cannot reliably determine which element is referenced.
    .user-impact = Assistive technologies cannot reliably determine which element is referenced.
    .typical-cause = The referenced ID occurs more than once in the DOM.
    .recommendation = Ensure the ID referenced by aria-owns/aria-describedby/aria-labelledby is unique in the document, or reference a different, uniquely identified element.
    .technical-note = The referenced ID occurs more than once in the DOM.

explanation-aria-label = Interactive element without a meaningful name
    .description = Interactive element without a meaningful name. Screen reader users are not told what the element does.
    .user-impact = Screen reader users are not told what the element does.
    .typical-cause = Missing or purely symbolic aria-label/aria-labelledby/text content.
    .recommendation = Add aria-label, aria-labelledby, or visible text content that describes the element's purpose (not just its icon).
    .technical-note = Missing or purely symbolic aria-label/aria-labelledby/text content.

explanation-title-only-description = Native title attribute as the only description
    .description = An interactive element is described only by the native title attribute. This is best practice, not a WCAG 4.1.2 failure: title counts as an accessible name.
    .user-impact = The tooltip only appears on mouse hover; with a keyboard and on touch devices the element's purpose stays invisible.
    .typical-cause = Icon buttons, logo links or search fields without visible text, where title serves as the label.
    .recommendation = Add a visible label, or set the name via aria-label/aria-labelledby or a <label>; use title only for supplementary hints.
    .technical-note = Best practice, anchored to WCAG 4.1.2. Not 1.4.13: the browser tooltip of the title attribute is controlled by the user agent and exempt there. Checks button, a[href] and input with title and without text content, aria-label and aria-labelledby.

explanation-aria-required-children = Required child elements of an ARIA role are missing
    .description = Required child elements of an ARIA role are missing. Assistive technologies cannot resolve the relationship between the widget's parts.
    .user-impact = Assistive technologies cannot resolve the relationship between the widget's parts.
    .typical-cause = e.g. role="tablist" without corresponding role="tabpanel" elements.
    .recommendation = Add the child elements required by the ARIA role, e.g. role="tabpanel" elements for every tab in a role="tablist".
    .technical-note = e.g. role="tablist" without corresponding role="tabpanel" elements.

explanation-dialog-name = Dialog without an accessible name
    .description = Dialog without an accessible name. Screen reader users are not told what the dialog is about when it opens.
    .user-impact = Screen reader users are not told what the dialog is about when it opens.
    .typical-cause = Dialog element without aria-label or aria-labelledby pointing to the dialog title.
    .recommendation = Add aria-labelledby pointing to the dialog's visible title, or use aria-label.
    .technical-note = Dialog element without aria-label or aria-labelledby pointing to the dialog title.

explanation-control-missing-label = Form control, link, or button without an accessible name
    .description = Form control, link, or button without an accessible name. Screen reader users are not told what input is expected or where a link/button leads.
    .user-impact = Screen reader users are not told what input is expected or where a link/button leads.
    .typical-cause = Form control/link/button without <label>, aria-label, aria-labelledby, or text content.
    .recommendation = Add a <label> element with a matching 'for' attribute (form controls) or text content (links/buttons), or use aria-label/aria-labelledby.
    .technical-note = Form control/link/button without <label>, aria-label, aria-labelledby, or text content.

explanation-link-as-button = Link used as a button substitute without a matching role
    .description = Link used as a button substitute without a matching role. Assistive technologies announce the element as a link even though it behaves like a button — keyboard users expect the wrong interaction.
    .user-impact = Assistive technologies announce the element as a link even though it behaves like a button — keyboard users expect the wrong interaction.
    .typical-cause = <a href="#">/<a href="javascript:..."> with onclick, without role="button".
    .recommendation = Add role="button" to the link (and keyboard handling for Space), or use a real <button> element instead.
    .technical-note = <a href="#">/<a href="javascript:..."> with onclick, without role="button".

explanation-redundant-role = Redundant ARIA role
    .description = Redundant ARIA role. No immediate user harm, but an unnecessary risk of inconsistency in future changes.
    .user-impact = No immediate user harm, but an unnecessary risk of inconsistency in future changes.
    .typical-cause = role attribute duplicates the tag's native implicit role.
    .recommendation = Remove the redundant role attribute; the native HTML element already exposes this role implicitly.
    .technical-note = role attribute duplicates the tag's native implicit role.

explanation-summary-name = Summary element without an accessible name
    .description = Summary element without an accessible name. Screen reader users are not told what the disclosure section contains before opening it.
    .user-impact = Screen reader users are not told what the disclosure section contains before opening it.
    .typical-cause = <summary> without text content or aria-label.
    .recommendation = Add text content to the <summary> element describing what the disclosure section contains.
    .technical-note = <summary> without text content or aria-label.

## Missing navigation and contentinfo landmark (plan 56)

explanation-landmark-navigation-present = Missing navigation landmark
    .description = The page has no area marked up as navigation (<nav> or role="navigation").
    .user-impact = Screen reader users cannot jump straight to the navigation and have to search for it in the page content.
    .typical-cause = The menu is built from <div> and <ul> elements without an enclosing <nav>.
    .recommendation = Wrap the main menu (and other navigation blocks) in a <nav> element; tell several navigations apart with aria-label.
    .technical-note = Pages without any navigation (such as a single landing page) do not need a navigation landmark.

explanation-landmark-contentinfo-present = Missing contentinfo landmark
    .description = The page has no area marked up as the page footer (a top-level <footer> or role="contentinfo").
    .user-impact = Screen reader users cannot jump straight to the footer with contact and legal information.
    .typical-cause = The footer is a <div>, or the <footer> sits inside <main>, <article> or <section>, where it does not count as the page footer.
    .recommendation = Mark up the page footer as a <footer> directly under <body>, outside <main>.
    .technical-note = Only a <footer> outside <main>, <article>, <aside>, <nav> and <section> implicitly gets role="contentinfo".

