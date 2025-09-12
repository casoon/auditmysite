# TODO: AuditMySite JSON-Datenstrukturen Integration

## 📋 Übersicht

Dieses Dokument analysiert die bestehenden JSON-Strukturen des AuditMySite-Vorgängersystems und dokumentiert, wie diese in der aktuellen Engine/CLI/Studio-Implementierung verwendet werden können.

## 📊 Vorhandene Datenstrukturen (Analysiert aus `/src/types/`)

### ✅ Vollständig definierte Haupt-Interfaces

#### 1. `FullAuditResult` (Komplettes Audit-Ergebnis)
```typescript
interface FullAuditResult {
  metadata: AuditMetadata;        // ✅ Audit-Konfiguration und Zeitstempel
  sitemap: SitemapResult;         // ✅ Sitemap-Parsing-Ergebnisse  
  pages: PageAuditResult[];       // ✅ Individual Page-Ergebnisse
  summary: AuditSummary;          // ✅ Zusammenfassung aller Seiten
}
```

#### 2. `PageAuditResult` (Einzelseiten-Audit)
```typescript
interface PageAuditResult {
  url: string;                    // ✅ Seiten-URL
  title?: string;                 // ✅ Seitentitel
  status: 'passed' | 'failed' | 'crashed';  // ✅ Status
  duration: number;               // ✅ Audit-Dauer
  auditedAt: string;             // ✅ Zeitstempel
  
  accessibility: AccessibilityResult;        // ✅ Barrierefreiheit
  performance?: PerformanceResult;           // ⚠️ Optional - Performance
  seo?: SEOResult;                          // ⚠️ Optional - SEO
  contentWeight?: ContentWeightResult;       // ⚠️ Optional - Content-Gewicht  
  mobileFriendliness?: MobileFriendlinessResult; // ⚠️ Optional - Mobile
}
```

#### 3. `AccessibilityResult` (Barrierefreiheit)
```typescript
interface AccessibilityResult {
  passed: boolean;                // ✅ Bestanden Ja/Nein
  wcagLevel: 'A' | 'AA' | 'AAA' | 'none';  // ✅ WCAG-Level
  score: number;                  // ✅ Score 0-100
  errors: AccessibilityIssue[];   // ✅ Kritische Fehler
  warnings: AccessibilityIssue[]; // ✅ Warnungen
  notices?: AccessibilityIssue[]; // ⚠️ Optional - Hinweise
  pa11yResults: {                // ✅ Pa11y-spezifische Ergebnisse
    totalIssues: number;
    runner: string;
  };
}
```

#### 4. `PerformanceResult` (Performance)
```typescript
interface PerformanceResult {
  score: number;                  // ✅ Performance-Score 0-100
  grade: 'A' | 'B' | 'C' | 'D' | 'F';  // ✅ Note
  coreWebVitals: {               // ✅ Core Web Vitals
    largestContentfulPaint: number;
    firstContentfulPaint: number;
    cumulativeLayoutShift: number;
    interactionToNextPaint?: number;
    timeToFirstByte: number;
  };
  metrics: {                     // ✅ Zusätzliche Metriken
    domContentLoaded: number;
    loadComplete: number;
    firstPaint?: number;
  };
  issues: PerformanceIssue[];    // ✅ Performance-Probleme
}
```

#### 5. `SEOResult` (Suchmaschinenoptimierung)
```typescript
interface SEOResult {
  score: number;                 // ✅ SEO-Score 0-100
  grade: 'A' | 'B' | 'C' | 'D' | 'F';  // ✅ Note
  metaTags: {                   // ✅ Meta-Tags-Analyse
    title?: { content: string; length: number; optimal: boolean; };
    description?: { content: string; length: number; optimal: boolean; };
    canonical?: string;
    openGraph: Record<string, string>;
    twitterCard: Record<string, string>;
  };
  headings: {                   // ✅ Überschriften-Struktur
    h1: string[];
    h2: string[];
    h3: string[];
    issues: string[];
  };
  images: {                     // ✅ Bilder-Analyse
    total: number;
    missingAlt: number;
    emptyAlt: number;
  };
  issues: SEOIssue[];          // ✅ SEO-Probleme
}
```

### 🔧 Utility-Funktionen verfügbar
- `calculateGrade(score: number): Grade` - Note aus Score berechnen
- `calculateOverallScore(scores: Record<string, number>): number` - Gesamtscore berechnen
- `validateScore(score: number): number` - Score validieren (0-100)
- `calculateWeightedScore()` - Gewichtete Scores
- `createBaseResult()` - Basis-Result erstellen

## 🚦 Kompatibilitäts-Status mit aktueller Implementierung

### ✅ **Bereits implementiert & funktionsfähig**

1. **Engine (Puppeteer-basiert)**
   - ✅ Generiert `AccessibilityResult` mit Pa11y
   - ✅ Sammelt Performance-Metriken (teilweise)
   - ✅ Erstellt Basis-JSON-Struktur

2. **CLI (Dart-basiert)**  
   - ✅ Lädt JSON-Results von Engine
   - ✅ Generiert HTML/CSV/JSON-Reports
   - ✅ Unterstützt multiple Formate

3. **Studio (Flutter-basiert)**
   - ✅ Kann Engine starten/stoppen
   - ✅ Zeigt Audit-Progress an
   - ✅ Lädt/speichert Results lokal
   - ✅ Export-Funktionalität

### ⚠️ **Teilweise implementiert**

1. **Performance-Analyse**
   - ✅ Grundlegende Core Web Vitals verfügbar
   - ❌ Vollständige `PerformanceResult`-Struktur fehlt
   - ❌ Performance-Issues-Detection fehlt
   - ❌ Performance-Grade-Berechnung fehlt

2. **SEO-Analyse** 
   - ❌ Komplett nicht implementiert
   - ❌ Meta-Tags-Analyse fehlt
   - ❌ Heading-Struktur-Analyse fehlt
   - ❌ Image-Alt-Text-Analyse fehlt

3. **Content Weight Analyse**
   - ❌ Komplett nicht implementiert
   - ❌ Resource-Size-Tracking fehlt
   - ❌ Optimization-Recommendations fehlen

### ❌ **Nicht implementiert**

1. **Mobile Friendliness**
   - ❌ Mobile-Viewport-Tests
   - ❌ Touch-Target-Size-Analyse
   - ❌ Mobile-Navigation-Tests

2. **Content Optimization**
   - ❌ Image-Compression-Analyse
   - ❌ CSS/JS-Minification-Checks
   - ❌ GZIP-Compression-Analyse

3. **Strukturierte Issues**
   - ❌ Einheitliches `StructuredIssue`-Format
   - ❌ Cross-Category-Issue-Aggregation

## 📈 Was funktioniert bereits (CURRENT STATE)

### Engine API (`localhost:3000`)
```bash
✅ POST /audit - Startet Audit mit sitemap_url
✅ GET /health - Health check
✅ WebSocket /ws - Live-Events während Audit
```

### CLI (`dart run`)
```bash  
✅ Lädt JSON von Engine
✅ --format=html,csv,json - Multiple Ausgabeformate
✅ Generiert vollständige Reports
```

### Studio App
```bash
✅ Engine-Connection & Management
✅ Audit-Progress mit WebSocket-Updates
✅ Results laden/speichern/exportieren
✅ Settings-Management (SharedPreferences)
✅ Export-Format-Auswahl
```

### Beispiel funktionierender JSON-Output (reduziert)
```json
{
  "url": "https://example.com",
  "title": "Example Page",
  "status": "passed",
  "duration": 1500,
  "accessibility": {
    "passed": true,
    "wcagLevel": "AA", 
    "score": 87,
    "errors": [],
    "warnings": [...]
  },
  "performanceMetrics": {
    "loadTime": 1200,
    "firstContentfulPaint": 800,
    "largestContentfulPaint": 1100
  }
}
```

## 🎯 TODO: Was muss implementiert werden

### 🔥 **Hochpriorität (Kurzfristig)**

1. **Engine erweitern - Performance-Vollanalyse**
   ```typescript
   // In engine/audit.js hinzufügen:
   - PerformanceIssue-Detection (LCP > 2.5s, CLS > 0.1, etc.)
   - Grade-Berechnung für Performance
   - Metrics-Vollerfassung (TTFB, INP, etc.)
   ```

2. **Engine erweitern - SEO-Basis**
   ```typescript  
   // In engine/audit.js hinzufügen:
   - Meta-Tags extrahieren (title, description, canonical)
   - H1-H6 Heading-Struktur analysieren
   - Image alt-text überprüfen
   - OpenGraph/TwitterCard-Tags
   ```

3. **JSON-Schema-Kompatibilität**
   ```typescript
   // In engine/audit.js:
   - Output an FullAuditResult-Schema anpassen
   - PageAuditResult-Struktur implementieren  
   - AuditSummary-Generierung hinzufügen
   ```

### 📊 **Medium-Priorität (Mittelfristig)**

4. **Studio UI für neue Datentypen**
   ```dart
   // In Flutter Studio:
   - Performance-Metriken-Anzeige
   - SEO-Results-Widgets  
   - Grade-Visualization (A-F Badges)
   - Certificate-Level-Display
   ```

5. **CLI Report-Templates erweitern**
   ```dart
   // In CLI:
   - HTML-Templates für Performance/SEO
   - CSV-Export für alle Kategorien
   - PDF-Export-Option
   ```

6. **Content Weight Analysis**
   ```typescript
   // In engine:
   - Resource-Sizes erfassen (CSS, JS, Images)
   - GZIP-Compression-Status
   - Optimization-Recommendations
   ```

### ⚡ **Niedrige Priorität (Langfristig)**

7. **Mobile Friendliness**
   ```typescript
   - Viewport-Configuration-Tests
   - Touch-Target-Size-Analyse
   - Mobile-Navigation-Usability
   ```

8. **Advanced Features**
   ```typescript
   - Lighthouse-Integration
   - Security-Headers-Analysis
   - Structured-Data-Validation
   ```

## 🛠️ Konkrete Implementierungsschritte

### Phase 1: Performance & SEO Engine-Integration (1-2 Wochen)

1. **Engine Performance erweitern**
   ```bash
   cd auditmysite_engine
   # In audit.js hinzufügen:
   - calculatePerformanceGrade() Funktion
   - detectPerformanceIssues() Funktion
   - Vollständige PerformanceResult-Struktur
   ```

2. **Engine SEO-Basis**
   ```bash
   # In audit.js hinzufügen:
   - extractMetaTags() Funktion
   - analyzeHeadingStructure() Funktion  
   - checkImageAltText() Funktion
   - generateSEOResult() Funktion
   ```

3. **JSON-Output anpassen**
   ```bash
   # Engine Output erweitern:
   - FullAuditResult-kompatible Struktur
   - AuditMetadata hinzufügen
   - AuditSummary berechnen
   ```

### Phase 2: Studio UI-Erweiterung (1 Woche)

4. **Flutter Widgets für neue Daten**
   ```bash
   cd auditmysite_studio  
   # Neue Widgets erstellen:
   - PerformanceMetricsCard
   - SEOResultsCard
   - GradeBadgeWidget
   - CertificateLevelIndicator
   ```

5. **Results-View erweitern**
   ```bash
   # In results_view.dart:
   - Tabs für Performance/SEO
   - Charts für Metriken-Anzeige
   - Issue-Kategorisierung
   ```

### Phase 3: CLI & Export-Verbesserung (3-5 Tage)

6. **CLI Templates erweitern**
   ```bash
   cd auditmysite_cli
   # Neue Report-Templates:
   - Performance-HTML-Section
   - SEO-Analysis-Section  
   - Multi-Category-CSV-Export
   ```

## 🧪 Test-Strategie

### Testbare URLs für Implementierung
```bash
# Performance-Tests:
https://web.dev/               # Gute Performance
https://example.com/           # Baseline  
https://httparchive.org/       # Langsame Seite

# SEO-Tests:
https://google.com/            # Perfect SEO
https://example.com/           # Basic SEO
https://httpbin.org/html       # No SEO
```

### Demo-Daten erstellen
```bash
# Für Studio-Development:
mkdir demo_data  
# JSON-Files mit unterschiedlichen Score-Ranges erstellen
- perfect_site.json (90-100 scores)
- good_site.json (70-90 scores)  
- poor_site.json (0-50 scores)
```

## 🎉 Erwartete Ergebnisse nach Implementierung

Nach Umsetzung aller TODO-Punkte wird das System liefern:

### 📊 **Vollständige Audit-Ergebnisse**
- ✅ Accessibility (Pa11y-basiert) - **Bereits vorhanden**
- ✅ Performance (Core Web Vitals) - **Nach Phase 1**
- ✅ SEO (Meta-Tags, Headings, Images) - **Nach Phase 1**
- ✅ Content Weight (Resource-Analysis) - **Nach Phase 3**
- ✅ Mobile Friendliness - **Nach Phase 4**

### 🎨 **Studio App Features**
- ✅ Multi-Category-Dashboard
- ✅ Performance-Charts & Metriken
- ✅ SEO-Recommendations
- ✅ Grade-Visualization (A-F)
- ✅ Certificate-Levels (Platinum, Gold, etc.)
- ✅ Detaillierte Issue-Listen
- ✅ Export in allen Formaten

### 📄 **CLI Report-Funktionen**  
- ✅ HTML-Reports mit allen Kategorien
- ✅ CSV-Export für Datenanalyse
- ✅ JSON-API für Integrationen
- ✅ PDF-Export für Präsentationen

### 🔄 **Voll funktionsfähiger Workflow**
```bash
1. Studio App starten
2. Sitemap-URL eingeben  
3. Audit starten (alle Kategorien)
4. Live-Progress verfolgen
5. Vollständige Ergebnisse anzeigen
6. In gewünschtem Format exportieren
7. Ergebnisse teilen/präsentieren
```

---

## 📝 Nächste Schritte

1. **✅ Diese TODO-Analyse fertiggestellt**
2. **🔥 Phase 1 starten: Performance & SEO Engine-Integration**
3. **📊 Demo-JSON-Daten erstellen für Studio-Development**
4. **🎨 Studio UI für neue Datentypen implementieren**

**Zeitschätzung Gesamt: 3-4 Wochen für vollständige Implementierung**

---

*Last Updated: {current_date}*
*Status: Analysis Complete ✅ - Ready for Implementation 🚀*
