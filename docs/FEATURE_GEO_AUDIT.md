# Feature Specification: Geo Audit

**Status:** 📋 Geplant (Nach Phase 4)  
**Priorität:** Medium  
**Aufwand:** ~2-3 Wochen

---

## 🌍 Übersicht

Geo Audits sind spezialisierte Prüfungen, die analysieren, wie gut eine Website auf verschiedene Regionen, Sprachen und Märkte optimiert ist. Sie sind eine geografisch fokussierte Erweiterung der bestehenden SEO- und Performance-Analysen.

**Ziel:** Sicherstellen, dass Menschen und Suchmaschinen in verschiedenen Regionen die bestmögliche, relevante Version einer Seite sehen und nutzen können.

---

## 📋 Analyse-Bereiche

### 1. Technische Lokalisierung

#### 1.1 hreflang-Tags
**Was wird geprüft:**
- Vorhandensein von `<link rel="alternate" hreflang="...">`
- Korrekte Syntax und Format (ISO 639-1 für Sprache, ISO 3166-1 für Land)
- Bidirektionale Verlinkung (jede Variante verweist auf alle anderen)
- x-default Fallback vorhanden
- Selbstreferenzierende hreflang-Tags

**Probleme erkennen:**
- Fehlende hreflang-Tags
- Syntaxfehler (z.B. "de_DE" statt "de-DE")
- Einseitige Verlinkungen
- Defekte URLs in hreflang-Tags
- Konflikte mit Canonical-Tags

**Output:**
```typescript
interface HreflangAnalysis {
  isImplemented: boolean;
  totalLanguageVariants: number;
  correctlyImplemented: string[];  // ['de-DE', 'en-US', 'fr-FR']
  errors: HreflangError[];
  warnings: HreflangWarning[];
  hasXDefault: boolean;
  bidirectionalityScore: number;  // 0-100
  recommendations: string[];
}
```

#### 1.2 Geo-Targeting
**Was wird geprüft:**
- Meta-Tag `<meta name="geo.region" content="DE-BY">`
- HTML lang-Attribut
- Server-Header (Content-Language)
- Strukturierte Daten (address, geo-Location)

#### 1.3 Server & CDN
**Was wird geprüft:**
- Server-Standort (via IP-Geolocation)
- CDN-Nutzung und Verteilung
- PoP (Point of Presence) Abdeckung
- DNS-Auflösungszeiten nach Region
- TLS-Handshake-Latenz

**Output:**
```typescript
interface ServerGeoAnalysis {
  serverLocation: {
    country: string;
    city: string;
    coordinates: { lat: number; lon: number };
    provider: string;
  };
  cdnDetected: boolean;
  cdnProvider?: string;
  popLocations: string[];  // ['Frankfurt', 'Paris', 'London']
  regionalLatency: {
    [region: string]: {
      dns: number;      // ms
      connect: number;  // ms
      ttfb: number;     // ms
    };
  };
  score: number;
  recommendations: string[];
}
```

---

### 2. Sprach- und Kulturadaption

#### 2.1 Content-Qualität
**Was wird geprüft:**
- Spracherkennung vs. deklarierte Sprache
- Qualitätsindikatoren für Übersetzungen:
  - Unübersetzte Begriffe
  - Maschinelle Übersetzungsmuster
  - Gemischte Sprachen im Content
- Kulturelle Anpassungen:
  - Datumsformate
  - Zahlenformate
  - Währungen
  - Maßeinheiten
  - Lokale Feiertage/Events

**Output:**
```typescript
interface CulturalAdaptationAnalysis {
  declaredLanguage: string;
  detectedLanguage: string;
  languageMatch: boolean;
  translationQuality: {
    score: number;
    indicators: {
      untranslatedTerms: string[];
      mixedLanguages: boolean;
      machineTranslationLikelihood: number;  // 0-100
    };
  };
  culturalAdaptation: {
    dateFormat: 'localized' | 'generic' | 'mixed';
    numberFormat: 'localized' | 'generic';
    currency: string[];
    measurements: 'metric' | 'imperial' | 'mixed';
    localizedContent: boolean;
  };
  score: number;
  recommendations: string[];
}
```

#### 2.2 Meta-Daten Lokalisierung
- Title & Description in Zielsprache
- Alt-Texte für Bilder
- Schema.org Markup mit lokalisierten Daten
- OpenGraph & Twitter Cards

---

### 3. Local SEO

#### 3.1 NAP-Konsistenz (Name, Address, Phone)
**Was wird geprüft:**
- Strukturierte Daten (LocalBusiness Schema)
- Footer/Header Informationen
- Kontaktseite
- Konsistenz über alle Vorkommen
- Formatierung nach lokalen Standards

**Output:**
```typescript
interface NAPAnalysis {
  found: boolean;
  consistency: {
    name: { value: string; occurrences: number; variations: string[] };
    address: { value: string; occurrences: number; variations: string[] };
    phone: { value: string; occurrences: number; variations: string[] };
  };
  schemaMarkup: {
    present: boolean;
    valid: boolean;
    type: string;  // 'LocalBusiness', 'Restaurant', etc.
  };
  consistencyScore: number;  // 0-100
  recommendations: string[];
}
```

#### 3.2 Lokale Sichtbarkeit
- Google Business Profile Verlinkung
- Strukturierte Daten für lokale Entitäten
- Lokale Keywords in Content
- Geo-spezifische Landing Pages
- Regionale Backlink-Analyse (extern)

---

### 4. Regionale User Experience

#### 4.1 Performance nach Region
**Was wird geprüft:**
- Ladezeiten aus verschiedenen Regionen (via Lighthouse CI / WebPageTest API)
- CDN-Effizienz
- Resource-Loading-Strategien
- Image-Optimization für verschiedene Märkte

**Output:**
```typescript
interface RegionalPerformanceAnalysis {
  regions: {
    [regionCode: string]: {
      name: string;
      metrics: {
        lcp: number;
        fcp: number;
        ttfb: number;
        cls: number;
        inp: number;
      };
      score: number;
      grade: 'A' | 'B' | 'C' | 'D' | 'F';
    };
  };
  performanceGap: number;  // Differenz zwischen beste/schlechteste Region
  recommendations: string[];
}
```

#### 4.2 Rechtliche Konformität
**Was wird geprüft:**
- Cookie-Banner nach Region (DSGVO, CCPA, etc.)
- Datenschutzerklärung vorhanden & lokalisiert
- Impressum (für DE/AT/CH)
- AGB lokalisiert
- Rechtliche Hinweise nach Markt

**Output:**
```typescript
interface LegalComplianceAnalysis {
  gdprCompliance: {
    cookieBanner: boolean;
    privacyPolicy: boolean;
    privacyPolicyLanguage: string;
    dataProtectionOfficer: boolean;
    rightToErasure: boolean;
    score: number;
  };
  regionalRequirements: {
    [region: string]: {
      imprint: boolean;          // DE/AT/CH
      termsOfService: boolean;
      ageVerification: boolean;  // z.B. Alkohol/Tabak
      localTaxInfo: boolean;     // VAT/GST
      compliant: boolean;
      missingElements: string[];
    };
  };
  overallScore: number;
  recommendations: string[];
}
```

#### 4.3 Regionale Funktionen
- Versandoptionen nach Land
- Zahlungsmethoden (lokal bevorzugt)
- Währungsumrechnung
- Lokale Kontaktmöglichkeiten
- Regionale Angebote/Promotions

---

## 🏗️ Implementierung

### Phase 1: Foundation (Woche 1-2)

#### Neue Analyzer erstellen
```typescript
// src/analyzers/geo-audit-analyzer.ts
export class GeoAuditAnalyzer implements BaseAnalyzer {
  async analyze(page: Page, url: string): Promise<GeoAuditResult> {
    const [
      hreflang,
      serverGeo,
      cultural,
      nap,
      legal
    ] = await Promise.all([
      this.analyzeHreflang(page),
      this.analyzeServerLocation(url),
      this.analyzeCulturalAdaptation(page),
      this.analyzeNAPData(page),
      this.analyzeLegalCompliance(page)
    ]);
    
    return {
      overallScore: this.calculateScore({ hreflang, serverGeo, cultural, nap, legal }),
      hreflang,
      serverGeo,
      cultural,
      nap,
      legal,
      recommendations: this.generateRecommendations(...)
    };
  }
}
```

#### Types definieren
```typescript
// src/types/geo-audit.ts
export interface GeoAuditResult extends BaseAnalysisResult {
  hreflang: HreflangAnalysis;
  serverGeo: ServerGeoAnalysis;
  cultural: CulturalAdaptationAnalysis;
  nap: NAPAnalysis;
  regionalPerformance?: RegionalPerformanceAnalysis;
  legal: LegalComplianceAnalysis;
}
```

### Phase 2: Core Features (Woche 3-4)

1. **hreflang Parser**
   - HTML Head scannen
   - Syntax-Validierung
   - Bidirektionalität prüfen
   - Conflict-Detection

2. **Server Geolocation**
   - IP → Location Service (ipinfo.io, MaxMind)
   - CDN Detection (via Headers, DNS)
   - Latency Measurement (via Puppeteer)

3. **Cultural Adaptation**
   - Language Detection (fast-langdetect, franc)
   - Format-Pattern-Recognition (Regex)
   - Schema.org Extraction

4. **NAP Extraction**
   - Schema.org LocalBusiness
   - Footer/Contact-Page Scraping
   - Consistency-Checking (Levenshtein)

### Phase 3: Advanced Features (Woche 5-6)

1. **Multi-Region Performance**
   - WebPageTest API Integration
   - Lighthouse CI für verschiedene Locations
   - CDN-Effectiveness Score

2. **Legal Compliance**
   - Cookie-Banner Detection
   - Policy-Page Analysis
   - Region-specific Requirements

3. **Report Integration**
   - Geo Audit Section in HTML Report
   - Interactive Maps für Performance
   - hreflang Visualisierung

---

## 📊 Scoring-Algorithmus

```typescript
function calculateGeoAuditScore(data: GeoAuditResult): number {
  const weights = {
    hreflang: 0.25,      // 25%
    serverGeo: 0.15,     // 15%
    cultural: 0.20,      // 20%
    nap: 0.15,           // 15%
    performance: 0.15,   // 15%
    legal: 0.10          // 10%
  };
  
  return (
    data.hreflang.bidirectionalityScore * weights.hreflang +
    data.serverGeo.score * weights.serverGeo +
    data.cultural.score * weights.cultural +
    data.nap.consistencyScore * weights.nap +
    (data.regionalPerformance?.averageScore || 100) * weights.performance +
    data.legal.overallScore * weights.legal
  );
}
```

---

## 🔌 API-Integrationen

### Benötigte Services
1. **IP Geolocation**
   - ipinfo.io (Free: 50k requests/month)
   - MaxMind GeoIP2 (selbst gehostet)

2. **Performance Testing**
   - WebPageTest API (für Multi-Region Tests)
   - Lighthouse CI (selbst gehostet)

3. **Language Detection**
   - franc-min (offline, npm package)
   - Google Cloud Translation API (optional)

4. **CDN Detection**
   - whatismycdn.com API
   - Custom Header-Analysis

---

## 📝 CLI Integration

```bash
# Vollständiges Geo Audit
auditmysite https://example.com/sitemap.xml --geo-audit

# Nur bestimmte Regionen testen
auditmysite https://example.com/sitemap.xml --geo-audit --regions DE,FR,UK,US

# Mit Performance-Testing
auditmysite https://example.com/sitemap.xml --geo-audit --test-performance

# Geo Audit Report
auditmysite https://example.com/sitemap.xml --geo-audit --format html
```

### Neue CLI Optionen
```typescript
--geo-audit              // Enable Geo Audit analysis
--regions <codes>        // Test specific regions (comma-separated)
--test-performance       // Include multi-region performance tests
--hreflang-only         // Only check hreflang implementation
--legal-compliance      // Focus on legal compliance check
```

---

## 🎨 Report Visualisierung

### HTML Report Sections
1. **Executive Summary**
   - Geo Score (0-100)
   - Unterstützte Sprachen/Regionen
   - Kritische Probleme

2. **hreflang Matrix**
   - Interaktive Tabelle mit Bidirektionalität
   - Fehler-Highlighting

3. **Performance Map**
   - Weltkarte mit Ladezeiten
   - Color-coded Performance

4. **Compliance Dashboard**
   - Region-specific Checklists
   - DSGVO/CCPA Status

5. **Recommendations**
   - Priorisierte Handlungsempfehlungen
   - Quick Wins vs. Strategische Maßnahmen

---

## ✅ Definition of Done

- [ ] Alle 5 Haupt-Analyzer implementiert
- [ ] Types & Interfaces definiert
- [ ] Unit Tests (>80% Coverage)
- [ ] Integration Tests mit echten Sites
- [ ] CLI Integration
- [ ] HTML Report mit Visualisierungen
- [ ] Dokumentation (README, API Docs)
- [ ] Performance: <30s für Standard-Geo-Audit
- [ ] Beispiel-Reports erstellt

---

## 📚 Referenzen

### Standards & Guidelines
- [hreflang Best Practices (Google)](https://developers.google.com/search/docs/specialty/international/localized-versions)
- [ISO 639-1 Language Codes](https://en.wikipedia.org/wiki/List_of_ISO_639-1_codes)
- [ISO 3166-1 Country Codes](https://en.wikipedia.org/wiki/ISO_3166-1)
- [Schema.org LocalBusiness](https://schema.org/LocalBusiness)
- [GDPR Compliance Checklist](https://gdpr.eu/checklist/)

### Tools & Libraries
- `franc-min` - Language detection
- `ipinfo.io` - IP Geolocation
- `cheerio` - HTML parsing
- `fast-levenshtein` - String similarity

---

**Autor:** Jörn Seidel  
**Review:** Nach Phase 4 Completion
