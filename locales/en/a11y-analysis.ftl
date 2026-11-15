# Texts of the accessibility analysis layer, #936: the screen reader issue
# messages (`screen_reader::analyzer`) and the recognized-pattern messages
# (`patterns::pattern_message_text`, `patterns::easy_language`). The analysis
# stores canonical English in the JSON (`en = true`), the PDF calls the same
# functions with the run language (#406). Which message applies, and the
# detection (page-language stopwords), stay in Rust.

## Screen reader issues: interactive names
a11y-analysis-sr-non-descriptive = Interactive name "{ $name }" is not meaningful without context.
a11y-analysis-sr-duplicated-name = Accessible name "{ $name }" repeats "{ $half }" twice in a row. A screen reader announces it as "{ $name }", which sounds like a glitch.
a11y-analysis-sr-icon-font = Link or button name contains icon-font characters (Unicode Private Use Area). Screen readers read these out as cryptic character codes.
a11y-analysis-sr-empty-interactive = Interactive elements without an accessible name are not announced intelligibly by a screen reader.
a11y-analysis-sr-empty-form-labels = Form fields without a label are not intelligible in the screen reader's form list.

## Screen reader issues: link list
a11y-analysis-sr-link-empty = Empty link text
a11y-analysis-sr-link-context-dependent = Context-dependent link text
a11y-analysis-sr-duplicate-links = { $quality } occurs { $count } times and clutters the screen reader's link list.

## Screen reader issues: reading sequence and navigation
a11y-analysis-sr-desert = Long section from sequence position { $start } without a landmark, heading or focus target ({ $entries } entries).
a11y-analysis-sr-tab-stops = { $count } tab stops without a detectable skip link hamper keyboard and screen reader navigation.

## Screen reader issues: headings
a11y-analysis-sr-heading-level-unknown = an unknown level
a11y-analysis-sr-heading-skipped = Heading level is skipped: { $level } at sequence position { $seq }.
a11y-analysis-sr-heading-first-not-h1 = First heading in the document is H{ $level } instead of H1 (sequence position { $seq }).
a11y-analysis-sr-heading-h1-late = H1 first appears at sequence position { $seq } — after an H2/H3 heading.

## Screen reader issues: landmarks
a11y-analysis-sr-landmarks-unlabeled-duplicate = Several landmarks of the same type without a name are indistinguishable in the screen reader's landmark list.
a11y-analysis-sr-landmark-main-missing = No main landmark detectable in the screen reader's landmark list.
a11y-analysis-sr-landmark-missing-banner = No header area (banner landmark) present. Screen reader users cannot reach the page header through landmark navigation; its content stays reachable in reading mode.
a11y-analysis-sr-landmark-missing-navigation = No navigation landmark present. Screen reader users cannot jump to the navigation through landmark navigation; the links stay reachable in reading and tab order.
a11y-analysis-sr-landmark-missing-contentinfo = No footer landmark (contentinfo) present. Screen reader users cannot reach the footer through landmark navigation; its content stays reachable in reading mode.
a11y-analysis-sr-navigation-behind-disclosure = No navigation landmark present in the captured accessibility tree, but a disclosure/menu trigger was recognized — the navigation is likely collapsed rather than missing. Confirm it becomes reachable once expanded.

## Pattern messages: counted nouns
a11y-analysis-pattern-nav-landmark-count =
    { $count ->
        [one] 1 navigation landmark
       *[other] { $count } navigation landmarks
    }
a11y-analysis-pattern-landmark-count =
    { $count ->
        [one] 1 landmark
       *[other] { $count } landmarks
    }
a11y-analysis-pattern-native-disclosure-count =
    { $count ->
        [one] 1 native disclosure trigger
       *[other] { $count } native disclosure triggers
    }
a11y-analysis-pattern-disclosure-count =
    { $count ->
        [one] 1 disclosure trigger
       *[other] { $count } disclosure triggers
    }
a11y-analysis-pattern-dialog-count =
    { $count ->
        [one] 1 dialog
       *[other] { $count } dialogs
    }
a11y-analysis-pattern-tablist-count =
    { $count ->
        [one] 1 tablist
       *[other] { $count } tablists
    }
a11y-analysis-pattern-tab-count =
    { $count ->
        [one] 1 tab
       *[other] { $count } tabs
    }
a11y-analysis-pattern-accordion-count =
    { $count ->
        [one] 1 accordion trigger
       *[other] { $count } accordion triggers
    }
a11y-analysis-pattern-field-count =
    { $count ->
        [one] 1 required field
       *[other] { $count } required fields
    }

## Pattern messages
a11y-analysis-pattern-navigation-unnamed = { $landmarks } found, but { $unnamed } have no accessible name. Screen reader users cannot distinguish them.
a11y-analysis-pattern-navigation-without-links = { $landmarks } found, but none contain link children. Pattern is structurally incomplete.
a11y-analysis-pattern-navigation = Semantic main navigation recognized ({ $landmarks } with native links). Good foundation for keyboard and screen reader use.
a11y-analysis-pattern-disclosure-native = { $native } (<details>/<summary>) — semantics come from the element, no ARIA needed.
a11y-analysis-pattern-disclosure-complete = { $triggers } with aria-expanded and aria-controls — well-formed pattern.
a11y-analysis-pattern-disclosure-partial = { $triggers } with aria-expanded ({ $controls } with aria-controls). Controls relationship strengthens screen-reader announcements.
a11y-analysis-pattern-disclosure-native-extra =
    { $count ->
        [one] Plus 1 native <details>/<summary> trigger.
       *[other] Plus { $count } native <details>/<summary> triggers.
    }
a11y-analysis-pattern-modal-dialog = { $dialogs } detected, { $well_formed } structurally well-formed (accessible name, aria-modal, focusable descendants).
a11y-analysis-pattern-tab-list = { $tablists } with { $tabs }; { $selected } have aria-selected, { $controls } declare aria-controls.
a11y-analysis-pattern-skip-link-first = Skip link recognized and correctly positioned as first focusable element ("{ $name }").
a11y-analysis-pattern-skip-link-not-first = Skip link detected ("{ $name }") but not the first focusable element — keyboard users may have to tab past other content first.
a11y-analysis-pattern-accordion = { $triggers }; { $controls } with aria-controls; { $non_button } non-button triggers.
a11y-analysis-pattern-form = { $fields } detected; submit trigger: { $trigger }
a11y-analysis-pattern-add-to-cart = Add-to-cart trigger detected: { $trigger }
a11y-analysis-pattern-quantity-stepper = Spinbutton quantity control detected: { $name }
a11y-analysis-pattern-unnamed-button = (unnamed button)
a11y-analysis-pattern-unnamed = (unnamed)
a11y-analysis-pattern-easy-language = The page offers itself in, or links to, an easy-language ("Leichte Sprache") version of its content.
