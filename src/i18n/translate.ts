import zh from './zh-TW.json';
export type Language = 'en' | 'zh-TW';
export const LANGUAGE_KEY = 'trace.ui-language.v1';
const catalog: Record<string, string> = zh;
export function translateEvidence(language:Language,e:{kind:string},text:string):string {
  return ['commit','hypothesis','incident','source_line','stack_frame'].includes(e.kind)?text:translate(language,text);
}
export function initialLanguage(storage: Pick<Storage, 'getItem'>, browserLanguage: string): Language {
  try { const saved = storage.getItem(LANGUAGE_KEY); if (saved === 'en' || saved === 'zh-TW') return saved; } catch { /* Preferences can be unavailable in private/restricted contexts. */ }
  return browserLanguage.toLowerCase().startsWith('zh') ? 'zh-TW' : 'en';
}
export function translate(language: Language, text: string, ...values: unknown[]): string {
  let result = text;
  if (language === 'zh-TW') {
    const key = text.trim().replace(/\s+/g, ' ');
    if (Object.hasOwn(catalog, key)) result = text.match(/^\s*/)?.[0] + catalog[key] + text.match(/\s*$/)?.[0];
    else result = dynamicText(text);
  }
  return result.replace(/\{(\d+)\}/g, (placeholder, n: string) => Number(n) < values.length ? String(values[Number(n)]) : placeholder);
}
function dynamicText(text: string): string {
  const patterns: Array<[RegExp, (...parts: string[]) => string]> = [
    [/^Case: (.*)$/s, (_, title) => `案件：${title}`],
    [/^(\d+) evidence items · (\d+) connections$/, (_, n, edges) => `${n} 項證據 · ${edges} 條連線`],
    [/^The selected test first fails at commit (\w+) within the selected good\/bad range\. This verifies a regression boundary for the selected test predicate\.$/, (_, sha) => `所選測試首次在提交 ${sha} 失敗，範圍限定於所選的正常／失敗版本。此結果驗證了所選測試判定條件的回歸邊界。`],
    [/^Selected test first fails at (\w+) within the approved range$/, (_, sha) => `所選測試首次在 ${sha} 失敗，範圍限定於已批准的版本範圍`],
    [/^(PASSED|FAILED|TIMEOUT|CANCELLED|LAUNCH_ERROR|OUTPUT_LIMIT) · (.*)$/, (_, state, sha) => `${translate('zh-TW', state.toLowerCase())} · ${sha}`],
    [/^(\w+) · (\d+) executions · selected test predicate$/, (_, state, n) => `${stateLabels[state] ?? state} · ${n} 次執行 · 所選測試判定條件`],
    [/^(\d+) recent commits$/, (_, n) => `${n} 次最近提交`],
    [/^(\d+) revisions evaluated$/, (_, n) => `${n} 個版本已評估`],
    [/^Frame (\d+): (.*)$/s, (_, n, detail) => `第 ${n} 個堆疊框架：${translate('zh-TW',detail)}`],
    [/^Stack source does not exist: (.*)$/s, (_, path) => `堆疊來源不存在：${path}`],
    [/^Test executable unavailable outside the source repository: (.*?)\. Install it or select its absolute path; no dependencies are installed automatically\.$/s, (_, name) => `來源儲存庫外找不到測試執行程式：${name}。請安裝或選擇其絕對路徑；不會自動安裝依賴。`],
    [/^SyntaxError: (.*)$/s, (_, detail) => `JSON 格式無效：${detail}`],
    [/^Repository folder unavailable: (.*)$/s, (_, detail) => `無法開啟儲存庫資料夾：${detail}`],
    [/^VERIFICATION INCONCLUSIVE: (.*)$/s, (_, detail) => `驗證無法判定：${detail}`],
    [/^INVALID GOOD BOUNDARY: (.*)$/s, (_, detail) => `正常版本邊界無效：${detail}`],
    [/^INVALID BAD BOUNDARY: (.*)$/s, (_, detail) => `失敗版本邊界無效：${detail}`],
    [/^INTERRUPTED VERIFICATION\. Execution will not resume automatically\. Inspect retained steps and clean up the owned worktree\.$/, () => '驗證已中斷。不會自動續跑。請檢視保留的步驟並清理 TRACE 工作樹。'],
  ];
  for (const [pattern, format] of patterns) { const match = text.match(pattern); if (match) return format(...match); }
  return text;
}
const stateLabels: Record<string, string> = {awaiting_approval:'等待批准',running:'執行中',cleaning_up:'清理中',bad_confirmed:'已重現測試失敗',good_confirmed:'正常版本通過',bisect_confirmed:'二分驗證已確認',inconclusive:'無法判定',failed:'驗證失敗',cancelled:'已取消',interrupted:'已中斷',discarded:'已捨棄',integrity_error:'證據無效'};
