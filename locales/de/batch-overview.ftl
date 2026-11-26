# German counterpart of `locales/en/batch-overview.ftl`: the texts of the batch
# PDF's overview page, page comparison, causes, next step and technical
# part (`output::pdf::batch_report::{glance, compare, causes, technical}`,
# `batch-overview-*`, `batch-compare-*`, `batch-causes-*`, `batch-next-*`,
# `batch-tech-*`, `batch-flag-*`), #1212 and #1213. The same keys as the
# English file.

## Overview page: three questions
batch-overview-title = Überblick
batch-overview-kicker = Abschneiden der Seiten, Wiederkehrendes und Empfehlung — aus den gemessenen Daten.
batch-overview-q-scores = Wie schneiden die Seiten ab?
batch-overview-a-scores = Je Seite { $min } bis { $max }; { $passed } von { $total } { $total ->
        [one] Seite
       *[other] Seiten
    } bestanden, kritische oder hohe Befunde auf { $severe } von { $total }.
batch-overview-a-scores-equal = { $total ->
        [one] Eine geprüfte Seite
       *[other] Alle { $total } Seiten liegen gleichauf
    }; { $passed } von { $total } { $total ->
        [one] Seite
       *[other] Seiten
    } bestanden, kritische oder hohe Befunde auf { $severe } von { $total }.
batch-overview-q-repeats = Was wiederholt sich?
batch-overview-a-repeats = { $types } { $types ->
        [one] Befundart
       *[other] Befundarten
    } auf { $pages } von { $total } Seiten: { $shared } auf mehreren Seiten (davon { $all } auf allen), { $specific } nur auf einer Seite.
batch-overview-a-repeats-top = Wichtigste: { $list }.
batch-overview-a-repeats-none = Keine Befunde auf den geprüften Seiten.
batch-overview-q-pick = Welche Seite lohnt einen Einzelbericht?
batch-overview-a-pick = { $page }: { $reason }
batch-overview-a-pick-none = Keine Seite sticht hervor: Auf den geprüften Seiten wurden keine Befunde gemessen.
batch-overview-pick-reason = Hier treten { $types } von { $batch_types } { $batch_types ->
        [one] Befundart
       *[other] Befundarten
    } der Stichprobe auf, { $severe } davon kritisch oder hoch ({ $shared_on } von { $shared } wiederkehrenden).
batch-overview-pick-tied = Die Barrierefreiheitswerte sind gleich; schwerster Befund eines anderen Moduls ({ $module }): „{ $title }“ ({ $severity }).
batch-overview-pick-lead = Zuerst die wiederkehrende Ursache „{ $title }“ prüfen.

## Page comparison
batch-compare-title = Seitenvergleich
batch-compare-intro = Die geprüften Seiten nach Barrierefreiheit, niedrigster Wert zuerst. „Befunde“ zählt je Seite gruppierte Befunde (eine Regel auf einer Seite ist ein Befund); die Zahlen der Titelseite zählen einzelne Vorkommen.
batch-compare-col-rank = Nr.
batch-compare-col-page = Seite
batch-compare-col-accessibility = Barrierefreiheit (0–100)
batch-compare-col-severe = Kritisch/hoch (Befunde)
batch-compare-col-focus = Einzelbericht, weil …
batch-compare-focus = { $types } { $types ->
        [one] Befundart
       *[other] Befundarten
    }, { $severe } kritisch oder hoch
batch-compare-focus-only = ; nur hier: { $list }
batch-compare-focus-none = keine Befunde
batch-compare-pick = Empfohlen: { $text }
batch-compare-more = { $count ->
        [one] Eine weitere Seite
       *[other] { $count } weitere Seiten
    } mit mittleren Werten stehen im JSON-Bericht und in der Tabelle des technischen Teils.

## Causes
batch-causes-title = Gemeinsame und seitenspezifische Ursachen
batch-causes-intro = Eine Karte je Befundart mit der größten Wirkung: wie weit sie reicht, woher sie vermutlich kommt und was zu tun ist. Die Befund-ID verbindet die Karte mit dem JSON-Bericht; die Belege je Seite stehen im Einzelbericht der Seite.
batch-causes-reach-page = { $reach } ({ $page })
batch-causes-more-json = { $count ->
        [one] Eine weitere Befundart steht
       *[other] { $count } weitere Befundarten stehen
    } im JSON-Bericht.

## Next step
batch-next-title = Nächster Schritt
batch-next-body = Einzelbericht für „{ $page }“ erstellen, um die Befunde dieser Seite im Detail zu sehen (Befehl: { $command }).

## Technical part
batch-tech-title = Technischer Teil und Methode
batch-tech-intro = Prüfrahmen, Berechnung der Empfehlung und die Zahlen hinter dem Überblick. Einzelheiten je URL stehen im JSON-Bericht und in den Einzelberichten.
batch-tech-method-title = Methode
batch-tech-pick-rule = Empfehlung für einen Einzelbericht: Jede Seite erhält ein Gewicht aus ihren verschiedenen Befundarten (kritisch 4, hoch 3, mittel 2, niedrig 1 je Befundart; die Zahl der Vorkommen zählt nicht). Empfohlen wird die Seite mit dem höchsten Gewicht; bei Gleichstand die mit dem niedrigeren Barrierefreiheitswert, danach die mit mehr kritischen oder hohen Befundarten, danach die URL in alphabetischer Reihenfolge. Haben alle Seiten denselben Barrierefreiheitswert, nennt die Empfehlung zusätzlich den schwersten Befund eines anderen Moduls auf der Seite. Gemessen wird nur an den geprüften Seiten.
batch-tech-seo-title = Seitenübergreifende SEO-Angaben
batch-tech-seo-intro = Angaben, die auf mehreren Seiten gleich sind oder auf Seiten fehlen — fehlen sie überall, liegt es meist am Template.
batch-tech-detail-title = Detail-Kapitel (Berichtsstufe technical)
batch-tech-detail-intro = Die folgenden Kapitel führen Einzelheiten je Seite auf, die der JSON-Bericht ebenfalls enthält.
batch-tech-consistency-detail-title = Seitenübergreifende Konsistenz: Einzelheiten
batch-tech-consistency-detail-intro = Seiten ohne beobachteten eingehenden Link, Schema-Konflikte, strukturierte Daten je Seite und Themenüberschneidungen.

## Audit caveats
batch-flag-text-consent-wall-artifact = Auf diesen Seiten kann ein Consent-Dialog statt des Seiteninhalts gemessen worden sein. Der Lauf mit --dismiss-consent wiederholt die Prüfung ohne Dialog.
batch-flag-text-other = Zur Messung dieser Seiten liegt ein weiterer Hinweis vor; Einzelheiten stehen im JSON-Bericht unter audit_flags.
