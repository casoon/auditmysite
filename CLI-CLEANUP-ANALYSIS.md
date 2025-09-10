# CLI Parameter Cleanup Analysis - AuditMySite

## 🎯 Aktueller Status: 43+ Parameter - viel zu viel!

### ❌ KÖNNEN WEG - Veraltete/Doppelte Parameter (15):

#### Tauri Integration (nicht mehr relevant)
- `--stream` - Tauri Streaming Modus
- `--session-id <id>` - Session Tracking 
- `--chunk-size <size>` - Chunk Size für Reports

#### Experimentelle Features (nicht stabil)
- `--unified-queue` - Experimentelles Queue System
- `generateSeoReport: false` - Schon hardcoded auf false
- `generateSecurityReport: false` - Schon hardcoded auf false
- `lighthouse: false` - Schon hardcoded auf false

#### Veraltete Screenshot/Testing Features
- `captureScreenshots` - Selten genutzt, komplex
- `testKeyboardNavigation` - Experimentell, nicht stabil
- `testColorContrast` - Experimentell 
- `testFocusManagement` - Experimentell

#### Browser-spezifische Features (zu nischig)
- `chrome135Features` - Versionsspezifisch, wird obsolet
- `modernHtml5` - Sollte Standard sein
- `ariaAdvanced` - Sollte Standard sein  
- `semanticAnalysis` - Sollte Standard sein

### ✅ MÜSSEN BLEIBEN - Essentiell (8):

#### Core Functionality
1. `<sitemapUrl>` - **REQUIRED** - Basis Parameter
2. `--max-pages <number>` - **ESSENTIAL** - Wichtigste Kontrolle
3. `--format <type>` - **ESSENTIAL** - HTML vs Markdown
4. `--output-dir <dir>` - **USEFUL** - Output Kontrolle

#### User Experience  
5. `--expert` - **USEFUL** - Erweiterte Konfiguration
6. `--non-interactive` - **ESSENTIAL** - CI/CD Support  
7. `--verbose` - **USEFUL** - Debug Information

#### Backward Compatibility
8. `--full` - **LEGACY** - Kann durch `--max-pages 1000` ersetzt werden

### ⚠️ DISKUSSION - Performance Budget (6):
- `--budget <template>` - **NÜTZLICH** - Templates sind praktisch
- `--lcp-budget <ms>` - **NISCHIG** - Nur für Experten
- `--cls-budget <score>` - **NISCHIG**
- `--fcp-budget <ms>` - **NISCHIG**  
- `--inp-budget <ms>` - **NISCHIG**
- `--ttfb-budget <ms>` - **NISCHIG**

**Empfehlung:** Nur `--budget <template>` behalten, individuelle Budgets über Expert Mode.

### ⭐ DISKUSSION - Analysis Toggles (4):
- `--no-performance` - **SINNVOLL** - Performance kann Overhead haben
- `--no-seo` - **SINNVOLL** - SEO nicht immer relevant
- `--no-content-weight` - **WENIGER WICHTIG** - Lightweight Analysis
- `--no-mobile` - **SINNVOLL** - Mobile Test kann Zeit kosten

**Empfehlung:** Die ersten 3 behalten, `--no-mobile` diskutieren.

## 🎯 EMPFOHLENE NEUE CLI (11 Parameter statt 43+):

```bash
auditmysite <sitemapUrl>
  --max-pages <number>      # Statt --full
  --format <type>           # html|markdown  
  --output-dir <dir>
  --expert                  # Interaktiver Modus
  --non-interactive         # CI/CD Modus
  --verbose                 
  --budget <template>       # Nur Templates: default|ecommerce|blog|corporate
  --no-performance         # Opt-out statt Opt-in
  --no-seo                 # Opt-out statt Opt-in  
  --no-content-weight      # Opt-out statt Opt-in
```

## 🔧 Expert Mode - Alle Details verfügbar

Der Expert Mode (`--expert`) würde alle erweiterten Optionen abfragen:
- Individuelle Performance Budgets
- Spezielle Testing Features  
- Browser-spezifische Einstellungen
- Screenshot und Keyboard Testing
- Detaillierte ARIA Analyse

## 🚀 Migration Strategy

### Phase 1: Deprecation Warnings
```bash
# Zeige Warnung für veraltete Parameter
if (options.stream) {
  console.warn('⚠️  --stream is deprecated and will be removed in v2.1');
}
```

### Phase 2: Removal (v2.1)
- Alle experimentellen Features entfernen
- Tauri Integration Code entfernen  
- Testing Features in separate Module auslagern

### Phase 3: Simplification (v2.2)
- Expert Mode als primäre Konfigurationsmethode
- Weniger CLI Parameter
- Bessere UX durch geführte Konfiguration

## 📊 Impact Analysis

### Aktuell: 43+ Parameter
- **Verwirrend** für neue Nutzer
- **Maintenance Overhead** 
- **Dokumentation** sehr komplex
- **Testing** aller Kombinationen unmöglich

### Vorschlag: 11 Parameter  
- **Einfach** zu verstehen
- **Schnell** zu dokumentieren
- **Expert Mode** für Power Users
- **CI/CD** weiterhin gut unterstützt

## 💡 Zusätzliche Vereinfachungen

### Entfernen aus dem Code:
1. **Streaming Funktionen** (`runStreamingAudit`)
2. **Lighthouse Integration** (bereits disabled)
3. **Screenshot Funktionen** (zu komplex)
4. **Experimentelle Testing Features**
5. **Browser-spezifische Checks**

### Vereinfachen:
1. **Performance Budgets** → Nur Templates
2. **Analysis Features** → Alle standardmäßig aktiv, nur Opt-out
3. **Expert Mode** → Alle erweiterten Features dort
4. **Defaults** → Vernünftige Standards ohne Konfiguration

## 🎯 Fazit

**Von 43+ auf 11 Parameter = 75% Reduktion**

Das würde AuditMySite **viel einfacher** machen:
- ✅ Neue Nutzer können sofort loslegen
- ✅ Power Users haben Expert Mode  
- ✅ CI/CD bleibt voll funktional
- ✅ Wartung wird deutlich einfacher
- ✅ Dokumentation wird übersichtlich
