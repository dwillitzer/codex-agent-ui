import assert from "node:assert/strict";
import crypto from "node:crypto";
import test from "node:test";
import {
  replacementAllowed,
  selectArtifact,
  verifyHelper,
  verifyIntegrity,
} from "../prepare-agent-ui-runtime.mjs";

test("package corruption is rejected before extraction", () => {
  const bytes = Buffer.from("known package");
  const sri = `sha512-${crypto.createHash("sha512").update(bytes).digest("base64")}`;
  verifyIntegrity(bytes, sri);
  assert.throws(
    () => verifyIntegrity(Buffer.from("changed package"), sri),
    /integrity mismatch/,
  );
  assert.throws(() => verifyIntegrity(bytes, "sha1-abc"), /SHA-512/);
});

test("extracted helper must match its independent pin", () => {
  const bytes = Buffer.from("known helper");
  const sha = crypto.createHash("sha256").update(bytes).digest("hex");
  assert.equal(verifyHelper(bytes, sha), sha);
  assert.throws(
    () => verifyHelper(Buffer.from("changed helper"), sha),
    /checksum mismatch/,
  );
});

test("unsupported platforms and unsafe download/member recipes are rejected", () => {
  const entry = {
    package: "@openai/codex",
    version: "0.170.0-darwin-x64",
    tarball:
      "https://registry.npmjs.org/@openai/codex/-/codex-0.170.0-darwin-x64.tgz",
    member: "package/vendor/target/bin/helper",
  };
  assert.equal(
    selectArtifact({ artifacts: { "darwin-x64": entry } }, "darwin", "x64"),
    entry,
  );
  assert.throws(
    () => selectArtifact({ artifacts: {} }, "linux", "arm64"),
    /No qualified runtime/,
  );
  for (const tarball of [
    "https://registry.npmjs.org/unrelated.tgz",
    "http://registry.npmjs.org/example.tgz",
    "https://example.org/runtime.tgz",
  ]) {
    assert.throws(
      () =>
        selectArtifact(
          { artifacts: { "darwin-x64": { ...entry, tarball } } },
          "darwin",
          "x64",
        ),
      /official npm registry/,
    );
  }
  for (const member of [
    "/absolute",
    "package/vendor/../escape",
    "package/vendor/target\\escape",
  ]) {
    assert.throws(
      () =>
        selectArtifact(
          { artifacts: { "darwin-x64": { ...entry, member } } },
          "darwin",
          "x64",
        ),
      /Unsafe runtime/,
    );
  }
});

test("regular files are protected and changing symlinks requires explicit replacement", () => {
  const file = { isSymbolicLink: () => false },
    link = { isSymbolicLink: () => true };
  replacementAllowed(undefined, false, false);
  replacementAllowed(file, true, false);
  assert.throws(() => replacementAllowed(file, false, true), /regular helper/);
  assert.throws(() => replacementAllowed(link, false, false), /--replace/);
  replacementAllowed(link, false, true);
});
