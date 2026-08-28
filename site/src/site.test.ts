import { describe, expect, it } from "vitest";
import { detectPlatform } from "./site";

describe("download platform detection", () => {
  it("detects macOS", () => expect(detectPlatform("Mozilla Macintosh")).toBe("macos"));
  it("detects Windows", () => expect(detectPlatform("Windows NT 10.0")).toBe("windows"));
  it("falls back to Linux", () => expect(detectPlatform("X11 Linux x86_64")).toBe("linux"));
});
