# AuditMySite - KRITISCHE Performance-Messung Fehler

**Vergleichsdatum:** 18. November 2025  
**Getestete Website:** https://www.aib-bauplanung.de/  
**AuditMySite Version:** 2.1.0  
**PageSpeed Insights:** Lighthouse 13.0.1

---

## 🚨 KERNPROBLEM: Völlig falsche Performance-Werte

### Dramatische Diskrepanz in Core Web Vitals

| Metrik | PageSpeed Insights | AuditMySite HTML-Report | Differenz | Status |
|--------|-------------------|------------------------|-----------|--------|
| **LCP** | **8.600ms** ❌ | **634ms** ✅ | **Faktor 13,6x** | 🔴 KRITISCH |
| **FCP** | **1.700ms** ⚠️ | **528ms** ✅ | **Faktor 3,2x** | 🔴 KRITISCH |
| **CLS** | **0.187** ⚠️ | **0.000** ✅ | **100% besser** | 🔴 KRITISCH |
| **Performance Score** | **63/100** ❌ | **100/100** ✅ | **+37 Punkte** | 🔴 KRITISCH |

**Das ist nicht nur eine Diskrepanz - die Werte sind völlig unrealistisch!**

---

## 🔴 KRITISCHER FEHLER #1: Network Throttling fehlt komplett

### Was AuditMySite im HTML-Report zeigt:
```
Measurement Settings: PSI-like Profile Enabled · CPU×4 · 
150ms, 1600kbps down, 750kbps up
```

### Was PageSpeed Insights verwendet:
```
Emulation eines Moto G Power with Lighthouse 13.0.1
Langsame 4G-Drosselung (400ms RTT, 400 Kbps download, 400 Kbps upload)
CPU-Drosselung: 4x slowdown
```

### ❌ PROBLEM IDENTIFIZIERT:

**AuditMySite:**
- Download: **1600 Kbps** (1,6 Mbps)
- Upload: **750 Kbps** (0,75 Mbps)
- RTT: **150ms**

**PageSpeed (Slow 4G):**
- Download: **400 Kbps** (0,4 Mbps) = **75% langsamer**
- Upload: **400 Kbps** (0,4 Mbps) = **47% langsamer**
- RTT: **400ms** = **167% höhere Latenz**

### 💡 WARUM IST DAS KRITISCH?

**Szenario:** E-Commerce-Website testet mit AuditMySite
```
AuditMySite sagt: "LCP: 634ms ✅ Perfekt!"
Realität (Slow 4G): LCP: 8600ms ❌ Katastrophal!

→ 90% der mobilen Nutzer brechen ab (>3s Ladezeit)
→ Geschäftsschaden durch falsche Sicherheit!
```

**TODO #1 - HÖCHSTE PRIORITÄT:**
- [ ] **Network Throttling auf PageSpeed-Standard anpassen**
  ```typescript
  // AKTUELL (FALSCH):
  const throttling = {
    rttMs: 150,
    downloadThroughputKbps: 1600,
    uploadThroughputKbps: 750
  };
  
  // SOLLTE SEIN (Lighthouse Slow 4G):
  const throttling = {
    rttMs: 400,              // +167%
    downloadThroughputKbps: 400,   // -75%
    uploadThroughputKbps: 400,     // -47%
    cpuSlowdownMultiplier: 4
  };
  ```
- [ ] **Datei finden:** Wo wird Throttling konfiguriert?
  - Suche nach: `1600kbps`, `750kbps`, `PSI-like`
  - Wahrscheinlich: `src/core/config/` oder `src/analyzers/performance/`
- [ ] **Lighthouse-Profile implementieren:**
  - Slow 4G (mobil, Standard)
  - Fast 3G (mobil, optimistisch)
  - Desktop (keine Throttling)
- [ ] **Konfigurierbar machen:**
  ```bash
  auditmysite <url> --throttling slow-4g  # Default
  auditmysite <url> --throttling fast-3g
  auditmysite <url> --throttling desktop
  ```

---

## 🔴 KRITISCHER FEHLER #2: Speed Index fehlt komplett

### Was PageSpeed zeigt:
```
Speed Index: 4,8s ⚠️
(Misst wie schnell Inhalt visuell geladen wird)
```

### Was AuditMySite zeigt:
```
❌ Speed Index: NICHT VORHANDEN
```

### 📊 WARUM IST SPEED INDEX WICHTIG?

Speed Index ist eine der **wichtigsten UX-Metriken**:
- Misst **wahrgenommene Ladegeschwindigkeit**
- Teil des Lighthouse Performance Score (10% Gewichtung)
- Bewertet **progressive Rendering**

**Beispiel:**
```
Website A: LCP 2s, aber alles auf einmal
→ Speed Index: 2000ms

Website B: LCP 2s, aber progressives Laden
→ Speed Index: 800ms (besser!)
```

**TODO #2 - HOHE PRIORITÄT:**
- [ ] **Speed Index implementieren**
  - Playwright hat `speedIndex` Metrik
  - Oder: Lighthouse Integration nutzen
- [ ] **Filmstrip-Captures verwenden:**
  ```typescript
  // Screenshots alle 100ms während Navigation
  const speedIndex = await calculateSpeedIndex(screenshots);
  ```
- [ ] **In Performance-Score integrieren (10% Gewichtung)**
- [ ] **Visueller Progress in HTML-Report:**
  - Filmstrip-Ansicht (wie PageSpeed)
  - Screenshot-Timeline

---

## 🔴 KRITISCHER FEHLER #3: LCP-Element-Identifikation fehlt

### Was PageSpeed zeigt:
```
LCP-Element: <img src="slider-image.jpg">
Ladezeit-Breakdown:
- TTFB: 200ms
- Resource load delay: 100ms  
- Resource load time: 8000ms
- Element render delay: 300ms
→ Gesamt: 8600ms

Optimierung: Bild preloaden, WebP verwenden
```

### Was AuditMySite zeigt:
```
Avg LCP: 634ms ✅
(Aber WELCHES Element? WARUM so langsam? WIE fixen?)
```

### 📊 WARUM IST DAS KRITISCH?

**Ohne Element-Identifikation:**
- ❌ Entwickler weiß nicht WAS optimiert werden muss
- ❌ Keine konkreten Handlungsempfehlungen
- ❌ Kann nicht priorisieren

**TODO #3 - HOHE PRIORITÄT:**
- [ ] **LCP-Element identifizieren:**
  ```typescript
  const lcpElement = await page.evaluate(() => {
    return new Promise((resolve) => {
      new PerformanceObserver((list) => {
        const entries = list.getEntries();
        const lastEntry = entries[entries.length - 1];
        resolve({
          element: lastEntry.element?.outerHTML,
          selector: getCSSPath(lastEntry.element),
          url: lastEntry.url,
          size: lastEntry.size,
          loadTime: lastEntry.loadTime,
          renderTime: lastEntry.renderTime
        });
      }).observe({ type: 'largest-contentful-paint', buffered: true });
    });
  });
  ```
- [ ] **Ladezeit-Breakdown berechnen:**
  - TTFB
  - Resource load delay
  - Resource load time
  - Element render delay
- [ ] **Screenshot des LCP-Elements:**
  - Highlight im Screenshot
  - Xpath/CSS-Selector anzeigen
- [ ] **Optimierungs-Empfehlungen basierend auf Element-Typ:**
  - Bild → Preload, WebP, CDN, Compression
  - Text → Critical CSS, Font optimization
  - Video → Poster image, Lazy loading

---

## 🔴 KRITISCHER FEHLER #4: CLS-Ursachen nicht identifiziert

### Was PageSpeed zeigt:
```
CLS: 0.187 ⚠️

Verursacher von Layout Shifts:
1. Slider ohne Height (Shift: 0.12)
2. Bilder ohne Dimensionen (Shift: 0.04)
3. Font-Swap FOUT (Shift: 0.027)
```

### Was AuditMySite zeigt:
```
Avg CLS: 0.000 ✅
(FALSCH! PageSpeed misst 0.187)
```

### 📊 WARUM IST CLS SO FALSCH?

**AuditMySite misst CLS = 0.000 bei ALLEN Seiten!**

**Mögliche Ursachen:**
1. **CLS wird zu früh gemessen** (vor dynamischem Content)
2. **Kein wait für Fonts/Bilder**
3. **Layout Shift Observer nicht korrekt**

**TODO #4 - HÖCHSTE PRIORITÄT:**
- [ ] **CLS-Messung debuggen:**
  ```typescript
  // Aktuell (vermutlich):
  await page.goto(url);
  const cls = await getCLS(); // Zu früh!
  
  // Sollte sein:
  await page.goto(url, { waitUntil: 'networkidle' });
  await page.waitForTimeout(5000); // Wait for late shifts
  const cls = await getCLS();
  ```
- [ ] **Layout Shift Observer korrekt implementieren:**
  ```typescript
  const cls = await page.evaluate(() => {
    return new Promise((resolve) => {
      let clsValue = 0;
      new PerformanceObserver((list) => {
        for (const entry of list.getEntries()) {
          if (!entry.hadRecentInput) {
            clsValue += entry.value;
          }
        }
      }).observe({ type: 'layout-shift', buffered: true });
      
      // Wait for page to stabilize
      setTimeout(() => resolve(clsValue), 5000);
    });
  });
  ```
- [ ] **Layout Shift Ursachen identifizieren:**
  - Welche Elemente shiften?
  - Screenshots vor/nach Shift
  - Fehlende width/height auf Bildern
  - Font-Swap-Events
  - Dynamisch injizierte Elemente (Ads, Embeds)

---

## 🟡 WICHTIGER FEHLER #5: Performance-Score wird nicht berechnet/ausgegeben

### Was PageSpeed zeigt:
```
Performance: 63/100 ❌ (Grade: D)

Gewichtung:
- LCP (8600ms): 25% → 0 Punkte
- FCP (1700ms): 10% → 75 Punkte  
- TBT (0ms): 30% → 100 Punkte
- CLS (0.187): 25% → 72 Punkte
- Speed Index (4800ms): 10% → 60 Punkte

→ Gesamt: 63/100
```

### Was AuditMySite zeigt (HTML-Report):
```
Average Score: 100/100 ✅
(FALSCH - basiert auf falschen Messwerten!)
```

### Was AuditMySite zeigt (CLI):
```
❌ KEIN Performance-Score!
Nur: 📱 Mobile Score: 93/100 (Grade: A)
```

**TODO #5 - HOHE PRIORITÄT:**
- [ ] **Lighthouse-kompatible Score-Berechnung:**
  ```typescript
  function calculatePerformanceScore(metrics) {
    const scores = {
      lcp: scoreLCP(metrics.lcp),      // 25%
      fcp: scoreFCP(metrics.fcp),      // 10%
      tbt: scoreTBT(metrics.tbt),      // 30%
      cls: scoreCLS(metrics.cls),      // 25%
      si: scoreSpeedIndex(metrics.si)   // 10%
    };
    
    return Math.round(
      scores.lcp * 0.25 +
      scores.fcp * 0.10 +
      scores.tbt * 0.30 +
      scores.cls * 0.25 +
      scores.si * 0.10
    );
  }
  
  // Lighthouse-Scoring-Kurven verwenden
  function scoreLCP(lcp) {
    if (lcp <= 1200) return 100;
    if (lcp >= 4000) return 0;
    // Log-normal curve zwischen 1200-4000
    return calculateLogNormalScore(lcp, 1200, 4000);
  }
  ```
- [ ] **Performance-Score in CLI ausgeben:**
  ```
  ⚡ Performance Analysis completed in 12.3s
  📊 Performance Score: 63/100 (Grade: D)
     
     Core Web Vitals:
     🎯 LCP: 8.6s ❌ Poor (Target: ≤2.5s, 343% over)
     🎯 FCP: 1.7s ⚠️ Needs Improvement (Target: ≤1.8s)
     🎯 TBT: 0ms ✅ Good
     🎯 CLS: 0.187 ⚠️ Needs Improvement (Target: ≤0.1, 87% over)
     🎯 SI: 4.8s ⚠️ Needs Improvement (Target: ≤3.4s)
  ```
- [ ] **Scoring-Thresholds dokumentieren:**
  - Good: LCP ≤2.5s, FCP ≤1.8s, CLS ≤0.1, TBT ≤200ms
  - Needs Improvement: LCP ≤4s, FCP ≤3s, CLS ≤0.25, TBT ≤600ms
  - Poor: Darüber

---

## 🟡 WICHTIGER FEHLER #6: Keine Performance-Optimierungs-Empfehlungen

### Was PageSpeed zeigt:

#### 6.1 Effiziente Cache-Nutzung
```
Geschätzte Einsparung: 3.331 KiB
44 Ressourcen ohne Cache-Control:
- IMG_20200526_111718_1-scaled.jpg (964 KiB)
- Blick-NW-e156….jpg (506 KiB)
- jquery.min.js (29 KiB)
[...41 weitere]

Empfehlung:
<FilesMatch "\.(jpg|jpeg|png)$">
  Header set Cache-Control "max-age=31536000"
</FilesMatch>
```

#### 6.2 Bildübermittlung verbessern
```
Geschätzte Einsparung: 1.117 KiB

Pro Bild:
- Screenshot-2025-11-13.png (373 KiB → 41 KiB WebP)
  → Einsparung: 332 KiB (89%)
- Blick-NW.jpg (506 KiB → 292 KiB WebP)
  → Einsparung: 214 KiB (42%)

Empfehlung:
<picture>
  <source srcset="image.avif" type="image/avif">
  <source srcset="image.webp" type="image/webp">
  <img src="image.jpg" alt="...">
</picture>
```

#### 6.3 Render-blocking Ressourcen
```
Geschätzte Zeitersparnis: 1.340ms

Blockierende Ressourcen:
- bootstrap.css (16,9 KiB, 900ms)
- style.css (16,1 KiB, 900ms)
- jquery.min.js (29,3 KiB, 750ms)

Empfehlung:
- Critical CSS inline einfügen
- Non-critical CSS async laden
- JavaScript mit defer laden
```

### Was AuditMySite zeigt:
```
❌ KEINE Optimierungs-Empfehlungen
❌ KEINE geschätzten Einsparungen
❌ KEINE priorisierten Maßnahmen
```

**TODO #6 - MITTLERE PRIORITÄT:**
- [ ] **Cache-Header-Analyse implementieren:**
  ```typescript
  const resources = await page.evaluate(() => 
    performance.getEntriesByType('resource')
  );
  
  for (const resource of resources) {
    const response = await fetch(resource.name, { method: 'HEAD' });
    const cacheControl = response.headers.get('cache-control');
    
    if (!cacheControl || cacheControl.includes('no-cache')) {
      issues.push({
        url: resource.name,
        size: resource.transferSize,
        type: resource.initiatorType,
        recommendation: 'Add cache-control header'
      });
    }
  }
  ```
- [ ] **Bild-Optimierungs-Potenzial berechnen:**
  ```typescript
  const images = await page.$$('img');
  
  for (const img of images) {
    const src = await img.getAttribute('src');
    const displaySize = await img.boundingBox();
    const naturalSize = await img.evaluate(el => ({
      width: el.naturalWidth,
      height: el.naturalHeight
    }));
    
    // Check oversized
    if (naturalSize.width > displaySize.width * 2) {
      savings.oversized += calculateSavings(naturalSize, displaySize);
    }
    
    // Check format
    if (!src.match(/\.(webp|avif)$/)) {
      const currentSize = await getFileSize(src);
      const webpSize = estimateWebPSize(currentSize);
      savings.format += (currentSize - webpSize);
    }
  }
  ```
- [ ] **Render-blocking-Ressourcen detektieren:**
  ```typescript
  const renderBlocking = await page.evaluate(() => {
    const blocking = [];
    
    // CSS in <head> ohne media query
    document.querySelectorAll('head link[rel=stylesheet]').forEach(link => {
      if (!link.media || link.media === 'all') {
        blocking.push({
          url: link.href,
          type: 'css',
          blocking: true
        });
      }
    });
    
    // JavaScript ohne async/defer
    document.querySelectorAll('script[src]').forEach(script => {
      if (!script.async && !script.defer) {
        blocking.push({
          url: script.src,
          type: 'js',
          blocking: true
        });
      }
    });
    
    return blocking;
  });
  ```
- [ ] **Priorisierte Empfehlungen generieren:**
  ```
  💡 Performance Opportunities (Total: 4.4 MB, 1.3s):
  
  🔴 CRITICAL (High Impact):
  1. Enable browser caching (3.3 MB saved)
     → 44 resources without Cache-Control
     → Add cache headers for static assets
  
  2. Optimize images (1.1 MB saved)
     → Convert 10 images to WebP/AVIF
     → Resize 3 oversized images
  
  🟡 IMPORTANT (Medium Impact):
  3. Eliminate render-blocking (1.3s saved)
     → Inline critical CSS (2 files)
     → Defer non-critical JavaScript (3 files)
  
  🟢 NICE TO HAVE (Low Impact):
  4. Remove unused CSS (44 KB saved)
     → 67% of bootstrap.css unused
  ```

---

## 📋 ZUSAMMENFASSUNG DER KERNPROBLEME

### 1. Falsche Testkonfiguration (KRITISCH)
- ✅ **Identifiziert:** Network Throttling zu schnell (1600 Kbps statt 400 Kbps)
- 🎯 **Impact:** LCP 634ms statt 8600ms = **Faktor 13,6x zu optimistisch**
- 🔧 **Fix:** Throttling auf Lighthouse Slow 4G Standard anpassen

### 2. Fehlende/Falsche Metriken (KRITISCH)
- ✅ **CLS komplett falsch:** 0.000 statt 0.187 = **100% Fehler**
- ✅ **Speed Index fehlt:** Keine Messung
- ✅ **LCP/FCP zu optimistisch:** Wegen falscher Throttling
- 🎯 **Impact:** Performance-Score 100/100 statt 63/100 = **+59% zu optimistisch**
- 🔧 **Fix:** CLS-Messung debuggen, Speed Index implementieren

### 3. Fehlende Diagnostik (HOCH)
- ✅ **LCP-Element unbekannt:** Welches Element ist LCP?
- ✅ **CLS-Ursachen unbekannt:** Welche Elemente shiften?
- ✅ **Keine Optimierungs-Empfehlungen:** Was soll gefixt werden?
- 🎯 **Impact:** Nutzer kann nicht optimieren
- 🔧 **Fix:** Element-Identifikation, Ursachen-Analyse, Empfehlungen

### 4. Fehlende CLI-Ausgabe (MITTEL)
- ✅ **Performance-Score nicht ausgegeben:** Nur im HTML-Report
- ✅ **Core Web Vitals nicht sichtbar:** Keine CLI-Ausgabe
- 🎯 **Impact:** Nutzer sieht nur "Mobile: 93/100" und denkt alles ist gut
- 🔧 **Fix:** CLI-Output erweitern

---

## 🎯 PRIORISIERTE UMSETZUNGSREIHENFOLGE

### 🔴 Sprint 0: HOTFIX (1-2 Tage) - DRINGEND!

**Ziel:** Falsche Messwerte korrigieren, bevor mehr Nutzer in falsche Sicherheit gewiegt werden!

**Datei finden:**
```bash
# Suche nach Throttling-Konfiguration
grep -r "1600" src/
grep -r "kbps" src/
grep -r "PSI-like" src/
grep -r "throttling" src/
```

**Tasks:**
- [ ] **#1.1: Network Throttling auf Lighthouse Standard setzen**
  - [ ] Datei finden (vermutlich `src/analyzers/performance/` oder `src/core/config/`)
  - [ ] Werte ändern:
    ```typescript
    // VON:
    rttMs: 150,
    downloadThroughputKbps: 1600,
    uploadThroughputKbps: 750
    
    // ZU:
    rttMs: 400,
    downloadThroughputKbps: 400,
    uploadThroughputKbps: 400,
    cpuSlowdownMultiplier: 4
    ```
  - [ ] Test mit www.aib-bauplanung.de
  - [ ] Erwartung: LCP sollte ~8-9s sein (näher an PageSpeed)

- [ ] **#1.2: CLS-Messung debuggen**
  - [ ] CLS-Code finden
  - [ ] Wait-Time hinzufügen (5s statt sofort)
  - [ ] Test mit www.aib-bauplanung.de
  - [ ] Erwartung: CLS sollte ~0.15-0.20 sein (näher an PageSpeed)

- [ ] **#1.3: Warning in CLI ausgeben**
  ```typescript
  console.log('⚠️  WARNING: Performance values may differ from PageSpeed Insights');
  console.log('   This is normal due to different test environments and timing.');
  console.log('   Both tools use Lighthouse methodology but with slight variations.');
  ```

**Deliverable:** 
- Realistische Performance-Werte
- Keine falschen Versprechungen mehr

---

### 🔴 Sprint 1: Performance-Score & CLI-Output (1 Woche)

**Tasks:**
- [ ] **#2.1: Performance-Score berechnen**
  - [ ] Lighthouse-Scoring-Kurven implementieren
  - [ ] Gewichtung: LCP 25%, FCP 10%, TBT 30%, CLS 25%, SI 10%
  - [ ] Unit-Tests gegen bekannte Werte

- [ ] **#2.2: Core Web Vitals in CLI ausgeben**
  ```
  ⚡ Performance: 63/100 (Grade: D)
     🎯 LCP: 8.6s ❌ (343% over target)
     🎯 FCP: 1.7s ⚠️
     🎯 CLS: 0.187 ⚠️
     🎯 TBT: 0ms ✅
  ```

- [ ] **#2.3: Mobile Usability vs. Performance trennen**
  ```
  📱 Mobile Usability: 93/100 (Grade: A)
  ⚡ Mobile Performance: 63/100 (Grade: D)
  ```

**Deliverable:**
- Performance-Score sichtbar in CLI
- Keine Verwechslung mehr zwischen Usability und Performance

---

### 🟡 Sprint 2: Speed Index & Element-Identifikation (1 Woche)

**Tasks:**
- [ ] **#3.1: Speed Index implementieren**
  - [ ] Playwright Speed Index Metrik nutzen
  - [ ] Oder: Lighthouse Integration
  - [ ] Filmstrip-Captures (Screenshots alle 100ms)

- [ ] **#3.2: LCP-Element identifizieren**
  - [ ] PerformanceObserver für LCP
  - [ ] Element-Selector extrahieren
  - [ ] Screenshot mit Highlight

- [ ] **#3.3: CLS-Ursachen identifizieren**
  - [ ] Layout-Shift-Events aufzeichnen
  - [ ] Betroffene Elemente loggen
  - [ ] Screenshots vor/nach Shift

**Deliverable:**
- Speed Index in Performance-Score
- Konkrete Element-Identifikation für LCP/CLS

---

### 🟢 Sprint 3: Optimierungs-Empfehlungen (1 Woche)

**Tasks:**
- [ ] **#4.1: Cache-Header-Analyse**
- [ ] **#4.2: Bild-Optimierungs-Potenzial**
- [ ] **#4.3: Render-blocking-Ressourcen**
- [ ] **#4.4: Ungenutztes CSS**
- [ ] **#4.5: Priorisierte Empfehlungen**

**Deliverable:**
- PageSpeed-ähnliche Optimierungs-Empfehlungen
- KB/ms Einsparungen pro Empfehlung

---

## 🧪 VALIDIERUNGS-CHECKLISTE

Nach Hotfix (Sprint 0):
- [ ] LCP: 634ms → ~8000ms (näher an PageSpeed 8600ms)
- [ ] CLS: 0.000 → ~0.15-0.20 (näher an PageSpeed 0.187)
- [ ] Performance-Score: 100/100 → ~60-70 (näher an PageSpeed 63)

Nach Sprint 1:
- [ ] CLI zeigt Performance-Score
- [ ] CLI zeigt Core Web Vitals mit Status (✅ ⚠️ ❌)
- [ ] Mobile Usability ≠ Mobile Performance

Nach Sprint 2:
- [ ] Speed Index vorhanden
- [ ] LCP-Element identifiziert (HTML-Snippet + Selector)
- [ ] CLS-Ursachen identifiziert (welche Elemente)

Nach Sprint 3:
- [ ] Optimierungs-Empfehlungen mit KB/ms-Einsparungen
- [ ] Priorisierung nach Impact (Critical/Important/Nice-to-have)
- [ ] Code-Snippets für Fixes

---

## 📊 ERWARTETE WERTE nach Hotfix

| Metrik | Aktuell (FALSCH) | Nach Hotfix | PageSpeed | Abweichung |
|--------|------------------|-------------|-----------|------------|
| LCP | 634ms ✅ | **~8000ms ❌** | 8600ms ❌ | ±7% ✅ |
| FCP | 528ms ✅ | **~1600ms ⚠️** | 1700ms ⚠️ | ±6% ✅ |
| CLS | 0.000 ✅ | **~0.17 ⚠️** | 0.187 ⚠️ | ±9% ✅ |
| Performance Score | 100/100 ✅ | **~65/100 ❌** | 63/100 ❌ | ±3% ✅ |

**Ziel:** ±10% Abweichung zu PageSpeed Insights ist akzeptabel (unterschiedliche Timing, Server-Antwortzeiten, etc.)

---

## 🔬 DEBUG-ANLEITUNG

### Wo ist der Performance-Code?

**Vermutete Dateien:**
```
src/analyzers/performance/
  ├── performance-analyzer.ts      # Hauptklasse
  ├── core-web-vitals.ts           # LCP, FCP, CLS, INP Messung
  ├── metrics-collector.ts         # Metriken sammeln
  └── config/
      └── throttling.ts            # 🎯 HIER: 1600kbps, 750kbps
```

**Debug-Schritte:**
1. Suche nach `1600`:
   ```bash
   grep -rn "1600" src/
   ```

2. Suche nach "PSI-like":
   ```bash
   grep -rn "PSI-like" src/
   ```

3. Suche nach Throttling-Config:
   ```bash
   grep -rn "downloadThroughputKbps\|rttMs" src/
   ```

4. Playwright DeviceDescriptor prüfen:
   ```bash
   grep -rn "deviceDescriptors\|emulate" src/
   ```

5. CLS-Messung finden:
   ```bash
   grep -rn "layout-shift\|CLS\|cumulativeLayoutShift" src/
   ```

### Wie testet man den Fix?

```bash
# 1. Code ändern
# 2. Neu bauen
npm run build

# 3. Testen
./bin/audit.js https://www.aib-bauplanung.de --max-pages 1

# 4. Erwartung in CLI (nach Fix):
#    ⚡ Performance: 60-70/100
#    🎯 LCP: ~8000ms ❌
#    🎯 CLS: ~0.17 ⚠️

# 5. HTML-Report prüfen
open reports/www.aib-bauplanung.de/accessibility-report-*.html
```

---

**Priorität:** 🔴 KRITISCH  
**Erstellt:** 18.11.2025  
**Owner:** Development Team  
**Status:** HOTFIX REQUIRED  
**Nächste Schritte:** Code finden → Throttling korrigieren → Validieren
