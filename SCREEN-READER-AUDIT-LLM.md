# Screen Reader Audit: LLM-Assisted Quality Analysis

`auditmysite` produces a structured JSON file during a screen reader audit containing the complete reading flow a screen reader traverses on the page. The tool's rule-based analysis detects structural violations (empty links, missing names, WCAG criteria). It **cannot** judge whether the reading flow is comprehensible, coherent, and navigable as an experience.

This workflow describes how to prepare the JSON output for a qualitative LLM analysis pass.

## What the JSON contains

| Field | Content |
|---|---|
| `reading_sequence` | All nodes in the order a screen reader encounters them — with role, name, and announcement text |
| `navigation_views.headings` | Heading hierarchy with quality ratings |
| `navigation_views.landmarks` | Present ARIA landmarks |
| `navigation_views.links` | All links, grouped and rated |
| `issues` | Rule-based violations detected automatically |
| `bfsg_compliance` | BFSG obligations and affected nodes |

## What an LLM can add

Rule-based tools check whether a field is present or empty. An LLM can assess:

- Does the reading flow **make sense** as text? Can a listener understand the page content and purpose without visual context?
- Are there **redundancies** — the same text repeated consecutively due to nested roles?
- Are **navigation anchors** (landmarks, headings, tab stops) distributed usefully, or are there long stretches without orientation?
- Are link texts **unambiguous in context** — given the surrounding heading and paragraph?
- Does the sequence of announcements feel **natural** or mechanical?

## Workflow

### 1. Extract the reading flow

The `announcement` field of each entry in `reading_sequence` contains exactly what a screen reader announces. Extract this as a continuous text:

```bash
cat <file>.json | python3 -c "
import json, sys
data = json.load(sys.stdin)
seq = data.get('reading_sequence', [])
for e in seq:
    ann = e.get('announcement', '')
    if ann and ann != '(kein Name), generic' and ann != '(kein Name), InlineTextBox':
        tab = ' [TAB-STOP]' if e.get('tab_stop') else ''
        print(f'{ann}{tab}')
" > transcript.txt
```

This produces a cleaned reading flow without purely structural wrapper nodes.

### 2. Send to an LLM

Send the contents of `transcript.txt` together with the following system prompt to an LLM:

```
You are an accessibility expert specialising in screen reader usability.

The text below is the exact reading flow that NVDA or VoiceOver announces on a web page — top to bottom, in DOM order. Lines marked [TAB-STOP] are keyboard-focusable elements.

Assess this reading flow on:
1. Comprehensibility — Can a listener understand the page content and purpose from this flow alone?
2. Redundancy — Are there passages where the same content is announced unnecessarily more than once?
3. Navigation — Are landmarks, headings, and tab stops distributed well enough for structured navigation?
4. Context gaps — Are there links or buttons whose meaning is unclear without visual context?
5. Technical artefacts — Are there icon characters, cryptic strings, or empty announcements?

For each category, quote the specific text from the flow that supports your finding.
```

### 3. Add landmark and heading context

For a more precise assessment, prepend a `navigation_views` summary before the transcript:

```bash
cat <file>.json | python3 -c "
import json, sys
data = json.load(sys.stdin)
nav = data.get('navigation_views', {})

print('## Landmarks')
for l in nav.get('landmarks', []):
    print(f'- {l[\"role\"]}: {l.get(\"name\") or \"(no name)\"}')

print()
print('## Heading hierarchy')
for h in nav.get('headings', []):
    indent = '  ' * (h['level'] - 1)
    print(f'{indent}H{h[\"level\"]}: {h[\"text\"]} [{h[\"quality\"]}]')

print()
print('## Links')
for l in nav.get('links', []):
    q = l.get('quality', '?')
    text = l.get('text') or '(empty)'
    count = l.get('count', 1)
    print(f'- [{q}] {repr(text)} ({count}x)')
"
```

Prepend this block before the transcript. The LLM can then directly relate gaps in the landmark structure to the reading flow.

### 4. Interpreting results

LLM findings are **qualitative, not normative** — they supplement the rule-based `issues` in the JSON, they do not replace them. Practical guidance:

- Findings with a concrete quote from the transcript → actionable, use directly
- Findings without a text reference → treat as a hint, verify manually
- Conflicts with rule-based `issues` → always prefer the rule (verifiable); treat the LLM assessment as context only

## Limitations

- LLMs occasionally hallucinate problems not present in the transcript — always cross-check against the original
- The transcript reflects the static page state only (no JavaScript interaction flow)
- Dynamic content (modals, tabs, live regions) is not captured in the snapshot
- Language models have no actual screen reader experience — the assessment is an analogy, not a test

## Related documents

- `WCAG-MANUAL-CHECKS.md` — manual checks no tool can replace
