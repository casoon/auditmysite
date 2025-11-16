# 🚀 Quick Start: Projekt-Verbesserungen

**Sofort starten mit diesen Schritten!**

---

## ✅ Phase 1: Tooling Setup (30 Minuten)

### 1. Dependencies installieren

```bash
cd /Users/jseidel/GitHub/auditmysite

# ESLint + TypeScript
pnpm add -D @typescript-eslint/parser @typescript-eslint/eslint-plugin

# Prettier
pnpm add -D prettier eslint-config-prettier eslint-plugin-prettier

# Husky + Lint-Staged (Pre-commit Hooks)
pnpm add -D husky lint-staged

# Type Coverage Tool
pnpm add -D type-coverage

# Testing Tools
pnpm add -D @types/jest jest-extended
```

### 2. Husky initialisieren

```bash
# Husky Setup
pnpm exec husky install
pnpm pkg set scripts.prepare="husky install"

# Pre-commit Hook erstellen
pnpm exec husky add .husky/pre-commit "pnpm exec lint-staged"
```

### 3. Lint-Staged konfigurieren

Füge zu `package.json` hinzu:

```json
{
  "lint-staged": {
    "*.ts": [
      "eslint --fix",
      "prettier --write"
    ]
  }
}
```

### 4. Erste Checks ausführen

```bash
# Type Check
pnpm run type-check

# Linting (wird viele Fehler zeigen - das ist OK!)
pnpm run lint 2>&1 | head -50

# Formatting
pnpm run format:check
```

---

## 🎯 Phase 2: Type Safety (2-3 Stunden)

### Schritt 1: Types-Modul aufräumen

**Priorität: HOCH**

```bash
# Finde alle `any` in types/
grep -r ": any" src/types/

# Fix: Ersetze durch spezifische Typen
```

**Beispiel-Fix:**

```typescript
// ❌ Vorher: src/types/audit-data.ts
export interface AuditData {
  results: any[];
  metadata: any;
}

// ✅ Nachher
export interface AuditResult {
  url: string;
  score: number;
  issues: Issue[];
}

export interface AuditMetadata {
  timestamp: Date;
  version: string;
  config: AuditConfig;
}

export interface AuditData {
  results: AuditResult[];
  metadata: AuditMetadata;
}
```

### Schritt 2: Core-Module Type Safety

**Dateien in dieser Reihenfolge:**

1. `src/core/queue/types.ts` - Queue-Typen
2. `src/core/browser/browser-pool-manager.ts` - Browser-Pool
3. `src/core/accessibility/accessibility-checker.ts` - Main Checker

**Command:**

```bash
# Pro Datei: Finde und fixe `any`
grep -n ": any" src/core/queue/types.ts

# Teste nach jedem Fix
pnpm run type-check
```

### Schritt 3: Strict Mode aktivieren (schrittweise!)

```json
// tsconfig.json - Step by Step!
{
  "compilerOptions": {
    "strict": true,
    "noImplicitAny": true,              // ← Start hier
    "strictNullChecks": false,          // ← Später
    "strictFunctionTypes": true,
    "noUncheckedIndexedAccess": false   // ← Zuletzt
  }
}
```

**Vorgehen:**
1. `noImplicitAny: true` → Alle `any` fixen
2. `strictNullChecks: true` → Null-Checks hinzufügen
3. `noUncheckedIndexedAccess: true` → Array-Zugriffe absichern

---

## 🧪 Phase 3: Testing Foundation (2 Stunden)

### Test-Coverage messen

```bash
# Coverage Report erstellen
pnpm run test:coverage

# Coverage-Report öffnen
open coverage/lcov-report/index.html
```

### Kritische Tests schreiben

**Priorität 1: AccessibilityChecker**

```typescript
// tests/unit/core/accessibility-checker.test.ts
import { AccessibilityChecker } from '@core/accessibility';
import { BrowserPoolManager } from '@core/browser/browser-pool-manager';

describe('AccessibilityChecker', () => {
  let checker: AccessibilityChecker;
  let poolManager: BrowserPoolManager;

  beforeEach(() => {
    poolManager = new BrowserPoolManager({ maxConcurrent: 1 });
    checker = new AccessibilityChecker({ poolManager });
  });

  afterEach(async () => {
    await poolManager.cleanup();
  });

  describe('testPage', () => {
    it('should test accessible page successfully', async () => {
      const result = await checker.testPage('https://www.w3.org/WAI/');
      
      expect(result.url).toBe('https://www.w3.org/WAI/');
      expect(result.accessibilityResult).toBeDefined();
      expect(result.duration).toBeGreaterThan(0);
    });

    it('should handle HTTP errors gracefully', async () => {
      const result = await checker.testPage('https://httpstat.us/404');
      
      expect(result.accessibilityResult.crashed).toBe(true);
      expect(result.accessibilityResult.errors).toContain('HTTP 404 error');
    });
  });
});
```

**Priorität 2: Queue System**

```typescript
// tests/unit/core/queue/queue.test.ts
import { Queue } from '@core/queue';

describe('Queue', () => {
  it('should process items in parallel', async () => {
    const queue = Queue.forAccessibilityTesting('parallel', {
      maxConcurrent: 3
    });

    const items = ['url1', 'url2', 'url3'];
    const processor = jest.fn(async (url) => ({ url, result: 'ok' }));

    const result = await queue.processWithProgress(items, processor);

    expect(result.completed).toHaveLength(3);
    expect(processor).toHaveBeenCalledTimes(3);
  });
});
```

### Test-Scripts nutzen

```bash
# Unit Tests
pnpm run test:unit

# Integration Tests
pnpm run test:integration

# Watch Mode (während Entwicklung)
pnpm run test:watch
```

---

## 🔍 Phase 4: Code Quality Monitoring (1 Stunde)

### ESLint Baseline erstellen

```bash
# Aktuellen Stand als Baseline speichern
pnpm run lint > eslint-baseline.txt

# Anzahl Fehler zählen
wc -l eslint-baseline.txt
```

### Type Coverage verfolgen

```bash
# Type Coverage messen
pnpm exec type-coverage

# Ziel: > 90% Type Coverage
```

### Metriken Dashboard

```bash
# Erstelle metrics.sh
cat > scripts/metrics.sh << 'EOF'
#!/bin/bash
echo "📊 Code Quality Metrics"
echo "======================="
echo ""
echo "TypeScript Files:"
find src -name "*.ts" | wc -l
echo ""
echo "Lines of Code:"
find src -name "*.ts" -exec wc -l {} + | tail -1
echo ""
echo "Test Files:"
find tests -name "*.test.ts" | wc -l
echo ""
echo "Type Coverage:"
pnpm exec type-coverage --detail | tail -5
EOF

chmod +x scripts/metrics.sh

# Ausführen
./scripts/metrics.sh
```

---

## 🎨 Phase 5: Formatter + Linter (30 Minuten)

### Alle Dateien formatieren

```bash
# Erstmal nur checken
pnpm run format:check

# Dann formatieren
pnpm run format

# Commit
git add -A
git commit -m "chore: format all files with prettier"
```

### Linting schrittweise

```bash
# Module für Module
pnpm exec eslint src/types/ --fix
pnpm exec eslint src/core/queue/ --fix
pnpm exec eslint src/core/browser/ --fix

# Commit nach jedem Modul
git add src/types/
git commit -m "fix(types): resolve eslint issues"
```

---

## 📋 Wöchentliche Checkliste

### Woche 1: Foundation
- [x] Tooling installiert
- [x] ESLint + Prettier konfiguriert
- [x] Pre-commit Hooks aktiv
- [ ] Type Coverage Baseline: ____%
- [ ] `src/types/` komplett typsicher
- [ ] 5+ Unit Tests geschrieben

### Woche 2: Core Modules
- [ ] `src/core/queue/` typsicher
- [ ] `src/core/browser/` typsicher
- [ ] `src/core/accessibility/` typsicher
- [ ] Queue-Tests: 80%+ Coverage
- [ ] Browser-Pool-Tests: 70%+ Coverage

### Woche 3: Analyzers
- [ ] `src/analyzers/` aufgeräumt
- [ ] Duplizierte Queue-Systeme identifiziert
- [ ] Migration-Plan erstellt
- [ ] Integration Tests: 10+ geschrieben

### Woche 4: Quality Gate
- [ ] ESLint: 0 Errors
- [ ] Type Coverage: >85%
- [ ] Test Coverage: >60%
- [ ] CI/CD Pipeline grün

---

## 🐛 Häufige Probleme

### Problem: `noImplicitAny` zeigt zu viele Fehler

**Lösung:** Schrittweise pro Modul aktivieren

```json
// tsconfig.types.json (nur für types/)
{
  "extends": "./tsconfig.json",
  "compilerOptions": {
    "noImplicitAny": true
  },
  "include": ["src/types/**/*"]
}
```

```bash
# Nur types/ prüfen
pnpm exec tsc -p tsconfig.types.json
```

### Problem: Tests brechen nach Type-Fixes

**Lösung:** Test-Mocks aktualisieren

```typescript
// ❌ Vorher
const mockConfig: any = { maxPages: 5 };

// ✅ Nachher
const mockConfig: AccessibilityCheckerConfig = {
  poolManager: mockPoolManager,
  logger: mockLogger,
  enableComprehensiveAnalysis: false,
  analyzerTypes: []
};
```

### Problem: ESLint zu langsam

**Lösung:** Cache aktivieren

```bash
# .eslintrc.json
{
  "cache": true,
  "cacheLocation": ".eslintcache"
}

# .gitignore
echo ".eslintcache" >> .gitignore
```

---

## 📊 Erfolgs-Tracking

### Baseline (Heute)
- TypeScript Files: 127
- Lines of Code: ~40.500
- Test Files: 17
- Type Coverage: ??% (messen!)
- ESLint Errors: ??

### Ziel (4 Wochen)
- Type Coverage: >90%
- Test Coverage: >70%
- ESLint Errors: 0
- `any` in Core: 0

### Wöchentliches Update

```bash
# Erstelle weekly-report.sh
cat > scripts/weekly-report.sh << 'EOF'
#!/bin/bash
date >> progress.log
echo "Type Coverage:" >> progress.log
pnpm exec type-coverage --detail >> progress.log
echo "" >> progress.log
echo "Test Coverage:" >> progress.log
pnpm run test:coverage 2>&1 | grep "All files" >> progress.log
echo "---" >> progress.log
EOF

chmod +x scripts/weekly-report.sh

# Jede Woche ausführen
./scripts/weekly-report.sh
```

---

## 🎯 Quick Wins (< 1 Stunde)

### 1. Low-Hanging Fruit Typen

```bash
# Finde einfache Fixes
grep -r "function.*any" src/ | grep -v test

# Oft sind das Utility-Functions
# Beispiel: formatDate(date: any) → formatDate(date: Date)
```

### 2. Console.log entfernen

```bash
# Finde alle console.log
grep -r "console.log" src/

# Ersetze durch Logger
sed -i '' 's/console.log/logger.debug/g' src/path/to/file.ts
```

### 3. TODOs dokumentieren

```bash
# Alle TODOs sammeln
grep -r "TODO\|FIXME" src/ > todos.txt

# In Issues konvertieren
# Oder in ANALYSIS.md aufnehmen
```

---

## 🚀 Los geht's!

```bash
# 1. Dependencies installieren
pnpm install

# 2. Baseline messen
pnpm run type-check 2>&1 | tee baseline-typecheck.txt
pnpm run lint 2>&1 | tee baseline-lint.txt

# 3. Ersten Fix machen
# Wähle eine Datei mit wenig `any`
grep -c ": any" src/types/*.ts | sort -t: -k2 -n | head -5

# 4. Fix, Test, Commit
# Kleiner Zyklus: Fix → Test → Commit → Push

# 5. Erfolg feiern! 🎉
```

---

**Viel Erfolg! Bei Fragen: ANALYSIS.md oder WARP.md konsultieren.**
