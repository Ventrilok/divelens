import { describe, expect, it, vi } from "vitest";
import type { UpdatePhase, UpdateStatus } from "./types";

vi.stubGlobal("navigator", { language: "fr-FR" });
const u = await import("./updates");

const status = (phase: UpdatePhase, more: Partial<UpdateStatus> = {}): UpdateStatus => ({
  phase,
  installable: true,
  lastCheck: null,
  dismissed: null,
  notesUrl: null,
  releasesUrl: "https://github.com/Ventrilok/divelens/releases",
  ...more,
});

// Moments in this computer's time zone (the tests run in UTC+14).
const at = (d: number, h: number, m: number) => new Date(2026, 10, d, h, m);
const seconds = (date: Date) => date.getTime() / 1000;

describe("when updates were last looked for", () => {
  it("says today, in this computer's time", () => {
    expect(u.checkedWhen(seconds(at(15, 9, 12)), at(15, 18, 0))).toBe("aujourd'hui à 09:12");
  });
  it("gives the day otherwise", () => {
    expect(u.checkedWhen(seconds(at(14, 23, 50)), at(15, 0, 10))).toBe("le 14 novembre 2026 à 23:50");
  });
});

describe("the line under the version", () => {
  const now = at(15, 18, 0);
  it("says up to date with the last check", () => {
    expect(u.stateLine(status({ state: "upToDate" }, { lastCheck: seconds(at(15, 9, 12)) }), now)).toBe(
      "À jour · dernière vérification aujourd'hui à 09:12",
    );
  });
  it("says when nothing was looked for yet", () => {
    expect(u.stateLine(status({ state: "idle" }), now)).toBe("Pas encore vérifié");
    expect(u.stateLine(status({ state: "idle" }, { lastCheck: seconds(at(15, 9, 12)) }), now)).toBe(
      "Dernière vérification aujourd'hui à 09:12",
    );
  });
  it("says it is looking", () => {
    expect(u.stateLine(status({ state: "checking" }), now)).toBe("Vérification…");
  });
});

describe("the notice above the screens", () => {
  it("announces the version found", () => {
    expect(u.noticeVersion(status({ state: "available", version: "2.1.0" }))).toBe("2.1.0");
  });
  it("stays closed for that version, and comes back for a newer one", () => {
    expect(u.noticeVersion(status({ state: "available", version: "2.1.0" }, { dismissed: "2.1.0" }))).toBeNull();
    expect(u.noticeVersion(status({ state: "available", version: "2.2.0" }, { dismissed: "2.1.0" }))).toBe("2.2.0");
  });
  it("shows nothing otherwise", () => {
    expect(u.noticeVersion(null)).toBeNull();
    expect(u.noticeVersion(status({ state: "upToDate" }))).toBeNull();
    expect(u.noticeVersion(status({ state: "downloading", version: "2.1.0", percent: 40 }))).toBeNull();
  });
});

describe("one thing at a time", () => {
  it("is busy while checking, downloading or installing", () => {
    expect(u.isBusy(status({ state: "checking" }))).toBe(true);
    expect(u.isBusy(status({ state: "downloading", version: "2.1.0", percent: null }))).toBe(true);
    expect(u.isBusy(status({ state: "installing", version: "2.1.0" }))).toBe(true);
    expect(u.isBusy(status({ state: "available", version: "2.1.0" }))).toBe(false);
    expect(u.isBusy(status({ state: "failed", reason: "network", version: null }))).toBe(false);
  });
  it("explains each failure", () => {
    expect(u.failureText("signature")).toBe("Le fichier téléchargé n'est pas signé par Dive Lens : il n'a pas été installé.");
  });
});
