# Texte von `auditmysite compare` (#1235): Tabelle und PDF des Vorher/Nachher-Vergleichs, `output::compare`.
compare-title = Vorher/Nachher-Vergleich
compare-run-old = Alter Lauf
compare-run-new = Neuer Lauf
compare-run-single = Einzelbericht
compare-run-batch = Sammelbericht
compare-run-pages =
    { $count ->
        [one] 1 Seite
       *[other] { $count } Seiten
    }
compare-run-tool = auditmysite { $version }
compare-pages-compared =
    { $count ->
        [one] 1 Seite verglichen
       *[other] { $count } Seiten verglichen
    }
compare-scores = Werte
compare-scores-average = Werte (Durchschnitt über die verglichenen Seiten)
compare-col-score = Wert
compare-col-old = Alt
compare-col-new = Neu
compare-col-change = Änderung
compare-col-page = Seite
compare-col-findings = Befunde
compare-score-overall = Gesamt (Website-Qualität)
compare-score-accessibility = Barrierefreiheit
compare-verdict = Urteil
compare-verdict-pass = bestanden
compare-verdict-warn = Warnung
compare-verdict-fail = nicht bestanden
compare-verdict-unchanged = { $verdict } (unverändert)
compare-verdict-changed = { $old } zu { $new }
compare-findings = Befunde
compare-findings-counts = { $resolved } behoben, { $new } neu, { $unchanged } unverändert
compare-resolved = Behoben
compare-new = Neu
compare-unchanged = Unverändert
compare-none = keine
compare-page-old-url = alter Lauf: { $url }
compare-pages = Seiten
compare-only-old = Nur im alten Bericht
compare-only-new = Nur im neuen Bericht
compare-reason-new-findings = neue Befunde
compare-reason-score-loss = Wertverlust
compare-result-worse = Ergebnis: Der neue Lauf ist schlechter ({ $reasons })
compare-result-ok = Ergebnis: keine neuen Befunde, kein Wertverlust
compare-note = Befunde werden über ihre stabile ID zugeordnet (eine Regel auf einer Seite), nicht über Elemente: Ein neues Element, das eine bekannte Regel verletzt, ist kein neuer Befund.
compare-more =
    { $count ->
        [one] und 1 weiterer
       *[other] und { $count } weitere
    }
