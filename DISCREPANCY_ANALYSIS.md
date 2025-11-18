# Analyse der Unterschiede: AuditMySite vs. PageSpeed Insights

**Website:** https://www.aib-bauplanung.de/  
**Datum:** 18. November 2025  
**Problem:** Signifikante Unterschiede in den Ergebnissen

---

## 🚨 HAUPTPROBLEM: Widersprüchliche Bewertungen

### Vergleich der Scores

| Kategorie | PageSpeed | AuditMySite | Diskrepanz |
|-----------|-----------|-------------|------------|
| **Performance** | 63/100 | ❌ Kein Score | Nicht vergleichbar |
| **Accessibility** | 74/100 | Pass/Fail (50 Issues) | ❓ Unklar |
| **Mobile** | ℹ️ Keine separate Bewertung | 93/100 (A) | ✅ AuditMySite besser |
| **SEO** | 77/100 | ❌ Kein Score | Nicht vergleichbar |
| **Best Practices** | 96/100 | ❌ Nicht getestet | Fehlt komplett |

---

## 🔍 DETAILANALYSE: Warum unterscheiden sich die Ergebnisse?

### 1. Mobile-Friendliness: MASSIVER WIDERSPRUCH

#### PageSpeed Insights sagt:
```
Performance (Mobile): 63/100 ❌ SCHLECHT
- LCP: 8,6s (Ziel: ≤2,5s)
- CLS: 0.187 (Ziel: ≤0,1)
- FCP: 1,7s
```

#### AuditMySite sagt:
```
✅ Mobile-friendliness analysis completed in 5010ms
📱 Mobile Score: 93/100 (Grade: A) ✅ AUSGEZEICHNET
```

### ❓ WIE KANN DAS SEIN?

**Hypothese 1: Verschiedene Metriken**
- **PageSpeed "Mobile Performance"** = Core Web Vitals (LCP, FCP, CLS, TBT, SI)
- **AuditMySite "Mobile-Friendliness"** = Usability (Touch-Targets, Viewport, Lesbarkeit)

**Mögliche Erklärung:**
```
PageSpeed fragt: "Ist die Seite SCHNELL auf Mobile?"
→ Antwort: NEIN (63/100) - LCP ist 8,6s

AuditMySite fragt: "Ist die Seite BENUTZBAR auf Mobile?"  
→ Antwort: JA (93/100) - Buttons groß genug, kein horizontales Scrollen
```

**PROBLEM:** Beide Scores heißen "Mobile", bedeuten aber etwas völlig anderes!

---

### 2. Accessibility: Unterschiedliche Zählweisen?

#### PageSpeed Insights:
```
Accessibility: 74/100
- Bildelemente haben keine [alt]-Attribute
- Links haben keinen erkennbaren Namen
- Kontrastverhältnis nicht ausreichend
- Überschriften nicht in Reihenfolge
- Dokument hat keine Hauptmarkierung
- Identische Links haben denselben Zweck
```

#### AuditMySite:
```
Total Issues: 50
- 14 color-contrast Errors
- 2 image-alt Errors  
- 1 link-name Error
- 3 heading-order Warnings
- 1 landmark-one-main Warning
- 6 region Warnings
- 2 images without alt attribute (warning)
- 1 button without aria-label (warning)
```

### ❓ DISKREPANZ-ANALYSE

**Gleiche Probleme erkannt:**
- ✅ Alt-Text fehlt
- ✅ Farbkontrast-Probleme
- ✅ Link-Name fehlt
- ✅ Überschriften-Reihenfolge
- ✅ Hauptmarkierung fehlt

**Unterschiedliche Zählweise:**

PageSpeed gruppiert:
- "Bildelemente haben keine [alt]-Attribute" = 1 Fehlertyp
- Anzahl betroffener Bilder: unklar

AuditMySite listet einzeln:
- 2 Bilder ohne alt (Error)
- 2 Bilder ohne alt (Warning) - Deduplizierung?
- Jedes Element separat gezählt

**Color-Contrast:**
- AuditMySite: **14 separate Errors** (jedes Element einzeln)
- PageSpeed: **1 Fehlertyp** "Kontrastverhältnis nicht ausreichend"

### 🎯 SCORING-UNTERSCHIED

**PageSpeed-Methodik (vermutet):**
```javascript
// Lighthouse-Scoring (vereinfacht)
score = 100 - (errorTypes * penalty)
// 6 Fehlertypen à ~4 Punkte = ~24 Punkte Abzug
// Score: 100 - 24 = 76 ≈ 74/100
```

**AuditMySite-Methodik (aktuell):**
```javascript
// Pass/Fail nur
if (errors.length > 0) {
  return 'FAILED';
}
// Kein numerischer Score!
```

**PROBLEM:** AuditMySite zählt **jedes betroffene Element**, PageSpeed zählt **Fehlertypen**!

---

### 3. Performance: Komplett unterschiedliche Messungen

#### PageSpeed zeigt:
```
Performance: 63/100

Core Web Vitals (Lab Data):
- FCP: 1,7s
- LCP: 8,6s ❌
- TBT: 0ms
- CLS: 0.187 ⚠️
- SI: 4,8s

Optimierungen:
- Cache-TTL: 3.331 KiB Einsparung
- Bilder: 1.117 KiB Einsparung  
- Render-blocking: 1.340ms
- Ungenutztes CSS: 44 KiB
```

#### AuditMySite zeigt:
```
✅ Mobile-friendliness analysis completed in 5010ms
📱 Mobile Score: 93/100 (Grade: A)

❌ KEINE Performance-Metriken in CLI!
❌ KEINE LCP/FCP/CLS-Werte sichtbar!
```

### ❓ WAS MISST AUDITMYSITE WIRKLICH?

Laut README:
> **⚡ Performance Analysis**
> - Core Web Vitals: LCP, FCP, CLS, INP, TTFB
> - Performance score (0-100) and letter grade (A-F)

**ABER:** In der CLI-Ausgabe sehe ich:
- ✅ Mobile-Friendliness: 93/100
- ❌ Performance: **NICHT AUSGEGEBEN**

**Mögliche Probleme:**
1. Performance wird analysiert, aber **nicht in CLI angezeigt**
2. Performance-Score wird nicht berechnet
3. Mobile-Friendliness wird mit Performance verwechselt
4. Core Web Vitals nur im HTML-Report, nicht in CLI

---

## 🐛 KONKRETE BUGS IDENTIFIZIERT

### Bug #1: Mobile-Friendliness vs. Performance verwechselt
```
AKTUELL:
✅ Mobile-friendliness analysis completed
📱 Mobile Score: 93/100 (Grade: A)

NUTZER DENKT:
"Super! Mobile Performance ist exzellent!"

REALITÄT:
LCP ist 8,6 Sekunden - die Seite ist EXTREM langsam!
```

**GEFAHR:** Nutzer werden in falsche Sicherheit gewiegt!

---

### Bug #2: Performance-Score wird nicht ausgegeben
```
ERWARTET (laut README):
⚡ Performance: ✅
📊 Performance Score: 63/100 (Grade: D)
   FCP: 1.7s
   LCP: 8.6s ❌
   CLS: 0.187 ⚠️

TATSÄCHLICH:
🚀 Analysis Features:
   ⚡ Performance: ✅
   
[...nichts weiter...]
```

**TODO:**
- [ ] Performance-Score berechnen
- [ ] In CLI ausgeben
- [ ] Core Web Vitals anzeigen

---

### Bug #3: Accessibility-Counting irreführend
```
AKTUELL:
Total Issues: 50
❌ 14 color-contrast errors

NUTZER DENKT:
"Oh nein, 50 Fehler! Die Seite ist katastrophal!"

REALITÄT:
14 Elemente mit gleichem Farbkontrast-Problem
= 1 CSS-Regel, die gefixt werden muss

PageSpeed:
"Kontrastverhältnis nicht ausreichend" = 1 Problem
```

**TODO:**
- [ ] Fehler nach Typ gruppieren
- [ ] "14 Elemente betroffen von 1 Problem" anzeigen
- [ ] Severity-gewichteten Score berechnen

---

### Bug #4: Keine Optimierungs-Empfehlungen
```
PageSpeed zeigt:
✅ "Bildübermittlung verbessern: 1.117 KiB Einsparung"
✅ "Cache verwenden: 3.331 KiB Einsparung"
✅ Konkrete Dateien mit KB-Zahlen

AuditMySite zeigt:
❌ Nur "Issues found"
❌ Keine Einsparungs-Berechnungen
❌ Keine priorisierten Empfehlungen
```

**TODO:**
- [ ] Performance-Budgets berechnen
- [ ] Einsparungen schätzen
- [ ] Priorisierte Liste generieren

---

## 🎯 WARUM SIND DIE ERGEBNISSE SO UNTERSCHIEDLICH?

### Methodische Unterschiede

| Aspekt | PageSpeed Insights | AuditMySite | Impact |
|--------|-------------------|-------------|---------|
| **Test-Umgebung** | Emuliertes Moto G Power, Slow 4G | Playwright, keine Throttling (?) | ⚠️ HOCH |
| **Scoring** | Lighthouse-Methodik (gewichtet) | Pass/Fail oder Mobile-only | ⚠️ HOCH |
| **Fehler-Zählung** | Nach Typ gruppiert | Jedes Element einzeln | ⚠️ MITTEL |
| **Performance-Messung** | Core Web Vitals (Lab) | Core Web Vitals (?) | ❓ UNKLAR |
| **Mobile-Definition** | Performance + Usability | Nur Usability | ⚠️ KRITISCH |

---

## 🔬 EXPERIMENTE ZUR VERIFIZIERUNG

### Experiment 1: Ist Network-Throttling aktiv?

**Hypothese:** AuditMySite misst ohne Network-Throttling, deshalb bessere Performance

**Test:**
```bash
# PageSpeed: Slow 4G (400ms RTT, 400 Kbps download)
# AuditMySite: ???
```

**TODO:**
- [ ] Dokumentation prüfen: Welche Network-Bedingungen?
- [ ] Code-Review: Ist Throttling implementiert?
- [ ] Vergleich mit/ohne Throttling

**Erwartung:**
- Mit Throttling: LCP ~8s (wie PageSpeed)
- Ohne Throttling: LCP ~2s (bessere Ergebnisse)

---

### Experiment 2: Was zeigt der HTML-Report?

**Hypothese:** Core Web Vitals sind im HTML-Report, nur nicht in CLI

**TODO:**
- [ ] HTML-Report öffnen und Performance-Section finden
- [ ] LCP/FCP/CLS-Werte prüfen
- [ ] Mit PageSpeed vergleichen

**Mögliche Ergebnisse:**
1. **Gleiche Werte** → CLI-Ausgabe fehlt nur
2. **Bessere Werte** → Kein Throttling aktiv
3. **Keine Werte** → Performance-Analyse fehlgeschlagen

---

### Experiment 3: Accessibility-Score rekonstruieren

**Hypothese:** Aus 50 Issues kann man einen Score ableiten

**PageSpeed-Berechnung (vereinfacht):**
```javascript
// Lighthouse Accessibility-Scoring
const audits = {
  'color-contrast': { weight: 3, score: 0 },      // 14 Fehler
  'image-alt': { weight: 10, score: 0 },          // 2 Fehler
  'link-name': { weight: 3, score: 0 },           // 1 Fehler
  'heading-order': { weight: 2, score: 0 },       // 3 Warnings
  'landmark-one-main': { weight: 3, score: 0 },   // 1 Warning
  'region': { weight: 1, score: 0 }               // 6 Warnings
};

// Gewichtete Summe
// Score ≈ 70-75/100
```

**AuditMySite-Berechnung (zu implementieren):**
```javascript
function calculateAccessibilityScore(issues) {
  const weights = {
    'color-contrast': 3,
    'image-alt': 10,
    'link-name': 3,
    'heading-order': 2,
    'landmark-one-main': 3,
    'region': 1
  };
  
  let totalWeight = 0;
  let failedWeight = 0;
  
  // Fehler nach Typ gruppieren
  const grouped = groupByType(issues);
  
  for (const [type, count] of Object.entries(grouped)) {
    totalWeight += weights[type] || 1;
    if (count > 0) {
      failedWeight += weights[type] || 1;
    }
  }
  
  return Math.round((1 - failedWeight / totalWeight) * 100);
}
```

**TODO:**
- [ ] Implementieren
- [ ] Mit PageSpeed vergleichen
- [ ] Kalibrieren

---

## 📋 KRITISCHE TODOS ZUR BEHEBUNG

### 1. Klarstellung: Was bedeutet "Mobile Score 93/100"?

**Aktuell:**
```
📱 Mobile Score: 93/100 (Grade: A)
```

**Problem:** Nutzer denken das ist Performance!

**Lösung:**
```
📱 Mobile Usability: 93/100 (Grade: A)
   ✅ Touch targets adequate (48x48px)
   ✅ Viewport configured correctly
   ✅ Font sizes readable
   ⚠️ Minor spacing issues

⚡ Mobile Performance: 63/100 (Grade: D)
   ❌ LCP: 8.6s (Poor - Target: ≤2.5s)
   ✅ FCP: 1.7s (Good)
   ⚠️ CLS: 0.187 (Needs Improvement)
```

**TODO:**
- [ ] Umbenennen: "Mobile-Friendliness" → "Mobile Usability"
- [ ] Separaten "Mobile Performance"-Score hinzufügen
- [ ] Beide Scores in CLI ausgeben

---

### 2. Performance-Score implementieren

**TODO:**
- [ ] Core Web Vitals messen (bereits implementiert?)
- [ ] Lighthouse-kompatible Gewichtung verwenden
- [ ] Performance-Score berechnen
- [ ] In CLI prominent anzeigen:
  ```
  ⚡ Performance Analysis:
     📊 Overall Score: 63/100 (Grade: D)
     
     Core Web Vitals:
     🎯 FCP: 1.7s ✅ (Good)
     🎯 LCP: 8.6s ❌ (Poor - 343% slower than target)
     🎯 TBT: 0ms ✅ (Good)
     🎯 CLS: 0.187 ⚠️ (Needs Improvement - 87% above target)
     🎯 SI: 4.8s ⚠️ (Needs Improvement)
  ```

---

### 3. Accessibility-Score implementieren

**TODO:**
- [ ] Fehler nach Typ gruppieren
- [ ] Severity-Gewichtung anwenden
- [ ] Numerischen Score berechnen
- [ ] In CLI ausgeben:
  ```
  ♿ Accessibility Analysis:
     📊 Overall Score: 74/100 (Grade: C)
     
     Issues by Type:
     ❌ Color Contrast (14 elements) - High Impact
     ❌ Missing Alt Text (2 images) - Critical
     ❌ Link without name (1 link) - Critical
     ⚠️ Heading order (3 instances) - Medium
     ⚠️ Missing landmarks (7 sections) - Low
  ```

---

### 4. Network-Throttling verifizieren

**TODO:**
- [ ] Dokumentieren: Welche Network-Profile werden verwendet?
- [ ] Playwright DeviceDescriptors prüfen
- [ ] Optional: Throttling konfigurierbar machen:
  ```bash
  auditmysite <url> --throttling slow-4g
  auditmysite <url> --throttling fast-3g
  auditmysite <url> --throttling none  # Desktop
  ```

---

### 5. Optimierungs-Empfehlungen hinzufügen

**TODO:**
- [ ] Cache-Header analysieren
- [ ] Bild-Optimierungs-Potenzial berechnen
- [ ] Render-blocking-Ressourcen identifizieren
- [ ] Ungenutztes CSS detektieren
- [ ] Priorisierte Liste mit KB-Einsparungen:
  ```
  💡 Optimization Opportunities:
  
  High Impact (Save 4.4 MB):
  1. ⚡ Enable browser caching (3.3 MB)
  2. 🖼️ Optimize images (1.1 MB)
  
  Medium Impact (Save 51 KB):
  3. 📦 Remove unused CSS (44 KB)
  4. 🗜️ Compress CSS (7 KB)
  
  Low Impact (Save 1.3s):
  5. ⏱️ Eliminate render-blocking (1.3s)
  ```

---

## 🎯 ERWARTETE VERBESSERUNGEN

### Nach Implementierung der TODOs:

**CLI-Output wird zeigen:**
```
🚀 AuditMySite v2.2.0 - Professional Website Testing

📊 Overall Results for https://www.aib-bauplanung.de/

┌─────────────────────┬─────────┬───────┐
│ Category            │ Score   │ Grade │
├─────────────────────┼─────────┼───────┤
│ ⚡ Performance      │ 63/100  │ D     │
│ ♿ Accessibility    │ 74/100  │ C     │
│ 🔍 SEO             │ 77/100  │ C     │
│ ✅ Best Practices  │ 96/100  │ A     │
│ 📱 Mobile Usability│ 93/100  │ A     │
└─────────────────────┴─────────┴───────┘

⚡ Performance Details:
   🎯 LCP: 8.6s ❌ (343% over target of 2.5s)
   💡 Top opportunity: Enable caching (save 3.3 MB)

♿ Accessibility Details:
   ❌ 6 unique issue types affecting 27 elements
   💡 Top priority: Fix color contrast (14 elements)

📈 Compared to PageSpeed Insights:
   ✅ Performance: ±0 points (aligned)
   ✅ Accessibility: ±0 points (aligned)
   ✅ SEO: ±0 points (aligned)
   ✅ Best Practices: ±0 points (aligned)
```

---

## 📊 VALIDIERUNGS-CHECKLISTE

Nach Implementierung validieren mit:

- [ ] **Gleiche Website testen:** www.aib-bauplanung.de
- [ ] **Scores vergleichen:** ±5% Abweichung zu PageSpeed akzeptabel
- [ ] **Core Web Vitals:** LCP/FCP/CLS ±10% Abweichung
- [ ] **Accessibility:** Issue-Count nach Typ vergleichen
- [ ] **Best Practices:** Security-Header-Übereinstimmung

**Erfolgs-Kriterien:**
- ✅ Performance-Score: 63 ±5 (58-68)
- ✅ Accessibility-Score: 74 ±5 (69-79)
- ✅ Core Web Vitals innerhalb 10% der PageSpeed-Werte
- ✅ Keine verwirrenden oder widersprüchlichen Aussagen

---

## 🔍 ZUSÄTZLICHE ANALYSE: Warum ist AuditMySite aktuell "optimistischer"?

### Hypothese-Matrix:

| Hypothese | Wahrscheinlich | Evidence | Impact |
|-----------|----------------|----------|--------|
| Kein Network-Throttling | ⚠️ HOCH | Mobile: 93 vs. 63 | Performance-Werte unrealistisch |
| Mobile ≠ Performance | ✅ SEHR HOCH | README vs. Output | Verwirrende Terminologie |
| Performance nicht ausgegeben | ✅ BESTÄTIGT | CLI-Log | Kritische Info fehlt |
| Fehler-Deduplizierung fehlt | ⚠️ HOCH | 50 Issues vs. 6 Types | Überbewertung von Problemen |
| Kein Scoring-System | ✅ BESTÄTIGT | Pass/Fail only | Keine Vergleichbarkeit |

---

**Fazit:** Die Unterschiede sind hauptsächlich auf **fehlende Features** und **unklare Terminologie** zurückzuführen, nicht auf unterschiedliche Mess-Methoden.

**Nächste Schritte:**
1. HTML-Report analysieren (sind die Daten da?)
2. Performance-Score-Berechnung implementieren
3. CLI-Output verbessern
4. Terminologie klären (Mobile Usability ≠ Mobile Performance)

---

**Erstellt:** 18.11.2025  
**Nächster Review:** Nach Sichtung des HTML-Reports  
**Priorität:** KRITISCH - Nutzer-Vertrauen gefährdet
