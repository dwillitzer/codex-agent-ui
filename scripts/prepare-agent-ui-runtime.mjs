#!/usr/bin/env node
import crypto from "node:crypto";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

export function verifyIntegrity(bytes, integrity) {
  if (!/^sha512-[A-Za-z0-9+/]+={0,2}$/.test(integrity)) {
    throw new Error("Expected a pinned SHA-512 package integrity value");
  }
  const actual = `sha512-${crypto.createHash("sha512").update(bytes).digest("base64")}`;
  if (actual !== integrity)
    throw new Error("Runtime package integrity mismatch");
}

export function verifyHelper(bytes, expected) {
  const actual = crypto.createHash("sha256").update(bytes).digest("hex");
  if (actual !== expected)
    throw new Error("Extracted helper checksum mismatch");
  return actual;
}

export function selectArtifact(manifest, platform, arch) {
  const artifact = manifest.artifacts[`${platform}-${arch}`];
  if (!artifact)
    throw new Error(`No qualified runtime recipe for ${platform}-${arch}`);
  const url = new URL(artifact.tarball);
  if (
    artifact.package !== "@openai/codex" ||
    !/^\d+\.\d+\.\d+(?:-[A-Za-z0-9.-]+)?$/.test(artifact.version) ||
    url.protocol !== "https:" ||
    url.hostname !== "registry.npmjs.org" ||
    decodeURIComponent(url.pathname) !==
      `/@openai/codex/-/codex-${artifact.version}.tgz`
  ) {
    throw new Error(
      "Runtime recipe must use the official npm registry over HTTPS",
    );
  }
  if (
    !artifact.member.startsWith("package/vendor/") ||
    artifact.member.split("/").some((part) => part === ".." || part === "") ||
    artifact.member.includes("\\")
  ) {
    throw new Error("Unsafe runtime archive member");
  }
  return artifact;
}

export function replacementAllowed(existing, sameHash, replace) {
  if (!existing || sameHash) return;
  if (!existing.isSymbolicLink())
    throw new Error("Refusing to overwrite a regular helper file");
  if (!replace)
    throw new Error(
      "A different helper symlink exists; pass --replace intentionally",
    );
}

export async function prepareRuntime({ targetDir, cacheDir, replace = false }) {
  const manifest = JSON.parse(
    await fs.readFile(
      path.join(root, "runtime/companion-manifest.json"),
      "utf8",
    ),
  );
  const artifact = selectArtifact(manifest, process.platform, process.arch);
  const cache = path.resolve(cacheDir, artifact.version);
  await fs.mkdir(cache, { recursive: true, mode: 0o700 });
  const archive = path.join(cache, "runtime.tgz");
  let bytes;
  try {
    bytes = await fs.readFile(archive);
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
    console.log(`Downloading ${artifact.package}@${artifact.version}`);
    const response = await fetch(artifact.tarball, {
      signal: AbortSignal.timeout(120_000),
    });
    if (!response.ok)
      throw new Error(`Runtime download returned HTTP ${response.status}`);
    bytes = Buffer.from(await response.arrayBuffer());
    verifyIntegrity(bytes, artifact.integrity);
    await fs.writeFile(archive, bytes, { mode: 0o600, flag: "wx" });
  }
  verifyIntegrity(bytes, artifact.integrity);
  // Stream only the pinned member to stdout: never unpack arbitrary paths into the filesystem.
  const helperBytes = execFileSync("tar", ["-xOzf", archive, artifact.member], {
    maxBuffer: 512 * 1024 * 1024,
  });
  const helperSha256 = verifyHelper(helperBytes, artifact.helperSha256);
  const helper = path.join(cache, "codex-code-mode-host");
  try {
    verifyHelper(await fs.readFile(helper), artifact.helperSha256);
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
    await fs.writeFile(helper, helperBytes, { mode: 0o755, flag: "wx" });
  }
  const help = execFileSync(helper, ["--help"], {
    encoding: "utf8",
    timeout: 45_000,
  });
  if (!help.includes("grpc+stdio://"))
    throw new Error("Helper lacks the required gRPC+stdio transport");
  const destination = path.join(
    path.resolve(targetDir),
    "codex-code-mode-host",
  );
  let existing;
  try {
    existing = await fs.lstat(destination);
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
  }
  let sameHash = false;
  if (existing) {
    try {
      sameHash =
        verifyHelper(await fs.readFile(destination), artifact.helperSha256) ===
        helperSha256;
    } catch (error) {
      if (
        error.code !== "ENOENT" &&
        !error.message.includes("checksum mismatch")
      )
        throw error;
    }
  }
  replacementAllowed(existing, sameHash, replace);
  if (!sameHash) {
    await fs.mkdir(path.dirname(destination), { recursive: true });
    if (existing) await fs.unlink(destination);
    await fs.symlink(helper, destination);
  }
  console.log(
    `Prepared ${artifact.package}@${artifact.version}: ${helperSha256}`,
  );
  return { destination, helperSha256, version: artifact.version };
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href
) {
  const options = {
    targetDir: path.join(root, "codex-rs/target/debug"),
    cacheDir: path.join(
      process.env.XDG_CACHE_HOME ||
        (process.platform === "darwin"
          ? path.join(os.homedir(), "Library/Caches")
          : path.join(os.homedir(), ".cache")),
      "codex-agent-ui/runtime",
    ),
    replace: false,
  };
  for (let i = 2; i < process.argv.length; i++) {
    const arg = process.argv[i];
    if (arg === "--target-dir" || arg === "--cache-dir") {
      const value = process.argv[++i];
      if (!value || value.startsWith("--"))
        throw new Error(`${arg} requires a path`);
      options[arg === "--target-dir" ? "targetDir" : "cacheDir"] = value;
    } else if (arg === "--replace") options.replace = true;
    else if (arg === "--help") {
      console.log(
        "node scripts/prepare-agent-ui-runtime.mjs [--target-dir PATH] [--cache-dir PATH] [--replace]",
      );
      process.exit(0);
    } else throw new Error(`Unknown option: ${arg}`);
  }
  await prepareRuntime(options);
}
