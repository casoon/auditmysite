#!/usr/bin/env python3
"""Labelling helper for the calibration of the heuristic signals (#784).

Procedure: docs/calibration.md. Subcommands:

  check     validate tests/fixtures/calibration/cases.json against the
            preregistered coverage requirements
  sheet     write a blind labelling sheet (HTML) and an empty CSV template
  finalize  import label CSVs into reference_set.json, freeze labels_sha256 in
            preregistration.json and run the calibration test

The sheet shows the URL and the preregistered questions only. It carries no
auditmysite output, no stratum, no split and no case notes. This script never
produces a label: every judgement comes from a label file.
"""
import argparse
import csv
import datetime
import hashlib
import html
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FIXTURES = ROOT / "tests/fixtures/calibration"
CASES = FIXTURES / "cases.json"
PREREG = FIXTURES / "preregistration.json"
REFSET = FIXTURES / "reference_set.json"
DOC = ROOT / "docs/calibration.md"
RESULTS_DIR = ROOT / "docs/calibration/results"
WORKDIR = ROOT / "reports/calibration"

# The preregistered questions, verbatim from docs/calibration.md ("check"
# fails if the two drift apart).
QUESTIONS = {
    "ux": "Does the page have clear structural usability problems: unclear or competing calls to action, no visual hierarchy, unclear content, missing trust information, or overload?",
    "journey": "Would a first-time visitor struggle to understand where they are, find their way, interact or complete the page's evident purpose?",
    "ai_visibility": "Is the content hard to extract for a machine — no usable structure, no self-contained passages, no structured data, or blocked for AI crawlers?",
    "source_quality": "Is the page poorly prepared as an information source — contradictory metadata, no identifiable publisher, no imprint/contact, no transport security?",
    "content_visibility": "Does the page lack the search-facing basics its purpose needs (title, description, headings, canonical, structured data where it applies)?",
}
SIGNALS = list(QUESTIONS)

JUDGEMENTS = {
    "ja": "problem", "j": "problem", "yes": "problem", "y": "problem", "problem": "problem",
    "nein": "no_problem", "n": "no_problem", "no": "no_problem", "no_problem": "no_problem",
    "unklar": "undecidable", "u": "undecidable", "unclear": "undecidable", "undecidable": "undecidable",
}
# Only used with --derive-ratings, and only where a rating is missing.
DERIVED_RATING = {"problem": 2, "undecidable": 3, "no_problem": 4}
CSV_COLUMNS = ["labeller", "case_id", "url", "signal", "judgement", "rating", "rationale", "labelled_at"]


def die(msg):
    sys.exit(f"error: {msg}")


def load_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def sheet_order(case_id):
    return hashlib.sha256(("sheet:" + case_id).encode()).hexdigest()


def opaque_id(case_id):
    """Id shown on the sheet and in the CSV: the readable slug could hint at the stratum."""
    return "c" + hashlib.sha256(("sheet-id:" + case_id).encode()).hexdigest()[:8]


def load_cases():
    return load_json(CASES)


# ─── check ──────────────────────────────────────────────────────────────────


def check_cases(cases, prereg):
    errors = []
    ids = [c["id"] for c in cases["cases"]]
    if len(ids) != len(set(ids)):
        errors.append("duplicate case ids")
    for c in cases["cases"]:
        if not re.match(r"^https?://", c["url"]):
            errors.append(f"{c['id']}: url is not http(s)")
        for s in c["signals"]:
            if s not in prereg["signals"]:
                errors.append(f"{c['id']}: signal {s} is not registered")
        if c["split"] not in ("dev", "holdout"):
            errors.append(f"{c['id']}: unknown split {c['split']}")
        if c["stratum"] not in ("positive", "negative", "borderline"):
            errors.append(f"{c['id']}: unknown stratum {c['stratum']}")
    for stratum in ("positive", "negative", "borderline"):
        members = [c["id"] for c in cases["cases"] if c["stratum"] == stratum]
        dev = sorted(members, key=lambda i: hashlib.sha256(i.encode()).hexdigest())[:2]
        for i in members:
            want = "dev" if i in dev else "holdout"
            have = next(c["split"] for c in cases["cases"] if c["id"] == i)
            if have != want:
                errors.append(f"{i}: split is {have}, split_rule gives {want}")
    req = prereg["requirements"]
    for signal in prereg["signals"]:
        hold = [c for c in cases["cases"] if c["split"] == "holdout" and signal in c["signals"]]
        n = lambda s: sum(c["stratum"] == s for c in hold)
        if len(hold) < req["min_holdout_cases_per_signal"]:
            errors.append(f"{signal}: {len(hold)} holdout cases < {req['min_holdout_cases_per_signal']}")
        for s, key in (("positive", "min_holdout_positive"), ("negative", "min_holdout_negative"), ("borderline", "min_holdout_borderline")):
            if n(s) < req[key]:
                errors.append(f"{signal}: {n(s)} holdout {s} < {req[key]}")
    return errors


def cmd_check(_args):
    cases, prereg = load_cases(), load_json(PREREG)
    errors = check_cases(cases, prereg)
    doc = DOC.read_text(encoding="utf-8")
    for s, q in QUESTIONS.items():
        if q not in doc:
            errors.append(f"question for {s} differs from docs/calibration.md")
    for e in errors:
        print("FAIL", e)
    cs = cases["cases"]
    hold = sum(c["split"] == "holdout" for c in cs)
    print(f"{len(cs)} cases ({hold} holdout, {len(cs) - hold} dev); labelling the holdout = {hold} pages x {len(SIGNALS)} questions per labeller")
    sys.exit(1 if errors else 0)


# ─── sheet ──────────────────────────────────────────────────────────────────

SHEET_CSS = """
:root{color-scheme:light dark;--bg:#fff;--fg:#1a1a1a;--mute:#555;--line:#c8c8c8;--acc:#0b5cad}
@media (prefers-color-scheme:dark){:root{--bg:#161616;--fg:#eee;--mute:#aaa;--line:#444;--acc:#7db7ff}}
body{margin:0 auto;max-width:56rem;padding:1rem 16px 5rem;background:var(--bg);color:var(--fg);font:16px/1.5 system-ui,sans-serif}
h1{font-size:1.4rem}h2{font-size:1.1rem;margin:0}
a{color:var(--acc)}
.case{border:1px solid var(--line);border-radius:8px;padding:1rem;margin:1.25rem 0}
.case.done{opacity:.7}
fieldset{border:0;border-top:1px solid var(--line);margin:.75rem 0 0;padding:.5rem 0 0}
legend{padding:0 .25rem 0 0;font-weight:600}
.q{margin:.1rem 0 .4rem;color:var(--mute)}
.opts label{margin-right:1.25rem;white-space:nowrap}
.extra{margin-top:.4rem;font-size:.9rem;color:var(--mute)}
.extra input[type=text]{width:min(30rem,100%);font:inherit}
.bar{position:fixed;left:0;right:0;bottom:0;background:var(--bg);border-top:1px solid var(--line);padding:.6rem 16px;display:flex;gap:1rem;align-items:center;flex-wrap:wrap}
button{font:inherit;padding:.4rem .9rem}
"""

SHEET_JS = """
const DATA=%s;
const STORE='calibration-sheet-v1';
let state={};try{state=JSON.parse(localStorage.getItem(STORE)||'{}')}catch(e){}
const $=s=>document.querySelector(s);
function save(){try{localStorage.setItem(STORE,JSON.stringify(state))}catch(e){}}
function count(){let n=0,t=0;for(const c of DATA.cases)for(const s of DATA.signals){t++;if(state[c.id+'|'+s]&&state[c.id+'|'+s].j)n++}
 $('#progress').textContent=n+' of '+t+' answered';}
function render(){
 const root=$('#cases');
 DATA.cases.forEach((c,i)=>{
  const sec=document.createElement('section');sec.className='case';
  const h=document.createElement('h2');h.textContent='Page '+(i+1)+' of '+DATA.cases.length;sec.append(h);
  const p=document.createElement('p');const a=document.createElement('a');a.href=c.url;a.target='_blank';a.rel='noopener';a.textContent=c.url;p.append(a);sec.append(p);
  for(const s of DATA.signals){
   const k=c.id+'|'+s;const v=state[k]||{};
   const f=document.createElement('fieldset');
   const l=document.createElement('legend');l.textContent=DATA.labels[s];f.append(l);
   const q=document.createElement('p');q.className='q';q.textContent=DATA.questions[s];f.append(q);
   const o=document.createElement('div');o.className='opts';
   for(const [val,txt] of [['problem','Yes (problem)'],['no_problem','No (no problem)'],['undecidable','Unclear']]){
    const lab=document.createElement('label');const r=document.createElement('input');r.type='radio';r.name=k;r.value=val;r.checked=v.j===val;
    r.onchange=()=>{state[k]=Object.assign(state[k]||{},{j:val,d:new Date().toISOString().slice(0,10)});save();count()};
    lab.append(r,' '+txt);o.append(lab)}
   f.append(o);
   const e=document.createElement('div');e.className='extra';
   const rl=document.createElement('label');rl.textContent='Rating 1 (severe problems) to 5 (none), optional: ';
   const rs=document.createElement('select');for(const x of ['','1','2','3','4','5']){const op=document.createElement('option');op.value=x;op.textContent=x||'-';rs.append(op)}
   rs.value=v.r||'';rs.onchange=()=>{state[k]=Object.assign(state[k]||{},{r:rs.value});save()};rl.append(rs);
   const ti=document.createElement('input');ti.type='text';ti.placeholder='One sentence why, optional';ti.setAttribute('aria-label','Rationale, optional');ti.value=v.t||'';
   ti.onchange=()=>{state[k]=Object.assign(state[k]||{},{t:ti.value});save()};
   e.append(rl,' ',ti);f.append(e);sec.append(f)}
  root.append(sec)});
 $('#name').value=state.__name||'';
 $('#name').onchange=()=>{state.__name=$('#name').value;save()};
 count();
}
function q(x){x=String(x==null?'':x);return /[",\\n;]/.test(x)?'"'+x.replace(/"/g,'""')+'"':x}
$('#dl').onclick=()=>{
 const name=$('#name').value.trim();if(!name){alert('Enter your name first.');return}
 const today=new Date().toISOString().slice(0,10);
 const rows=[%s];
 for(const c of DATA.cases)for(const s of DATA.signals){const v=state[c.id+'|'+s]||{};
  rows.push([name,c.id,c.url,s,v.j||'',v.r||'',v.t||'',v.j?(v.d||today):''].map(q).join(','))}
 const blob=new Blob([rows.join('\\n')+'\\n'],{type:'text/csv'});
 const a=document.createElement('a');a.href=URL.createObjectURL(blob);a.download='labels-'+name.replace(/[^A-Za-z0-9_-]+/g,'_')+'.csv';a.click();
};
render();
"""


def sheet_cases(cases, include_dev):
    sel = [c for c in cases["cases"] if include_dev or c["split"] == "holdout"]
    return sorted(sel, key=lambda c: sheet_order(c["id"]))


def cmd_sheet(args):
    cases = sheet_cases(load_cases(), args.include_dev)
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    # Only what a labeller may see: an opaque id, the URL and the questions.
    data = {
        "signals": SIGNALS,
        "questions": QUESTIONS,
        "labels": {s: s.replace("_", " ").capitalize() for s in SIGNALS},
        "cases": [{"id": opaque_id(c["id"]), "url": c["url"]} for c in cases],
    }
    header = json.dumps(",".join(CSV_COLUMNS))
    js = SHEET_JS % (json.dumps(data).replace("</", "<\\/"), header)
    page = f"""<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Calibration labelling sheet</title><style>{SHEET_CSS}</style></head>
<body><main>
<h1>Calibration labelling sheet</h1>
<p>Open each page, look at it as a visitor would, and answer each question with Yes, No or Unclear. Yes always means that the problem described in the question is present. Judge the page you see, not any report about it. Your answers are kept in this browser; download the CSV when you are done (you can come back and download again).</p>
<p><label>Your name: <input id="name" type="text" autocomplete="name"></label></p>
<div id="cases"></div>
</main>
<div class="bar"><span id="progress" role="status"></span><button id="dl" type="button">Download labels CSV</button></div>
<script>{js}</script></body></html>
"""
    (out / "sheet.html").write_text(page, encoding="utf-8")
    with open(out / "labels-template.csv", "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(CSV_COLUMNS)
        for c in cases:
            for s in SIGNALS:
                w.writerow([args.labeller or "", opaque_id(c["id"]), c["url"], s, "", "", "", ""])
    print(f"{len(cases)} pages x {len(SIGNALS)} questions")
    print(f"sheet:    {out / 'sheet.html'}")
    print(f"template: {out / 'labels-template.csv'}  (judgement: ja/nein/unklar)")


# ─── finalize ───────────────────────────────────────────────────────────────


def read_label_file(path, derive):
    text = Path(path).read_text(encoding="utf-8-sig")
    dialect = csv.Sniffer().sniff(text.splitlines()[0], delimiters=",;\t")
    rows = list(csv.DictReader(text.splitlines(), dialect=dialect))
    mtime = datetime.date.fromtimestamp(Path(path).stat().st_mtime).isoformat()
    names = {r["labeller"].strip() for r in rows if (r.get("judgement") or "").strip() and r["labeller"].strip()}
    if len(names) != 1:
        die(f"{path}: expected exactly one labeller name among answered rows, found {sorted(names) or 'none'}")
    labels, derived = [], 0
    for n, r in enumerate(rows, start=2):
        raw = (r.get("judgement") or "").strip().lower()
        if not raw:
            continue
        if raw not in JUDGEMENTS:
            die(f"{path}:{n}: judgement '{raw}' is not ja/nein/unklar")
        j = JUDGEMENTS[raw]
        rating = (r.get("rating") or "").strip()
        if rating:
            if not rating.isdigit() or not 1 <= int(rating) <= 5:
                die(f"{path}:{n}: rating '{rating}' is not 1-5")
            rating = int(rating)
        elif derive:
            rating, derived = DERIVED_RATING[j], derived + 1
        else:
            die(f"{path}:{n}: rating missing for {r['case_id']}/{r['signal']} (add it, or pass --derive-ratings)")
        labels.append({
            "case_id": r["case_id"].strip(), "signal": r["signal"].strip(), "labeller": names.copy().pop(),
            "labelled_at": (r.get("labelled_at") or "").strip() or mtime,
            "judgement": j, "rating": rating, "rationale": (r.get("rationale") or "").strip(),
        })
    return names.copy().pop(), labels, derived


def is_synthetic(name):
    return bool(re.match(r"(?i)^(dummy|synthetic|synth)", name))


def set_json_field(text, key, value):
    pattern = rf'("{key}":\s*)(null|"[^"]*")'
    new, n = re.subn(pattern, lambda m: m.group(1) + json.dumps(value), text, count=1)
    if n != 1:
        die(f"could not find \"{key}\" in preregistration")
    return new


def cmd_finalize(args):
    cases = load_json(args.cases)
    prereg = load_json(args.prereg_template)
    files = [read_label_file(p, args.derive_ratings) for p in args.labels]
    names = [f[0] for f in files]
    synthetic = [is_synthetic(n) for n in names]
    errors = check_cases(cases, prereg)
    if errors and not any(synthetic):
        die("cases do not meet the preregistered coverage:\n  " + "\n  ".join(errors))
    if errors:
        print("synthetic run, coverage not enforced:\n  " + "\n  ".join(errors))
    selected_by = args.selected_by or cases.get("selected_by")
    if not selected_by:
        die("--selected-by NAME is required (it confirms you reviewed the case list)")

    if len(set(names)) != len(names):
        die("two label files carry the same labeller name; labellers must be independent people")
    default_paths = args.out_set == REFSET and args.prereg == PREREG
    if any(synthetic) and not all(synthetic):
        die("synthetic and real labellers are mixed")
    if any(synthetic) and (not args.allow_synthetic or default_paths):
        die("synthetic labels may only be used with --allow-synthetic and --out-set/--prereg outside tests/fixtures/calibration")
    if not any(synthetic) and not default_paths:
        die("real labels go to the committed files; do not pass --out-set/--prereg")
    if not any(synthetic):
        for p, what in ((REFSET, "reference_set.json"), (PREREG, "preregistration.json")):
            if what == "reference_set.json" and load_json(p)["cases"]:
                die("reference_set.json already holds cases: a frozen set is immutable (docs/calibration.md)")
            if what == "preregistration.json" and load_json(p)["labels_sha256"]:
                die("labels_sha256 is already set: a frozen set is immutable")

    by_case = {c["id"]: c for c in cases["cases"]}
    by_case.update({opaque_id(i): c for i, c in list(by_case.items())})
    out_cases = []
    for c in cases["cases"]:
        labels = []
        for _name, ls, _d in files:
            for l in ls:
                if by_case.get(l["case_id"]) is c:
                    if l["signal"] not in c["signals"]:
                        die(f"label for unknown signal {l['signal']} on {c['id']}")
                    labels.append({k: l[k] for k in ("signal", "labeller", "labelled_at", "judgement", "rating", "rationale")})
        out_cases.append({
            "id": c["id"], "url": c["url"], "split": c["split"], "stratum": c["stratum"],
            "selected_by": selected_by, "selected_at": cases["selected_at"],
            "signals": c["signals"], "labels": labels,
        })
    for _n, ls, _d in files:
        for l in ls:
            if l["case_id"] not in by_case:
                die(f"label for unknown case {l['case_id']}")

    req = prereg["requirements"]
    missing = []
    for c in out_cases:
        if c["split"] != "holdout":
            continue
        for s in c["signals"]:
            have = sum(l["signal"] == s for l in c["labels"])
            if have < req["min_labellers_per_case"]:
                missing.append(f"{c['id']}/{s}: {have} of {req['min_labellers_per_case']} labels")
    if missing:
        die(f"{len(missing)} holdout answers missing:\n  " + "\n  ".join(missing[:25]) + ("\n  ..." if len(missing) > 25 else ""))

    ref = load_json(REFSET)
    ref["cases"] = out_cases
    body = json.dumps(ref, indent=2, ensure_ascii=False) + "\n"
    args.out_set.parent.mkdir(parents=True, exist_ok=True)
    args.out_set.write_bytes(body.encode("utf-8"))
    digest = hashlib.sha256(body.encode("utf-8")).hexdigest()

    text = args.prereg_template.read_text(encoding="utf-8")
    text = set_json_field(text, "labels_sha256", digest)
    if args.out_set != REFSET:
        text = set_json_field(text, "reference_set", str(args.out_set))
    args.prereg.parent.mkdir(parents=True, exist_ok=True)
    args.prereg.write_text(text, encoding="utf-8")
    derived = sum(f[2] for f in files)
    print(f"{args.out_set}: {len(out_cases)} cases, {sum(len(c['labels']) for c in out_cases)} labels from {', '.join(names)}")
    print(f"{args.prereg}: labels_sha256 = {digest}")
    if derived:
        print(f"note: {derived} ratings derived from the judgement (problem=2, unclear=3, no problem=4)")

    env = dict(os.environ)
    if args.prereg != PREREG:
        env["CALIBRATION_PREREGISTRATION"] = str(args.prereg)
    base = ["cargo", "test", "--test", "calibration_test"]
    if args.no_run:
        print("not run (--no-run). Next: " + " ".join(base + ["--", "--ignored", "--nocapture"]))
        return
    started = datetime.datetime.now().timestamp()
    for extra in ([], ["--", "--ignored", "--nocapture"]):
        rc = subprocess.run(base + extra, cwd=ROOT, env=env).returncode
        if extra == [] and rc != 0:
            sys.exit(rc)
    result = ROOT / "reports/calibration-results.json"
    if result.exists() and result.stat().st_mtime >= started and not any(synthetic):
        RESULTS_DIR.mkdir(parents=True, exist_ok=True)
        dest = RESULTS_DIR / f"{datetime.date.today().isoformat()}-calibration-results.json"
        shutil.copyfile(result, dest)
        print(f"results copied to {dest} - commit it whether the run passed or not")
    sys.exit(rc)


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = p.add_subparsers(dest="cmd", required=True)
    sub.add_parser("check", help="validate the case list").set_defaults(fn=cmd_check)
    s = sub.add_parser("sheet", help="write the blind labelling sheet and CSV template")
    s.add_argument("--out", default=str(WORKDIR))
    s.add_argument("--labeller", help="prefill the labeller column of the CSV template")
    s.add_argument("--include-dev", action="store_true", help="also label dev cases (not needed for the run)")
    s.set_defaults(fn=cmd_sheet)
    f = sub.add_parser("finalize", help="import labels, freeze labels_sha256, run the calibration test")
    f.add_argument("--labels", nargs="+", required=True, help="one CSV per labeller (at least two)")
    f.add_argument("--selected-by", help="your name; confirms you reviewed cases.json")
    f.add_argument("--derive-ratings", action="store_true", help="fill missing ratings from the judgement (problem=2, unclear=3, no problem=4)")
    f.add_argument("--no-run", action="store_true", help="import and freeze only")
    f.add_argument("--cases", type=Path, default=CASES)
    f.add_argument("--out-set", type=Path, default=REFSET)
    f.add_argument("--prereg", type=Path, default=PREREG)
    f.add_argument("--prereg-template", type=Path, default=PREREG)
    f.add_argument("--allow-synthetic", action="store_true", help="test mode for dummy labels, writes outside the committed files")
    f.set_defaults(fn=cmd_finalize)
    args = p.parse_args()
    args.fn(args)


if __name__ == "__main__":
    main()
