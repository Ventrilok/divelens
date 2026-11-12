import { describe, expect, it } from "vitest";
import { errorKind } from "./sourceText";

describe("errorKind", () => {
  it("keeps Subsurface's own errors for Subsurface only", () => {
    expect(errorKind("subsurface", { kind: "severalLogs", detail: "" })).toBe("severalLogs");
    expect(errorKind("subsurface", { kind: "oldFormat", detail: "" })).toBe("oldFormat");
    expect(errorKind("macDive", { kind: "oldFormat", detail: "" })).toBe("database");
  });

  it("reads other reader errors as a database error", () => {
    expect(errorKind("macDive", { kind: "notFound", detail: "" })).toBe("notFound");
    expect(errorKind("subsurface", { kind: "notADiveLog", detail: "" })).toBe("notADiveLog");
    expect(errorKind("subsurface", { kind: "badProfile", detail: "" })).toBe("database");
  });
});
