# Texts of the security findings (`security::security_issue_text`, #936): one
# message per finding kind. Counts reach the messages as numbers, so the plural
# agreement lives here; which findings exist, their severity and the default
# values stay in Rust. Also the labels of the security categories.

security-missing-https = Seite wird nicht über HTTPS ausgeliefert
security-missing-csp = Content-Security-Policy-Header fehlt
security-missing-x-content-type-options = X-Content-Type-Options-Header fehlt
security-missing-x-frame-options = X-Frame-Options-Header fehlt (Clickjacking-Schutz)
security-missing-hsts = HSTS-Header fehlt
security-missing-referrer-policy = Referrer-Policy-Header fehlt
security-missing-permissions-policy = Permissions-Policy-Header fehlt
security-missing-coop = Cross-Origin-Opener-Policy ist nicht gesetzt. Dieser Header ist nur relevant, wenn die Seite SharedArrayBuffer, hochauflösende Timer verwendet oder sich von Cross-Origin-Popups isolieren muss — trifft das nicht zu, ist hier keine Maßnahme nötig. Trifft es zu: auf same-origin setzen und prüfen, ob Popup-/Fenster-Interaktionen weiterhin wie erwartet funktionieren.
security-missing-corp = Cross-Origin-Resource-Policy ist nicht gesetzt. Dieser Header ist nur relevant, wenn die Seite Schriften, Skripte oder Medien ausliefert, die andere Origins nicht laden können sollen — sind die Ressourcen bewusst öffentlich, ist hier keine Maßnahme nötig. Soll Cross-Origin-Laden eingeschränkt werden: auf same-origin oder same-site setzen.
security-cors-wildcard-credentials = CORS erlaubt jede Origin und gleichzeitig Credentials
security-hsts-preload-ineligible = HSTS-preload-Direktive ist gesetzt, die Voraussetzungen sind aber nicht erfüllt (max-age={ $max_age }, includeSubDomains={ $subdomains }) — die Seite wird nicht in die HSTS-Preload-Liste aufgenommen
security-permissions-policy-permissive = Permissions-Policy-Header ist vorhanden, schränkt aber kein Feature ein (leer oder nur Wildcards)
security-public-source-map =
    { $count } öffentlich erreichbare { $count ->
        [one] Source-Map
       *[other] Source-Maps
    } gefunden{ $example } — legt Originalquellcode, Kommentare und interne Dateipfade offen
security-public-source-map-example = {" "}(z. B. { $url })
security-csp-unsafe-inline-script = CSP erlaubt unsafe-inline-Skripte ohne Nonce-/Hash-Absicherung
security-csp-unsafe-eval-script = CSP erlaubt unsafe-eval in den Skript-Quellen
security-csp-wildcard-script-source = CSP erlaubt Wildcard-Skriptquellen
security-csp-unsafe-inline-style = CSP erlaubt unsafe-inline-Styles ohne Nonce-/Hash-Absicherung
security-csp-wildcard-source = CSP enthält Wildcard-Quellausdrücke
security-csp-missing-directive = CSP fehlt die { $directive }-Direktive

# Labels of the security categories (`security::SecurityCategory::label`).
security-category-response-headers = Response-Header
security-category-transport-security = Transportsicherheit
security-category-access-policies = Zugriffs-Richtlinien
security-category-third-party-exposure = Drittanbieter-Exposition
