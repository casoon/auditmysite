// Rule pages are generated, not written: `cargo test --test rule_docs` (with
// UPDATE_RULE_DOCS=1) writes site/src/data/rules/ from the rule catalogue, the
// rule contracts and the detection corpus (tests/common/rule_docs.rs). The URL
// scheme is `rules/<page id>/`, documented in src/taxonomy/rule_docs.rs and
// src/audit/rule_docs.rs.

export type Outcome = 'automatic' | 'hint-only' | 'manual';
export type Domain = 'wcag' | 'security' | 'schema' | 'libraries';

export interface Example {
  case: string;
  file: string;
  verdict: 'violation' | 'needs_review' | 'pass' | 'absent';
  selector?: string;
  response_headers?: Record<string, string>;
  html?: string;
}

export interface Check {
  contract_id: string;
  label?: string;
  outcome: Outcome;
  source: string;
  applicability: string;
  expectation: string;
  method: string;
  capabilities: string[];
  page_level: boolean;
  manual_reason?: string;
  failing?: Example;
  passing?: Example;
}

export interface RulePage {
  id: string;
  url: string;
  domain: Domain;
  title: string;
  outcome: Outcome;
  area: string;
  criterion?: string;
  level?: string;
  axe_id?: string;
  severity?: string;
  user_impact?: string;
  technical_impact?: string;
  checks: Check[];
}

export interface RuleIndex {
  base_url: string;
  counts: {
    domain: Domain;
    label: string;
    automatic: number;
    hint_only: number;
    manual: number;
    total: number;
  }[];
  total: number;
  rules: {
    id: string;
    title: string;
    domain: Domain;
    outcome: Outcome;
    criterion?: string;
    level?: string;
  }[];
}

const files = import.meta.glob<RulePage | RuleIndex>('../data/rules/**/*.json', {
  eager: true,
  import: 'default',
});

export const ruleIndex = files['../data/rules/index.json'] as RuleIndex;

export const rulePages: RulePage[] = Object.entries(files)
  .filter(([path]) => path !== '../data/rules/index.json')
  .map(([, page]) => page as RulePage);

/** What each outcome class means; the same wording on the landing page and on every rule page. */
export const outcomes: Record<Outcome, { label: string; meaning: string }> = {
  automatic: {
    label: 'Automatic',
    meaning: 'An automated check that can report a confirmed violation.',
  },
  'hint-only': {
    label: 'Hint only',
    meaning:
      'An automated check that by design reports review hints only: its findings are marked for manual review, never as confirmed violations.',
  },
  manual: {
    label: 'Manual',
    meaning:
      'A tool cannot decide this check. The audit only points a person at it; it is never counted as automatically verified.',
  },
};

export const repoUrl = 'https://github.com/casoon/auditmysite';
