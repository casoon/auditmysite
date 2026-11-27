# Texts of the PDF finding cards (`output::pdf::findings`,
# `pdf-findings-*`) and of the shared module detail helpers
# (`output::pdf::detail_modules`, `pdf-details-*`), #936: the severity
# prefix, the code labels and row keys of a finding card, the occurrence
# evidence rows, the structural-cause labels, the customer-perspective
# titles, the module customer context, the derived-from and accessibility
# penalty notes, the score card caption, the Web Vitals rating and the
# journey category labels. Which message applies, the bracketless parts of
# the card header and the decimal notation of the contrast ratio stay in
# Rust. The code labels High, Medium and Low are `severity-high`,
# `severity-medium` and `severity-low` of report.ftl; the journey
# categories SPA navigation and Quantity stepper are
# `a11y-details-journey-label-spa-navigation` and
# `a11y-details-journey-label-quantity-stepper` of
# accessibility-details.ftl.

## Finding card: severity prefix
pdf-findings-severity-critical = [Kritisch]
pdf-findings-severity-high = [Hoch]
pdf-findings-severity-medium = [Mittel]
pdf-findings-severity-low = [Gering]

## Finding card: code labels
pdf-findings-code-very-high = Sehr hoch
pdf-findings-code-very-low = Sehr niedrig
pdf-findings-code-immediate = Sofort
pdf-findings-code-quick-win = Schnell umsetzbar
pdf-findings-code-normal = Normal
pdf-findings-code-manual-review-recommended = Manuelle Prüfung empfohlen
pdf-findings-code-automatically-confirmed = Automatisch bestätigt

## Finding card: metadata and assessment
pdf-findings-implementation-priority = Umsetzungspriorität
pdf-findings-qa-metadata = Prüfmetadaten
pdf-findings-detection-confidence = Erkennungssicherheit
pdf-findings-false-positive-risk = Falschpositiv-Risiko
pdf-findings-bfsg-eaa-expanded = BFSG-/EAA-Relevanz (Barrierefreiheitsstärkungsgesetz / European Accessibility Act)
pdf-findings-bfsg-eaa = BFSG-/EAA-Relevanz
pdf-findings-action-assessment = Maßnahmenbewertung
pdf-findings-expected-effect = Erwartete Wirkung
pdf-findings-complexity-reason = Komplexitätsgrund
pdf-findings-verification = Prüfung
pdf-findings-self-test = Selbsttest

## Finding card: occurrences
pdf-findings-dom-path = DOM-Pfad
pdf-findings-measured-value = Messwert
pdf-findings-contrast = Kontrast { $ratio }
pdf-findings-contrast-required = Kontrast { $ratio } (erforderlich { $required })
pdf-findings-opacity = Deckkraft
pdf-findings-opacity-value = { $opacity }: Das Verhältnis gilt für diesen Zustand (Textfarbe nach Deckkraft { $color })
pdf-findings-opacity-pattern = In den Stichproben bei verringerter Deckkraft gemessen ({ $values }): Die Textfarbe selbst kann genügen; das Verhältnis gilt für den abgeblendeten Zustand, etwa ein inaktives oder einblendendes Element.
pdf-findings-color-scheme = Farbschema
pdf-findings-color-scheme-dark = Dunkel (prefers-color-scheme: dark)
pdf-findings-screenshot-mobile = Element auf der Seite (Mobile-Ansicht)
pdf-findings-screenshot-desktop = Element auf der Seite (Desktop-Ansicht)
pdf-findings-screenshot-alt-mobile = Screenshot des beanstandeten Elements { $selector } in der Mobile-Ansicht
pdf-findings-screenshot-alt-desktop = Screenshot des beanstandeten Elements { $selector } in der Desktop-Ansicht
pdf-findings-examples-of = { $shown } { $shown ->
        [one] Beispiel
       *[other] Beispiele
    } von { $total } Vorkommen
pdf-findings-finding-id = Befund-ID
pdf-findings-rule = Regel

## Finding card: origin of the markup (#1109). A label, never a verdict.
pdf-findings-origin = Herkunft
pdf-findings-origin-cross-origin = Drittanbieter-Frame (Anbieter { $list }), Behebung beim Anbieter
pdf-findings-origin-cross-origin-unknown = Drittanbieter-Frame, Behebung beim Anbieter des eingebetteten Inhalts
pdf-findings-origin-cross-origin-source = Drittanbieter-Frame (Quelle { $list }), Behebung beim Anbieter des eingebetteten Inhalts
pdf-findings-origin-same-site = Frame derselben Website, Behebung in der eingebetteten Seite
pdf-findings-origin-mixed = Teils in eingebetteten Frames, teils in der Seite selbst
pdf-findings-origin-mixed-provider = Teils in eingebetteten Frames (Anbieter { $list }), teils in der Seite selbst

## Finding card: customer perspective
pdf-findings-perspective-operability = Bedienbarkeit
pdf-findings-perspective-structure = Orientierung und Struktur
pdf-findings-perspective-perception = Wahrnehmbarkeit und Lesbarkeit
pdf-findings-perspective-discoverability = Auffindbarkeit und KI-Verständnis
pdf-findings-perspective-trust = Vertrauen und technische Qualität
pdf-findings-perspective-loading = Lade- und Mobile-Erlebnis
pdf-findings-perspective-customer-impact = Kundenauswirkung

## Module details: customer context
pdf-details-weakness-clear = Der Score zeigt in diesem Bereich eine deutliche Schwäche.
pdf-details-weakness-visible = Der Score zeigt in diesem Bereich erkennbares Verbesserungspotenzial.
pdf-details-context-performance = Besucher können Verzögerungen, instabiles Rendering oder unnötige Datenmenge erleben, bevor die Seite nutzbar wirkt.
pdf-details-context-seo = Suchmaschinen und KI-Systeme benötigen klare Titel, Überschriften, strukturierte Daten und ausreichend lesbaren Inhalt, um die Seite zu verstehen.
pdf-details-context-security = Security Header und HTTPS-Signale beeinflussen sichtbares Vertrauen und reduzieren vermeidbare Browser-Risiken im geprüften Umfang. Welche Header eine Seite braucht, hängt von der Anwendung ab; eine alternative Schutzmaßnahme zählt, wo die Prüfung sie akzeptiert (eine CSP-Direktive frame-ancestors ersetzt X-Frame-Options), und Referrer-Policy, Permissions-Policy sowie die Cross-Origin-Policies werden als architektur- bzw. einsatzabhängig eingeordnet.
pdf-details-context-html-conform = Sauberes, spezifikationskonformes HTML hilft Browsern, assistiven Technologien und Suchmaschinen, die Seite zuverlässig zu verarbeiten und darzustellen.
pdf-details-context-mobile = Mobile Besucher sind auf lesbaren Text, passende Inhaltsbreiten und gut antippbare Bedienelemente angewiesen.
pdf-details-context-ux = Dieser Indikator schätzt, ob die Seite für typische Besucheraufgaben verständlich, konsistent und reibungsarm wirkt.
pdf-details-context-journey = Dieser Indikator schätzt, ob Besucher ohne vermeidbare Reibung durch den Zweck der Seite kommen.
pdf-details-context-source-quality = Diese Signale zeigen, wie gut Maschinen die Seite verarbeiten können: Herausgeber- und Sicherheitsangaben und ob Titel, Sprache und strukturierte Daten zueinander passen.
pdf-details-context-ai-visibility = KI-Sichtbarkeit zeigt, ob Inhalte von KI-gestützten Systemen gelesen, gegliedert und zugeordnet werden können.
pdf-details-context-content-visibility = Content Visibility verbindet Auffindbarkeit, Vertrauen und inhaltliche Tiefe; es ist ein Indikator, keine Reichweiten-Garantie.
pdf-details-context-other = Dieses Modul beschreibt kundennahe Qualität über reine Accessibility-Befunde hinaus.

## Module details: notes and captions
pdf-details-score-caption = Gesamtwertung · 0–100

## Module details: Web Vitals rating
pdf-details-vital-good = Gut
pdf-details-vital-needs-improvement = Verbesserungswürdig
pdf-details-vital-poor = Schlecht

## Module details: journey categories
pdf-details-journey-tab-order = Tab-Reihenfolge
pdf-details-journey-focus-trap = Fokus-Falle
pdf-details-journey-state-transition = Zustandswechsel
pdf-details-journey-focus-restoration = Fokus-Wiederherstellung
pdf-details-journey-form-error = Formularfehler-Ansage
pdf-details-journey-hidden-focusable = Verstecktes fokussierbares Element
pdf-details-journey-skip-link = Skip-Link
pdf-details-journey-focus-indicator = Fokus-Indikator
pdf-details-journey-menu = Menü-Navigation
pdf-details-journey-tabs = Tab-Navigation
pdf-details-journey-link-text = Linktext
pdf-details-journey-landmark = Landmarke
pdf-details-journey-media-controls = Medienbedienung
pdf-details-journey-add-to-cart = In den Warenkorb legen
pdf-details-journey-heading-outline = Überschriftengliederung
