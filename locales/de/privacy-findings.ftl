# Texte der Datenschutz-Befunde, #936: die Befundmeldungen und die Namen der
# Rechtsdokumente aus `privacy::findings::finding_text`. Die Funktion bleibt eine
# reine `*_text(kind, provider, document, en)`-Textfunktion: die Analyse
# speichert kanonisches Englisch im JSON (`en = true`), das PDF ruft sie mit der
# Laufsprache auf (#406). Welche Meldung gilt, bleibt in Rust.

## Rechtsdokumente (die Link-Befunde)
privacy-finding-doc-privacy = Datenschutzerklärung
privacy-finding-doc-imprint = Impressum

## Befundmeldungen
privacy-finding-tracking-request-before-consent = { $provider } wurde beim ersten Besuch kontaktiert, bevor eine Einwilligungsentscheidung möglich war.
privacy-finding-non-essential-storage-before-consent = Cookies oder Speichereinträge von { $provider } waren nach dem ersten Besuch vorhanden, vor jeder Einwilligungsentscheidung.
privacy-finding-visitor-ip-transfer = { $provider } wurde beim ersten Besuch kontaktiert; die Anfrage übermittelt die IP-Adresse des Besuchers an einen Dritten.
privacy-finding-third-country-provider = Beim ersten Besuch kontaktierte Anbieter geben einen Sitz außerhalb der EU an. Das ist ein Hinweis auf den Anbieter, nicht auf den tatsächlichen Datenfluss.
privacy-finding-no-consent-platform-detected = Es wurden Tracking-Anfragen beobachtet, aber beim ersten Besuch ließ sich keine Consent-Plattform erkennen (Anfragen, Cookies, Speicher, Dokument).
privacy-finding-policy-link-missing = { $kind ->
        [imprint] Auf der Seite wurde kein Link zum Impressum gefunden.
       *[other] Auf der Seite wurde kein Link zur Datenschutzerklärung gefunden.
    }
privacy-finding-policy-link-unusable = { $kind ->
        [imprint] Der Link zum Impressum hat kein nutzbares Ziel (leer, '#' oder Skript).
       *[other] Der Link zur Datenschutzerklärung hat kein nutzbares Ziel (leer, '#' oder Skript).
    }
