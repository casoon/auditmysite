# Texts of the SEO module detail presentation of the single-page report
# (`output::builder::single::module_details::seo`, `seo-details-*`), #936.
# Which message applies, the rating bands and how lists and values are joined
# stay in Rust. The labels "Description", "Schema.org content type" and
# "Language" are the `pdf-seo-*` messages, "Yes"/"No" the `yes`/`no` messages
# of report.ftl; "present"/"missing" are the `seo-profile-detail-*` messages of
# seo-issues.ftl.

## Meta tags
seo-details-meta-title = Title
seo-details-meta-also-accessibility = { $message } — also counted under accessibility (WCAG { $criterion })
seo-details-meta-viewport-elsewhere = { $message } — counted under mobile and accessibility, not in the SEO score

## Headings and social tags
seo-details-heading-summary = { $h1 ->
        [one] { $h1 } H1 heading
       *[other] { $h1 } H1 headings
    }, { $total ->
        [one] { $total } heading
       *[other] { $total } headings
    } total, { $issues ->
        [one] { $issues } issue
       *[other] { $issues } issues
    }
seo-details-social-summary = Open Graph: { $og }, Twitter Card: { $twitter }, Completeness: { $completeness }%

## Schema inventory
seo-details-schema-questions = { $count ->
        [one] { $count } question
       *[other] { $count } questions
    }
seo-details-schema-with-search = { $name } (search)
seo-details-schema-regions = { $base } · { $count ->
        [one] { $count } region
       *[other] { $count } regions
    }
seo-details-schema-levels = { $count ->
        [one] { $count } level
       *[other] { $count } levels
    }

## Signal strength ratings
seo-details-signal-excellent = Excellent
seo-details-signal-good = Good
seo-details-signal-partial = Partial
seo-details-signal-minimal = Minimal
seo-details-signal-missing = Missing

## Content identity and page profile
seo-details-identity-page-title = Page title
seo-details-identity-topic-hints = Topic hints
seo-details-identity-no-topic-hints = No clear topic hints detected
seo-details-profile-page-type = SEO page type
seo-details-profile-characteristics = Characteristics
seo-details-profile-no-characteristics = No defining characteristics detected
seo-details-profile-classification = Classification
seo-details-profile-recommendation = Recommendation

## Technical summary
seo-details-language-tag = Language tag
seo-details-word-count = Word count
seo-details-internal-links = Internal links
seo-details-external-links = External links
seo-details-dofollow-links = Dofollow links
seo-details-nofollow-links = Nofollow links
seo-details-not-detected = Not detected
seo-details-amp-complete = Detected, basic signals complete
seo-details-amp-issues = Detected, { $count ->
        [one] { $count } basic issue
       *[other] { $count } basic issues
    }

## Tracking summary
seo-details-google-fonts = Google Fonts (external)
seo-details-google-fonts-used = Yes ({ $sources })
seo-details-tracking-cookies = Tracking cookies
seo-details-none-detected = None detected
seo-details-cookie-inventory = Cookie inventory
seo-details-cookie-inventory-not-collected = Not collected: the browser cookie jar could not be read
seo-details-no-cookies = No cookies detected
seo-details-categories = categories: { $summary }
seo-details-no-storage-keys = No keys detected
seo-details-tracking-signals = Tracking signals
seo-details-consent-cookies = Consent cookies
seo-details-consent-summary = { $before } before interaction, { $after } after interaction, { $added } new
seo-details-not-collected = Not collected
seo-details-zaraz-detected = Detected ({ $signals })
seo-details-zaraz-detected-no-signals = Detected
seo-details-browser-storage = Browser Storage
