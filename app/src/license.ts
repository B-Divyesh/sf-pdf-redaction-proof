const SLUG = "pdf-redaction-proof";
const KEY = `sb_license:${SLUG}`;
const VERDICT_KEY = `${KEY}:verdict`;
const ATTEMPT_KEY = `${KEY}:attempts`;
const BLOCKED_UNTIL_KEY = `${KEY}:blocked-until`;
const DAY = 86_400_000;
export const ATTEMPT_WINDOW_MS = 60_000;
export const ATTEMPT_LIMIT = 5;
const API = import.meta.env.VITE_BILLING_BASE_URL || "https://api.sociobot.in";

type CachedVerdict = { valid: boolean; checkedAt: number; token: string };

export class LicenseRateLimitError extends Error {
  readonly status = 429;
  constructor(readonly retryAfter: number) {
    super(`Too many license checks. Try again in ${retryAfter} seconds.`);
  }
}

export function nextAttemptWindow(attempts: number[], now = Date.now()): { attempts: number[]; retryAfter: number } {
  const recent = attempts.filter(value => Number.isFinite(value) && now - value < ATTEMPT_WINDOW_MS);
  if (recent.length >= ATTEMPT_LIMIT) {
    return { attempts: recent, retryAfter: Math.max(1, Math.ceil((recent[0] + ATTEMPT_WINDOW_MS - now) / 1000)) };
  }
  return { attempts: [...recent, now], retryAfter: 0 };
}

function reserveAttempt() {
  const now = Date.now();
  const blockedUntil = Number(localStorage.getItem(BLOCKED_UNTIL_KEY) || 0);
  if (blockedUntil > now) throw new LicenseRateLimitError(Math.max(1, Math.ceil((blockedUntil - now) / 1000)));
  let attempts: number[] = [];
  try { attempts = JSON.parse(localStorage.getItem(ATTEMPT_KEY) || "[]") as number[]; } catch { /* start a clean window */ }
  const next = nextAttemptWindow(attempts, now);
  localStorage.setItem(ATTEMPT_KEY, JSON.stringify(next.attempts));
  if (next.retryAfter) throw new LicenseRateLimitError(next.retryAfter);
}

function retryAfterSeconds(response: Response): number {
  const value = response.headers.get("Retry-After");
  const seconds = Number(value);
  if (Number.isFinite(seconds) && seconds > 0) return Math.ceil(seconds);
  if (value) {
    const date = Date.parse(value);
    if (Number.isFinite(date)) return Math.max(1, Math.ceil((date - Date.now()) / 1000));
  }
  return 60;
}

export function captureReturnedLicense(url = new URL(location.href)): string | null {
  const token = url.searchParams.get("license");
  if (!token) return localStorage.getItem(KEY);
  if (localStorage.getItem(KEY) !== token) localStorage.removeItem(VERDICT_KEY);
  localStorage.setItem(KEY, token);
  url.searchParams.delete("license");
  history.replaceState({}, "", `${url.pathname}${url.search}${url.hash}`);
  return token;
}

export function cachedUnlock(): boolean {
  const token = localStorage.getItem(KEY);
  if (!token) return false;
  try {
    const cached = JSON.parse(localStorage.getItem(VERDICT_KEY) || "null") as CachedVerdict | null;
    return cached?.valid === true && cached.token === token;
  } catch { return false; }
}

export async function verifyLicense(token: string, force = false): Promise<boolean> {
  const normalized = token.trim();
  if (localStorage.getItem(KEY) !== normalized) localStorage.removeItem(VERDICT_KEY);
  localStorage.setItem(KEY, normalized);
  try {
    const cached = JSON.parse(localStorage.getItem(VERDICT_KEY) || "null") as CachedVerdict | null;
    if (!force && cached?.token === normalized && Date.now() - cached.checkedAt < DAY) return cached.valid;
  } catch { /* reverify malformed cache */ }
  reserveAttempt();
  const response = await fetch(`${API}/api/v1/products/${SLUG}/verify?license=${encodeURIComponent(normalized)}`);
  if (response.status === 429) {
    const retryAfter = retryAfterSeconds(response);
    localStorage.setItem(BLOCKED_UNTIL_KEY, String(Date.now() + retryAfter * 1000));
    throw new LicenseRateLimitError(retryAfter);
  }
  if (!response.ok) throw new Error("The license service could not be reached.");
  const body = await response.json() as { valid: boolean };
  localStorage.setItem(VERDICT_KEY, JSON.stringify({ valid: body.valid, checkedAt: Date.now(), token: normalized }));
  return body.valid;
}

export function storedToken(): string | null { return localStorage.getItem(KEY); }
