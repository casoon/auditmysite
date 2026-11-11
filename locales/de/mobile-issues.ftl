# Texts of the mobile findings (`mobile::mobile_issue_text`, #936): one
# message per finding kind, the parts of the touch-target sentence and the
# touch-target context labels (`mobile::mobile_context_label`). Which findings
# exist, their severity and the number format stay in Rust.

mobile-missing-viewport = Viewport-Meta-Tag fehlt
mobile-improper-viewport = Viewport ist nicht korrekt konfiguriert
mobile-not-scalable = Seite unterbindet Zoomen (user-scalable=no)
mobile-small-targets = { $count } Bedienelemente sind zu klein (<44x44px){ $contexts }{ $samples }
mobile-small-targets-contexts = {" "}({ $list })
mobile-small-targets-samples = {" "}— z. B. { $list }
mobile-small-fonts = Kleinste Schriftgröße ist { $size }px (empfohlen: ≥12px)
mobile-horizontal-scroll = Seite scrollt horizontal

mobile-context-navigation = Navigation
mobile-context-footer = Fußbereich
mobile-context-header = Kopfbereich
mobile-context-sidebar = Seitenleiste
mobile-context-social-utility = Social/Utility
mobile-context-form = Formular
mobile-context-button = Schaltfläche
mobile-context-other = Sonstige
