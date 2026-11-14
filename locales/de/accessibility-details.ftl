# Texts of the accessibility detail sections of the single-page report
# (`output::pdf::detail_modules::accessibility`, `a11y-details-*`), #936:
# the keyboard accessibility journey, the screen-reader reading order and the
# BFSG callout. Which message applies, the deduplication and the score format
# stay in Rust. The severity labels of the screen-reader findings are the
# `severity-*` messages of report.ftl.

## Keyboard accessibility journey
a11y-details-journey-title = Tastatur-Journey zur Barrierefreiheit
a11y-details-journey-overview-critical = { $critical } kritische und { $high } hohe Befunde aus Tastatur- und Zustandswechsel-Tests.
a11y-details-journey-overview-high = { $count ->
        [one] { $count } interaktiver Befund mit hoher Schwere gefunden.
       *[other] { $count } interaktive Befunde mit hoher Schwere gefunden.
    }
a11y-details-journey-overview-minor = { $count } { $count ->
        [one] kleinerer interaktiver Befund
       *[other] kleinere interaktive Befunde
    } — keine kritischen oder hohen Barrieren erkannt.
a11y-details-journey-more = { $count ->
        [one] { $count } weiterer interaktiver Befund im JSON-Report.
       *[other] { $count } weitere interaktive Befunde im JSON-Report.
    }
a11y-details-journey-sequences = Geprüfte Journey-Sequenzen
a11y-details-journey-steps = { $count ->
        [one] { $count } Schritt
       *[other] { $count } Schritte
    }
a11y-details-journey-disclaimer = Diese Tests prüfen, ob Browser, DOM, Fokus und Accessibility Tree eine robuste Grundlage für Screenreader-Nutzung liefern. Sie simulieren nicht den exakten Output von NVDA, JAWS oder VoiceOver.

## Journey labels (internal journey ids)
a11y-details-journey-label-tab-walk = Tab-Durchlauf
a11y-details-journey-label-spa-navigation = SPA-Navigation
a11y-details-journey-label-skip-link = Sprunglink
a11y-details-journey-label-disclosure = Aufklapp-Menü
a11y-details-journey-label-menu = Menü
a11y-details-journey-label-modal = Dialog
a11y-details-journey-label-tabs = Tab-Panel
a11y-details-journey-label-form-error = Formularfehler
a11y-details-journey-label-add-to-cart = In den Warenkorb
a11y-details-journey-label-quantity-stepper = Mengenauswahl

## Screen reader reading order
a11y-details-sr-title = Screenreader-Lesereihenfolge
a11y-details-sr-intro = Wie die Seite Screenreader-Nutzern angekündigt wird: Lesereihenfolge, Landmark- und Heading-Qualität sowie zugängliche Namen. Strukturelle Prüfungen am Accessibility Tree, keine Simulation von NVDA, JAWS oder VoiceOver.
a11y-details-sr-heading-quality = Heading-Qualität
a11y-details-sr-landmark-quality = Landmark-Qualität
a11y-details-sr-name-quality = Namens-Qualität
a11y-details-sr-announced-nodes = Angekündigte Knoten
a11y-details-sr-tab-stops = Tab-Stopps
a11y-details-sr-score-note = Die drei Qualitätswerte nutzen eine Skala von 0–100 (höher ist besser) und verdichten erkannte Überschriftenhierarchie, Landmark-Struktur und zugängliche Namen. Angekündigte Knoten und Tab-Stopps sind reine Anzahlen, keine Qualitätswerte.
a11y-details-sr-findings-title = Screenreader-Befunde

## BFSG callout (never a conformance verdict)
a11y-details-bfsg-no-violations = BFSG-relevante technische Kriterien: im automatisierten Prüfumfang keine Verstöße erkannt. Das ist weder eine Konformitätsaussage noch eine rechtliche Prüfung; ein Test mit einem echten Screenreader steht noch aus.
a11y-details-bfsg-incomplete = BFSG-relevante technische Kriterien: Der Accessibility Tree wirkt unvollständig (vermutlich Consent-Banner), das Ausbleiben von Verstößen sagt daher nichts aus. Die Prüfung auf der vollständigen Seite wiederholen.
a11y-details-bfsg-violations = BFSG-relevante technische Kriterien: im automatisierten Prüfumfang Verstöße festgestellt (siehe Befunde unten). Dies ist keine rechtliche Konformitätsprüfung.
a11y-details-bfsg-needs-review = BFSG-relevante technische Kriterien: im automatisierten Prüfumfang kein bestätigter Verstoß, aber Hinweise, die von Hand zu prüfen sind (siehe Befunde unten). Dies ist keine rechtliche Konformitätsprüfung.
a11y-details-bfsg-not-evaluated = BFSG-relevante technische Kriterien wurden für diese Seite nicht geprüft.
