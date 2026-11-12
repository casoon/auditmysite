# Fixes of the findings outside the WCAG taxonomy (`audit::finding_titles::
# finding_fix`, #1039): one message per rule id `<source>.<check>`, keyed like
# `finding-titles.ftl` with the prefix `finding-fix-`. The fix is the
# recommendation of the finding card and the action of the action plan; a
# finding without one has no action to plan.

## SEO: page health
finding-fix-seo-page-health-missing-doctype = Jedes HTML-Dokument mit <!DOCTYPE html> beginnen, damit Browser es im Standardmodus darstellen.
finding-fix-seo-page-health-document-write = document.write() durch DOM-Methoden (createElement, append) ersetzen oder das Skript mit async bzw. defer laden.
finding-fix-seo-page-health-excessive-dom = Die Zahl der HTML-Elemente senken: Wrapper ohne Funktion entfernen, lange Listen seitenweise oder bei Bedarf rendern und versteckte Varianten (mobil/Desktop) nicht im selben Markup halten.
finding-fix-seo-page-health-large-dom = Die Zahl der HTML-Elemente senken: Wrapper ohne Funktion entfernen und lange Listen seitenweise oder bei Bedarf rendern.
finding-fix-seo-page-health-images-without-dimensions = Jedem img-Element width- und height-Attribute (oder ein CSS-aspect-ratio) geben, damit der Browser den Platz vor dem Laden des Bildes reserviert.
finding-fix-seo-page-health-paste-blocked-password = Den onpaste-Handler von Passwortfeldern entfernen, damit Passwortmanager und eingefügte Passwörter funktionieren.
finding-fix-seo-page-health-offscreen-images-without-lazy = Bildern unterhalb des ersten Bildschirms loading="lazy" geben; beim Laden sichtbare Bilder davon ausnehmen.
finding-fix-seo-page-health-images-without-srcset = Bilder per srcset und sizes in mehreren Breiten anbieten, damit kleine Bildschirme eine passende Datei laden.
finding-fix-seo-page-health-missing-preconnect = Für die wenigen externen Origins, die die Seite früh braucht, <link rel="preconnect"> ergänzen.
finding-fix-seo-page-health-non-crawlable-links = Jedem Link ein echtes href-Ziel geben; für Aktionen ohne Navigation ein button-Element verwenden.
finding-fix-seo-page-health-images-without-modern-format = Fotos und Grafiken als WebP oder AVIF ausliefern, bei Bedarf mit JPEG oder PNG als Fallback in einem picture-Element.
finding-fix-seo-page-health-oversized-images = Bilder in der Größe ausliefern, in der sie angezeigt werden, und größere Varianten nur per srcset dort anbieten, wo das Layout sie braucht.
finding-fix-seo-page-health-gif-images = Animierte GIFs durch MP4- oder WebM-Video ersetzen (video-Element mit autoplay, muted, loop und playsinline).
finding-fix-seo-page-health-font-display-missing = Jeder @font-face-Regel font-display: swap (oder fallback/optional) geben, damit Text in einer Ersatzschrift erscheint, solange die Webfont lädt.
finding-fix-seo-page-health-fonts-not-preloaded = Die im ersten Bildschirm genutzten Webfonts mit <link rel="preload" as="font" type="font/woff2" crossorigin> vorladen.
finding-fix-seo-page-health-orphaned-preload = Preload-Hinweise auf Ressourcen, die die Seite nicht lädt, entfernen oder ihre URL auf die tatsächlich angeforderte Ressource korrigieren.
finding-fix-seo-page-health-lcp-image-lazy-loaded = loading="lazy" vom größten beim Laden sichtbaren Bild entfernen; Lazy Loading verzögert genau dieses Bild.
finding-fix-seo-page-health-lcp-image-without-preload = Das größte sichtbare Bild mit <link rel="preload" as="image"> vorladen, damit der Browser es früh anfordert.
finding-fix-seo-page-health-lcp-image-without-fetchpriority = Dem größten beim Laden sichtbaren Bild fetchpriority="high" geben.
finding-fix-seo-page-health-deprecated-apis = Die veralteten Browser-APIs in den Inline-Skripten durch ihre aktuellen Entsprechungen ersetzen, bevor Browser sie entfernen.
finding-fix-seo-page-health-http1-only = HTTP/2 (oder HTTP/3) auf Webserver oder CDN aktivieren; es bedient mehrere Anfragen über eine Verbindung.
finding-fix-seo-page-health-missing-compression = Brotli- oder gzip-Kompression für HTML, CSS, JavaScript und andere Textantworten auf Server oder CDN aktivieren.
finding-fix-seo-page-health-inefficient-cache = Dem HTML-Dokument eine Cache-Control-Lebensdauer geben, die zu seiner Änderungshäufigkeit passt (max-age oder s-maxage), und per ETag oder Last-Modified neu validieren, statt das Caching abzuschalten.
finding-fix-seo-page-health-inefficient-resource-cache = Statische Ressourcen mit Inhalts-Hash im Dateinamen und langer Cache-Lebensdauer ausliefern (Cache-Control: max-age=31536000, immutable).
finding-fix-seo-page-health-incorrect-content-type = Jede statische Ressource mit ihrem korrekten Content-Type-Header ausliefern (text/css, text/javascript, image/webp usw.).
finding-fix-seo-page-health-charset-mismatch = Im HTTP-Header Content-Type und in <meta charset> denselben Zeichensatz angeben, vorzugsweise UTF-8.
finding-fix-seo-page-health-hreflang-invalid = In hreflang gültige Sprachcodes nach ISO 639-1 verwenden, optional mit Region nach ISO 3166-1 (de, de-AT), oder x-default.
finding-fix-seo-page-health-jsonld-invalid = Jedem JSON-LD-Block einen @context (https://schema.org) und einen @type geben.
finding-fix-seo-page-health-sync-head-scripts = Skripte im head mit defer, async oder type="module" laden, sofern sie nicht vor der Darstellung der Seite laufen müssen.
finding-fix-seo-page-health-missing-sri = Skripten und Stylesheets von fremden Origins ein integrity-Attribut mit dem Hash der Datei (und crossorigin) geben.
finding-fix-seo-page-health-broken-fragment-links = Ankerlinks auf IDs zeigen lassen, die auf der Seite existieren, oder dem Zielelement die fehlende id geben.
finding-fix-seo-page-health-generic-link-text = Im Text jedes Links sein Ziel nennen statt „hier“ oder „mehr“.
finding-fix-seo-page-health-soft-404 = Anfragen an nicht existierende URLs mit HTTP 404 (bzw. 410 für entfernte Inhalte) beantworten, nicht mit 200.
finding-fix-seo-page-health-custom-404-invalid-status = Die Fehlerseite für nicht existierende URLs mit Status 404 oder 410 ausliefern.
finding-fix-seo-page-health-generic-404-page = Der 404-Seite eine kurze Erklärung, ein Suchfeld oder Links zu den Hauptbereichen geben.
finding-fix-seo-page-health-meta-refresh = Die Meta-Refresh-Weiterleitung durch eine serverseitige 301-Weiterleitung ersetzen.
finding-fix-seo-page-health-frames = frame und frameset durch ein einzelnes Dokument ersetzen; externe Inhalte nur wo nötig per iframe einbetten.
finding-fix-seo-page-health-url-too-long = Kürzere, lesbare URLs ohne Füllwörter verwenden und eine geänderte URL per 301 weiterleiten.
finding-fix-seo-page-health-url-too-deep = Die URL-Struktur auf höchstens fünf Pfadebenen abflachen und alte URLs per 301 weiterleiten.
finding-fix-seo-page-health-dynamic-url = Für indexierbare Inhalte URLs ohne Parameter verwenden oder per Canonical-Link auf die Fassung ohne Parameter verweisen.
finding-fix-seo-page-health-multiple-redirects = Links und Weiterleitungen direkt auf die Ziel-URL richten, damit eine Anfrage höchstens eine Weiterleitung braucht.
finding-fix-seo-page-health-redirect = Direkt auf die Ziel-URL verlinken statt auf die weiterleitende, in Navigation, Sitemap und Canonical-Links gleichermaßen.
finding-fix-seo-page-health-www-not-consolidated = Eine Host-Variante (mit oder ohne www) per 301 auf die andere weiterleiten und diese in Canonical-Links verwenden.
finding-fix-seo-page-health-trailing-slash-inconsistent = Eine URL-Form festlegen, mit oder ohne abschließenden Schrägstrich, und die andere per 301 weiterleiten.
finding-fix-seo-page-health-http-not-redirected-to-https = Jede http://-Anfrage per 301 auf dieselbe https://-URL weiterleiten.

## Mobile
finding-fix-mobile-missing-viewport = <meta name="viewport" content="width=device-width, initial-scale=1"> im head ergänzen.
finding-fix-mobile-improper-viewport = Den Viewport auf width=device-width, initial-scale=1 setzen statt auf eine feste Breite.
finding-fix-mobile-not-scalable = user-scalable=no und maximum-scale-Grenzen aus dem Viewport-Meta-Tag entfernen, damit Nutzende zoomen können.
finding-fix-mobile-small-targets = Touch-Ziele über Innenabstand oder min-width/min-height auf mindestens 44 × 44 CSS-Pixel vergrößern oder so weit auseinanderrücken, dass Nachbarn nicht versehentlich getroffen werden.
finding-fix-mobile-small-fonts = Die kleinsten Schriftgrößen auf kleinen Bildschirmen auf mindestens 12 px anheben, Fließtext möglichst auf 16 px.
finding-fix-mobile-horizontal-scroll = Das Element finden, das breiter als der Viewport ist (feste Breiten, breite Tabellen, Bilder ohne max-width), und es schrumpfen oder in einem eigenen Container scrollen lassen.
