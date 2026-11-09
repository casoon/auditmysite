// Rule pages are generated, not written: `cargo test --test rule_docs` (with
// UPDATE_RULE_DOCS=1) writes site/src/data/rules/ from the rule catalogue, the
// rule contracts and the detection corpus (tests/common/rule_docs.rs). The URL
// scheme is `rules/<rule_id>/`, documented in src/taxonomy/rule_docs.rs.

export type Outcome = 'automatic' | 'hint-only' | 'manual';

export interface Example {
  case: string;
  file: string;
  verdict: 'violation' | 'needs_review' | 'pass' | 'absent';
  selector?: string;
  html: string;
}

export interface RulePage {
  id: string;
  url: string;
  title: string;
  outcome: Outcome;
  criterion?: string;
  level?: string;
  axe_id?: string;
  dimension: string;
  subcategory: string;
  issue_class: string;
  severity: string;
  user_impact: string;
  technical_impact: string;
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

export interface RuleIndex {
  base_url: string;
  counts: { automatic: number; hint_only: number; manual: number; total: number };
  rules: { id: string; title: string; outcome: Outcome; criterion?: string; level?: string }[];
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
    meaning:
      'The detection corpus pins at least one page on which the rule reports a confirmed violation.',
  },
  'hint-only': {
    label: 'Hint only',
    meaning:
      'The rule reports review hints: its findings are marked for manual review, and no corpus page makes it fail.',
  },
  manual: {
    label: 'Manual',
    meaning:
      'A tool cannot decide this check. The audit only points a person at it; it is never counted as automatically verified.',
  },
};

export const repoUrl = 'https://github.com/casoon/auditmysite';
