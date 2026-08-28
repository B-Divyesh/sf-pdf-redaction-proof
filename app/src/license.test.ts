import { describe, expect, it } from "vitest";
import { ATTEMPT_LIMIT, LicenseRateLimitError, nextAttemptWindow } from "./license";

describe("license verification response policy", () => {
  it("returns a 429-shaped error with Retry-After after five attempts per minute", () => {
    const now = 1_000_000;
    const attempts = Array.from({ length: ATTEMPT_LIMIT }, (_, index) => now - 5_000 + index);
    const result = nextAttemptWindow(attempts, now);
    const error = new LicenseRateLimitError(result.retryAfter);
    expect(error.status).toBe(429);
    expect(error.retryAfter).toBe(55);
    expect(result.attempts).toHaveLength(ATTEMPT_LIMIT);
  });

  it("opens a new request slot after the rolling minute", () => {
    expect(nextAttemptWindow([1_000, 2_000], 62_000)).toEqual({ attempts: [62_000], retryAfter: 0 });
  });
});
