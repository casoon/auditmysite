# Texte der experimentellen UX-Heuristiken (`ux::ux_heuristic_text`, #1035): eine
# Message je Heuristik und die beiden Zeilen der Terminal-Tabelle, die sie
# tragen (`output::terminal`). Welche Heuristiken es gibt, ihre Schwere und die
# Zählwerte bleiben in Rust. Hinweise, keine Befunde einer Regel.

ux-heuristic-cookie-banner = Auf der Seite liegt ein Cookie- oder Einwilligungsbanner. Prüfen, ob es den Hauptinhalt nicht verdeckt und der Tastaturfokus es erreichen und wieder verlassen kann.
ux-heuristic-dialog-not-modal = Ein Dialog ist nicht als modal ausgezeichnet (aria-modal). Verdeckt er die Seite, sollte er als modal ausgezeichnet sein und den Tastaturfokus bei sich halten.
ux-heuristic-multiple-dialogs = { $count } Dialoge liegen gleichzeitig in der Seite. Prüfen, ob sie sich nicht gegenseitig verdecken.
ux-heuristic-no-cta = Keine Schaltfläche und kein Link mit Handlungsaufforderung gefunden. Eine Seite, die zu einer Handlung führen soll, braucht womöglich eine.
ux-heuristic-many-ctas = { $count } Schaltflächen oder Links mit Handlungsaufforderung konkurrieren um Aufmerksamkeit. Prüfen, ob die Hauptaktion hervortritt.

## Zeilen der Terminal-Tabelle (`output::terminal`)
ux-heuristic-terminal-none = UX-Hinweise: keine
ux-heuristic-terminal-line = UX-Hinweis: { $text }
