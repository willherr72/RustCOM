export interface MatchRange {
  start: number;
  end: number; // exclusive
}

export interface SearchResult {
  matches: MatchRange[];
  invalid: string | null;
}

export function findMatches(text: string, pattern: string, useRegex: boolean): SearchResult {
  if (!pattern) return { matches: [], invalid: null };
  let re: RegExp;
  try {
    if (useRegex) {
      re = new RegExp(pattern, "g");
    } else {
      re = new RegExp(escapeRegex(pattern), "gi");
    }
  } catch (e) {
    return { matches: [], invalid: (e as Error).message };
  }
  const out: MatchRange[] = [];
  let m: RegExpExecArray | null;
  while ((m = re.exec(text)) !== null) {
    if (m.index === re.lastIndex) re.lastIndex += 1; // guard against zero-width loops
    out.push({ start: m.index, end: m.index + m[0].length });
  }
  return { matches: out, invalid: null };
}

function escapeRegex(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
