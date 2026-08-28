const SLUG = "pdf-redaction-proof";
const KEY = `sb_license:${SLUG}`;
const VERDICT_KEY = `${KEY}:verdict`;
const DAY = 86_400_000;
const API = import.meta.env.VITE_BILLING_BASE_URL || "https://api.sociobot.in";

type CachedVerdict = { valid: boolean; checkedAt: number };

export function captureReturnedLicense(url = new URL(location.href)): string | null {
  const token = url.searchParams.get("license");
  if (!token) return localStorage.getItem(KEY);
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
    return cached?.valid === true;
  } catch { return false; }
}

export async function verifyLicense(token: string, force = false): Promise<boolean> {
  localStorage.setItem(KEY, token.trim());
  try {
    const cached = JSON.parse(localStorage.getItem(VERDICT_KEY) || "null") as CachedVerdict | null;
    if (!force && cached && Date.now() - cached.checkedAt < DAY) return cached.valid;
  } catch { /* reverify malformed cache */ }
  const response = await fetch(`${API}/api/v1/products/${SLUG}/verify?license=${encodeURIComponent(token.trim())}`);
  if (!response.ok) throw new Error("The license service could not be reached.");
  const body = await response.json() as { valid: boolean };
  localStorage.setItem(VERDICT_KEY, JSON.stringify({ valid: body.valid, checkedAt: Date.now() }));
  return body.valid;
}

export function storedToken(): string | null { return localStorage.getItem(KEY); }
