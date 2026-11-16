# Texte der Interpretationsschicht, #936 (Slice 27): die SEO-, Performance- und
# Score-Interpretationssätze (`seo::interpretation`,
# `audit::performance_interpretation`, `audit::interpretation`), die
# Verdict-Gründe (`audit::verdict`), die Aufwands- und Wirkungssätze eines
# Befunds (`audit::finding_derive`), die Zusammenfassung der Audit-Lage
# (`audit::summary`), die Meldung zu langen Überschriften (`seo::headings`),
# die Baseline-Zeile (`audit::baseline`) und die Bereichsnamen des Scores
# (`taxonomy::score_area`). Die Funktionen bleiben reine
# `*_text(kind, values, en)`-Textfunktionen: die Analyse legt kanonisches
# Englisch ins JSON ab (`en = true`), das PDF ruft sie mit der Berichtssprache
# auf (#406). Welche Meldung gilt, die Schwellen und die Zahlenschreibweise
# bleiben in Rust. Messwerte kommen bereits formatiert als Argumente herein
# (`$share`, `$ms`, `$kb`, `$dom`); `$cross` ist der Hinweis auf die
# Wechselwirkung mit seinem führenden Leerzeichen, oder leer. Das "und" der
# Aufzählungen ist `audit-join-and` aus audit-texts.ftl.

## SEO interpretation (`seo::interpretation`)
audit-interpretation-seo-lead-excellent = Die technischen SEO-Grundlagen sind weitgehend erfüllt und unterstützen die Sichtbarkeit in Suchmaschinen.
audit-interpretation-seo-lead-good = Gute SEO-Grundlage mit gezieltem Optimierungspotenzial für mehr Sichtbarkeit.
audit-interpretation-seo-lead-partial = Wichtige SEO-Grundlagen fehlen teilweise, wodurch die Sichtbarkeit in Suchmaschinen eingeschränkt sein kann.
audit-interpretation-seo-lead-insufficient = SEO unzureichend — wesentliche Grundlagen fehlen und begrenzen die Auffindbarkeit deutlich.
audit-interpretation-seo-lead-critical = SEO kritisch — grundlegende Voraussetzungen fehlen; Indexierung und Auffindbarkeit sind gefährdet.
audit-interpretation-seo-page-type = Seitentyp: „{ $page_type }“.
audit-interpretation-seo-depth-note = Technisches SEO vollständig — inhaltliche Tiefe ({ $depth }/100) noch ausbaufähig.
audit-interpretation-seo-quality-coherent = sehr stimmig aufgebaut
audit-interpretation-seo-quality-supported = technisch und strukturell gut gestützt
audit-interpretation-seo-quality-partial = nur teilweise klar strukturiert
audit-interpretation-seo-quality-weak = aktuell inhaltlich und strukturell schwach ausgeprägt
audit-interpretation-seo-no-attributes = ohne klare Zusatzmerkmale
audit-interpretation-seo-profile-summary = Die Seite wirkt wie „{ $page_type }“ und ist { $quality }. Auffällig sind { $traits }.
audit-interpretation-seo-note-depth = Mehr inhaltliche Tiefe und klar gegliederte Abschnitte würden den Nutzwert erhöhen.
audit-interpretation-seo-note-structure = Mehr Zwischenüberschriften und eine klarere Inhaltsstruktur würden die Seite besser scannbar machen.
audit-interpretation-seo-note-media = Die Seite wirkt stark visuell. Mehr erklärender Text und klarer Kontext würden Nutzen und Orientierung verbessern.
audit-interpretation-seo-note-intent = Die Seite bedient ihren Seitentyp noch nicht sauber; Aufbau und Inhalte sollten stärker auf das eigentliche Nutzerziel einzahlen.
audit-interpretation-seo-note-fine = Die Seite passt insgesamt gut zu ihrem Seitentyp. Der größte Hebel liegt in weiterer inhaltlicher Schärfung statt in Grundsatzumbauten.

## Performance interpretation (`audit::performance_interpretation`)
audit-interpretation-perf-dom = die hohe DOM-Komplexität ({ $nodes } Knoten)
audit-interpretation-perf-lcp = der spät erscheinende Hauptinhalt unter Drosselung ({ $ms } ms LCP)
audit-interpretation-perf-render-blocking = { $count } render-blockierende Ressourcen
audit-interpretation-perf-unminified = unminifizierte Assets (~{ $kb } KB Einsparpotenzial)
audit-interpretation-perf-animations = { $count } nicht-composited Animationen
audit-interpretation-perf-positive-layout = Layout-Stabilität (CLS)
audit-interpretation-perf-positive-main-thread = Hauptthread-Blockierung (TBT)
audit-interpretation-perf-critical-here = Kritisch sind hier { $phrases }.
audit-interpretation-perf-contrast =
    Im Kontrast dazu { $count ->
        [one] bleibt
       *[other] bleiben
    } { $phrases } unauffällig.
audit-interpretation-perf-reason-dom = DOM-Größe
audit-interpretation-perf-reason-render-blocking = Render-blockierende Ressourcen
audit-interpretation-perf-gap = { $base } Score durch { $reasons } reduziert, obwohl Core Web Vitals im grünen Bereich liegen.

## Score interpretation per module and band (`audit::interpretation`)
audit-interpretation-score-accessibility-excellent = Sehr gut — die Barrierefreiheit ist technisch sauber umgesetzt und weist nur geringe Einschränkungen auf.
audit-interpretation-score-accessibility-good = Gut — die Barrierefreiheit ist insgesamt stabil und konsistent, kleinere Optimierungen sind sinnvoll.
audit-interpretation-score-accessibility-needs-improvement = Verbesserungswürdig — einzelne Barrieren können die Nutzung einschränken.
audit-interpretation-score-accessibility-weak = Ausbaufähig — relevante Barrieren beeinträchtigen Nutzbarkeit und Zugänglichkeit.
audit-interpretation-score-accessibility-critical = Kritisch — wesentliche Anforderungen an die Barrierefreiheit werden nicht erfüllt.
audit-interpretation-score-performance-excellent = Sehr gut — die Seite reagiert schnell und bietet eine flüssige Nutzererfahrung.
audit-interpretation-score-performance-good = Gut — die Performance ist stabil, vereinzelt bestehen Optimierungsmöglichkeiten.
audit-interpretation-score-performance-needs-improvement = Verbesserungswürdig — Ladezeiten und Reaktionsverhalten sind stellenweise uneinheitlich.
audit-interpretation-score-performance-weak = Ausbaufähig — Performance-Probleme können Nutzung und Conversion beeinträchtigen.
audit-interpretation-score-performance-critical = Kritisch — deutliche Performance-Probleme beeinträchtigen die Nutzererfahrung erheblich.
audit-interpretation-score-security-excellent = Sehr gut — keine wesentlichen Sicherheitsauffälligkeiten im geprüften Umfang erkannt.
audit-interpretation-score-security-good = Gut — grundlegende Sicherheitsmechanismen sind vorhanden, kleinere Optimierungspotenziale wurden erkannt.
audit-interpretation-score-security-needs-improvement = Verbesserungswürdig — einzelne Sicherheitsaspekte sollten überprüft und abgesichert werden.
audit-interpretation-score-security-weak = Ausbaufähig — relevante Sicherheitsauffälligkeiten oder Fehlkonfigurationen wurden erkannt.
audit-interpretation-score-security-critical = Kritisch — im geprüften Umfang fehlen grundlegende Schutzmaßnahmen. Das ist kein Nachweis einer ausnutzbaren Lücke, sollte aber vorrangig behoben werden.
audit-interpretation-score-html-conform-excellent = Sehr gut — das ausgelieferte HTML ist technisch sauber und spezifikationskonform, Browser und assistive Technologien können es zuverlässig verarbeiten.
audit-interpretation-score-html-conform-good = Gut — das HTML ist überwiegend konform, einzelne Abweichungen von der Spezifikation sind vorhanden.
audit-interpretation-score-html-conform-needs-improvement = Verbesserungswürdig — mehrere Spezifikationsverstöße im HTML können die Verarbeitung durch Browser oder assistive Technologien beeinträchtigen.
audit-interpretation-score-html-conform-weak = Ausbaufähig — verbreitete Spezifikationsverstöße erschweren eine zuverlässige Verarbeitung des Markups.
audit-interpretation-score-html-conform-critical = Kritisch — das HTML weicht erheblich von der Spezifikation ab, Rendering und Barrierefreiheit sind unmittelbar betroffen.
audit-interpretation-score-mobile-excellent = Sehr gut — die Nutzung auf Mobilgeräten funktioniert zuverlässig und ohne erkennbare Einschränkungen.
audit-interpretation-score-mobile-good = Gut — die Nutzung auf Mobilgeräten funktioniert insgesamt zuverlässig.
audit-interpretation-score-mobile-needs-improvement = Verbesserungswürdig — auf Mobilgeräten treten stellenweise Bedien- und Darstellungsprobleme auf.
audit-interpretation-score-mobile-weak = Ausbaufähig — Darstellung und Bedienung auf Mobilgeräten sind spürbar eingeschränkt.
audit-interpretation-score-mobile-critical = Kritisch — die Seite ist auf Mobilgeräten kaum zuverlässig nutzbar.
audit-interpretation-score-ux-excellent = Sehr gut — die strukturellen Signale sprechen für eine klare, gut geführte Bedienung.
audit-interpretation-score-ux-good = Gut — die strukturellen Signale sprechen für eine verständliche Nutzerführung; einzelne Stellen lassen sich straffen.
audit-interpretation-score-ux-needs-improvement = Verbesserungswürdig — die strukturellen Signale deuten stellenweise auf unnötig komplexe Nutzerführung hin.
audit-interpretation-score-ux-weak = Ausbaufähig — erkannte Reibungspunkte können eine klare Nutzerführung erschweren.
audit-interpretation-score-ux-critical = Kritisch — die strukturellen Signale deuten auf eine unübersichtliche Bedienung hin, die das Erreichen von Zielen erschweren kann.
audit-interpretation-score-journey-excellent = Sehr gut — die Seitenstruktur stützt die wichtigsten Nutzerpfade durchgängig und nachvollziehbar.
audit-interpretation-score-journey-good = Gut — die Seitenstruktur stützt die zentralen Nutzerpfade; einzelne Schritte lassen sich verbessern.
audit-interpretation-score-journey-needs-improvement = Verbesserungswürdig — die Seitenstruktur deutet auf umständliche oder unklare Schritte in der Nutzerführung hin.
audit-interpretation-score-journey-weak = Ausbaufähig — erkannte Brüche in der Nutzerführung können das Erreichen zentraler Ziele erschweren.
audit-interpretation-score-journey-critical = Kritisch — die Seitenstruktur deutet auf unnötig komplizierte oder unterbrochene Schritte der Nutzerführung hin.

## Accessibility claim bullet (`audit::interpretation`)
audit-interpretation-claim-not-measured = Accessibility-Systematik: Nicht gemessen — die automatisierte Accessibility-Prüfung lief nicht
audit-interpretation-claim-incomplete = Accessibility-Systematik: Keine Verstöße gefunden, aber der Lauf war unvollständig — kein Beleg in die eine oder andere Richtung
audit-interpretation-claim-clean = Accessibility-Systematik: Keine Verstöße in der automatisierten Prüfung ({ $automated } von { $all } WCAG-{ $version }-A/AA-Kriterien automatisch prüfbar) — keine Konformitätsaussage, manuelle Prüfung nötig
audit-interpretation-claim-systematic = Accessibility-Systematik: Systematische Muster ({ $rules } Regelgruppen, { $total } Vorkommen) — Prozess-Problem, kein Einzelfall
audit-interpretation-claim-blockers = Accessibility-Systematik: Mehrere kritische Blockaden ({ $critical } kritisch, { $high } hoch) — direkte Screenreader-Barrieren
audit-interpretation-claim-distributed = Accessibility-Systematik: Verteilt über { $rules } Regeltypen — kein Muster, einzeln behebbar
audit-interpretation-claim-focused = Accessibility-Systematik: { $total } Verstöße in { $rules } Bereichen — konzentriert und gezielt behebbar

## Technical overview bullets (`audit::interpretation`)
audit-interpretation-overview-seo-excellent = SEO-Level: { $score } Pkt — technische Ranking-Voraussetzungen erfüllt
audit-interpretation-overview-seo-good = SEO-Level: { $score } Pkt — Basis vorhanden, gezielte Optimierungen möglich
audit-interpretation-overview-seo-partial = SEO-Level: { $score } Pkt — relevante Signale fehlen, Sichtbarkeit eingeschränkt
audit-interpretation-overview-seo-weak = SEO-Level: { $score } Pkt — strukturelle Basis fehlt, Ranking-Potenzial deutlich eingeschränkt
audit-interpretation-overview-seo-none = SEO-Level: Nicht geprüft (--full für vollständige Analyse)
audit-interpretation-overview-security-full = Security-Level: { $score } Pkt — HTTP-Security-Header vollständig gesetzt
audit-interpretation-overview-security-core = Security-Level: { $score } Pkt — { $present } von { $total } Kern-Headern gesetzt, offene Sicherheitsbefunde bleiben
audit-interpretation-overview-security-basic = Security-Level: { $score } Pkt — Grundschutz vorhanden, einzelne Header fehlen
audit-interpretation-overview-security-several = Security-Level: { $score } Pkt — mehrere Basisschutz-Header fehlen
audit-interpretation-overview-security-almost = Security-Level: { $score } Pkt — Security-Header fehlen fast vollständig — vorrangig, mit geringem Aufwand behebbar
audit-interpretation-overview-security-none = Security-Level: Nicht geprüft (--full für vollständige Analyse)
audit-interpretation-overview-tech-high = Tech-Komplexität: Hoch — { $dom } DOM-Knoten, Performance { $perf } Pkt — Refactoring empfohlen
audit-interpretation-overview-tech-strain = Tech-Komplexität: Mittel-hoch — { $dom } DOM-Knoten belasten eine nur mittelmäßige Performance ({ $perf } Pkt) — Optimierungspotenzial vorhanden
audit-interpretation-overview-tech-holding = Tech-Komplexität: Mittel-hoch — { $dom } DOM-Knoten bei derzeit tragfähiger Performance ({ $perf } Pkt)
audit-interpretation-overview-tech-perf-critical = Tech-Komplexität: Performance kritisch ({ $perf } Pkt) — { $dom } DOM-Knoten analysiert
audit-interpretation-overview-tech-low-optimizable = Tech-Komplexität: Gering — { $dom } DOM-Knoten, Performance optimierbar ({ $perf } Pkt)
audit-interpretation-overview-tech-low-stable = Tech-Komplexität: Gering — { $dom } DOM-Knoten, Performance { $perf } Pkt — technische Basis stabil
audit-interpretation-overview-tech-unmeasured-high = Tech-Komplexität: Hoch — { $nodes } geprüfte Knoten (Performance nicht geprüft)
audit-interpretation-overview-tech-unmeasured = Tech-Komplexität: { $nodes } geprüfte Knoten (Performance nicht geprüft, --full)

## Overall impact table and benchmark (`audit::interpretation`)
audit-interpretation-impact-label-user = Nutzererlebnis
audit-interpretation-impact-label-risk = Risiko-Level
audit-interpretation-impact-label-conversion = Conversion-Effekt
audit-interpretation-impact-user-excellent = Sehr gut — keine relevanten Barrieren
audit-interpretation-impact-user-good = Gut — einzelne Barrieren für Hilfstechnologien
audit-interpretation-impact-user-limited = Eingeschränkt — spürbare Barrieren für Screenreader-Nutzer
audit-interpretation-impact-user-heavy = Stark eingeschränkt — wesentliche Inhalte nicht zugänglich
audit-interpretation-impact-risk-high = Hoch — BITV/WCAG-Verstoßrisiko akut
audit-interpretation-impact-risk-critical-topics = Mittel — kritische Themen vorhanden
audit-interpretation-impact-risk-backlog = Mittel — kumulierter Nachholbedarf
audit-interpretation-impact-risk-low = Niedrig
audit-interpretation-impact-conversion-negative = Hoch wahrscheinlich negativ
audit-interpretation-impact-conversion-possible = Möglicherweise negativ (Navigation, Formulare)
audit-interpretation-impact-conversion-low = Gering — gute Nutzbarkeit
audit-interpretation-benchmark-top5 = Top 5% — Ausnahmeniveau. Kein struktureller Handlungsdruck.
audit-interpretation-benchmark-top15 = Top 15% — Deutlich besser als die Mehrheit. Feinschliff genügt.
audit-interpretation-benchmark-upper-third = Oberes Drittel — Guter Stand, einzelne Optimierungen lohnen sich.
audit-interpretation-benchmark-middle = Mittleres Feld — Verbesserungspotenzial vorhanden, kein akuter Notfall.
audit-interpretation-benchmark-lower-middle = Unteres Mittelfeld — Deutlicher Rückstand gegenüber vergleichbaren Websites.
audit-interpretation-benchmark-lower-third = Unteres Drittel — Erheblicher Rückstand, strukturelle Defizite häufig.
audit-interpretation-benchmark-critical = Kritisch — Zu den schwächsten geprüften Seiten. Sofortiger Handlungsbedarf.

## Verdict reasons (`audit::verdict`)
audit-interpretation-verdict-legal-flags =
    { $count ->
        [one] 1 Befund mit möglicher Rechtsrelevanz (BFSG/EAA)
       *[other] { $count } Befunde mit möglicher Rechtsrelevanz (BFSG/EAA)
    }
audit-interpretation-verdict-fail-threshold = Score { $score } unter der Fail-Schwelle von { $threshold }
audit-interpretation-verdict-warn-threshold = Score { $score } unter der Warnschwelle von { $threshold }
audit-interpretation-verdict-incomplete = der Lauf ist nicht vollständig durchgelaufen
audit-interpretation-verdict-technical-blockers =
    { $count ->
        [one] 1 technischer Blocker (Seite soll indexiert werden, trägt aber noindex)
       *[other] { $count } technische Blocker
    }
audit-interpretation-verdict-findings-present =
    { $count ->
        [one] 1 offener Befund
       *[other] { $count } offene Befunde
    }

## Complexity and expected impact of a finding (`audit::finding_derive`)
audit-interpretation-complexity-high = { $count } Vorkommen deuten auf ein Komponenten- oder Template-Problem hin.
audit-interpretation-complexity-technical = Die Behebung ist technisch, betrifft aber nur wenige Muster.
audit-interpretation-complexity-moderate = { $count } Vorkommen erfordern einheitliche Anpassungen in Inhalten oder Templates.
audit-interpretation-complexity-low = Wenige Vorkommen und ein klar abgegrenzter Fix.
audit-interpretation-expected-impact-wcag = Betrifft { $count } Vorkommen; WCAG-Level: { $level }. Wirkung auf den Score nicht beziffert.
audit-interpretation-expected-impact-other = Betrifft { $count } Vorkommen; wirkt auf Sichtbarkeit/Struktur. Wirkung auf den Score nicht beziffert.

## Audit summary narrative (`audit::summary`); `$cross` is the cross-impact note with its leading space or empty
audit-interpretation-summary-groups =
    { $count ->
        [one] 1 Problemgruppe
       *[other] { $count } Problemgruppen
    }
audit-interpretation-summary-occurrences-note = , { $count } Vorkommen
audit-interpretation-summary-dominant-note = "{ $title }" macht { $share }{ "\u202F" }% der kritischen/hohen Findings aus ({ $groups }{ $occurrences }).
audit-interpretation-summary-detail-occurrences = { $count } Vorkommen
audit-interpretation-summary-detail-hits = { $count } Treffer
audit-interpretation-summary-intro-systematic = Kein Einzelproblem — { $total } Verstöße über { $critical } kritische und { $high } hohe Themen sind ein systematisches Muster. Betrifft große Teile der Seite, nicht einzelne Stellen.{ $cross }
audit-interpretation-summary-intro-dominant = Ein Problem dominiert: "{ $title }" verursacht { $share } % der kritischen/hohen Findings ({ $detail }). Hier konzentrieren -- eine Ursache, hoher Impact.{ $cross }
audit-interpretation-summary-intro-polished-clear = Automatisiert unauffällig — keine dringenden Barrieren erkannt.{ $cross } Niveau halten und regelmäßig nachprüfen.
audit-interpretation-summary-intro-polished-topics =
    Tragfähige Basis. { $urgent ->
        [one] 1 priorisiertes Thema
       *[other] { $urgent } priorisierte Theman
    } — gezielt beheben, bevor sie sich häufen.{ $cross }
audit-interpretation-summary-intro-clustered = Tragfähige Basis, aber { $urgent } priorisierte Themen verteilen sich auf mehrere unabhängige Bereiche. Strukturiert priorisieren — nicht alles auf einmal.{ $cross }
audit-interpretation-summary-intro-needs-work-clear = Tragfähige Basis ohne akute Risiken. { $total } Verbesserungen möglich — gut priorisierbar.{ $cross }
audit-interpretation-summary-intro-needs-work-urgent =
    Gute Basis, aber { $urgent ->
        [one] 1 priorisiertes Thema braucht
       *[other] { $urgent } priorisierte Theman brauchtn
    } sofortige Aufmerksamkeit.{ $cross }
audit-interpretation-summary-intro-weak = Relevante Barrieren vorhanden -- { $urgent } davon kritisch oder hoch. Strukturiert priorisieren und Phase 1 starten.{ $cross }
audit-interpretation-summary-intro-critical = Akuter Handlungsbedarf: { $critical } kritische, { $high } hohe Issues. Die Seite ist für einen Teil der Nutzer schwer nutzbar -- sofort Phase 1 starten.{ $cross }
audit-interpretation-summary-problem-minimal = Keine Verstöße im automatisierten Prüfumfang gefunden
audit-interpretation-summary-problem-dominant = Dominierendes Einzelproblem: "{ $title }" — konzentriert und gezielt behebbar
audit-interpretation-summary-problem-single = Einzelproblem — konzentriert und gezielt behebbar
audit-interpretation-summary-problem-structural = Strukturelle Defizite — flächendeckende Barrieren in mehreren Bereichen
audit-interpretation-summary-problem-clusters = Mehrere Problemcluster — über verschiedene Bereiche verteilt, gezielt behebbar
audit-interpretation-summary-problem-polish = Feinschliff — keine strukturellen Defizite, letzte Optimierungshebel
audit-interpretation-summary-problem-isolated = Mehrere kritische Einzelprobleme — konzentriert und gezielt behebbar
audit-interpretation-summary-cross-headings = Fehlende Überschriftenstruktur wirkt gleichzeitig als SEO-Schwäche und Accessibility-Barriere.
audit-interpretation-summary-cross-security-mobile = Security-Schwächen und Mobile-Probleme treten gemeinsam auf — Optimierungsbedarf zieht sich durch mehrere Bereiche.
audit-interpretation-summary-cross-performance-mobile = Schlechte Ladezeiten verstärken Mobile-Probleme — Mobile-Nutzbarkeit ist doppelt eingeschränkt.

## Small texts: long heading, baseline line, score area labels
audit-interpretation-long-heading = H{ $level } ist zu lang ({ $length } Zeichen): "{ $excerpt }..."
audit-interpretation-baseline-summary = Baseline: { $known } bekannte, { $new } neue, { $fixed } behobene Befunde; bekannte Befunde beeinflussen das Urteil nicht.
audit-interpretation-baseline-unmatched = { $count } Baseline-Seiten waren nicht Teil dieses Laufs.
audit-interpretation-score-area-semantics = Semantik
audit-interpretation-score-area-forms = Formulare
audit-interpretation-score-area-keyboard = Tastatur
audit-interpretation-score-area-focus = Fokusführung
audit-interpretation-score-area-images = Bilder / Alternativtexte
audit-interpretation-score-area-aria = ARIA
audit-interpretation-score-area-headings = Überschriftenstruktur
audit-interpretation-score-area-landmarks = Landmarken / Seitenstruktur
