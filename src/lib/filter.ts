export interface FilterResult {
  text: string;
  regex: RegExp | null;
  invalid: string | null;
}

export function applyFilter(text: string, pattern: string, enabled: boolean): FilterResult {
  if (!enabled || !pattern) return { text, regex: null, invalid: null };
  let re: RegExp;
  try {
    re = new RegExp(pattern);
  } catch (e) {
    return { text, regex: null, invalid: (e as Error).message };
  }
  const lines = text.split("\n");
  const kept: string[] = [];
  for (const line of lines) {
    if (re.test(line)) kept.push(line);
  }
  return { text: kept.join("\n"), regex: re, invalid: null };
}
