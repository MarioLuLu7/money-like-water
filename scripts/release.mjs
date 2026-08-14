import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync, copyFileSync } from "node:fs";
import { dirname, extname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(scriptDir, "..");
const packageJsonPath = join(repoRoot, "package.json");
const readmePath = join(repoRoot, "README.md");
const tauriConfigPath = join(repoRoot, "src-tauri", "tauri.conf.json");
const cargoTomlPath = join(repoRoot, "src-tauri", "Cargo.toml");
const bundleRoot = join(repoRoot, "src-tauri", "target", "release", "bundle");
const localUpdaterKeyPath = join(repoRoot, ".tauri", "updater.key");
const localUpdaterKeyPasswordPath = join(repoRoot, ".tauri", "updater.key.password");

const options = parseArgs(process.argv.slice(2));
let githubCliPath = "";

main();

function main() {
  try {
    process.chdir(repoRoot);

    if (options.version) {
      step(`Update app version to ${options.version}`, () => updateReleaseVersion(options.version));
    }

    const resolvedVersion = options.version || getAppVersion();
    const tagName = `v${resolvedVersion}`;
    const releaseDir = join(repoRoot, "releases", tagName);
    const shouldPublishGitHubRelease = !options.noGitHubRelease;
    ensureUpdaterSigningKey();

    if (shouldPublishGitHubRelease) {
      step("Check GitHub CLI", () => {
        githubCliPath = ensureGitHubCli();
        ensureGitHubAuth(githubCliPath);
        console.log(`Using GitHub CLI: ${githubCliPath}`);
      });
    }

    if (!options.skipInstall) {
      step("Install dependencies with npm ci", () => run("npm", ["ci"]));
    }

    step("Build frontend", () => run("npm", ["run", "build"]));
    step("Clean previous installer bundle output", () => {
      rmSync(bundleRoot, { recursive: true, force: true });
      rmSync(releaseDir, { recursive: true, force: true });
    });
    step("Build Tauri installer bundle", () => run("npm", ["run", "tauri", "--", "build"]));

    const artifacts = getReleaseArtifacts();
    if (artifacts.length === 0) {
      throw new Error(`No installer artifacts were found under ${bundleRoot}.`);
    }

    step("Sign updater installer artifacts", () => {
      for (const artifact of artifacts) {
        signUpdaterArtifact(artifact.path);
      }
    });

    step("Collect installer artifacts", () => {
      mkdirSync(releaseDir, { recursive: true });
      for (const artifact of artifacts) {
        const destination = join(releaseDir, artifact.name);
        copyFileSync(artifact.path, destination);
        console.log(`Copied ${artifact.name}`);

        if (existsSync(`${artifact.path}.sig`)) {
          copyFileSync(`${artifact.path}.sig`, `${destination}.sig`);
          console.log(`Copied ${artifact.name}.sig`);
        }
      }

      const manifest = createUpdaterManifest({
        releaseDir,
        artifacts,
        tagName,
        version: resolvedVersion,
      });
      writeFileSync(join(releaseDir, "latest.json"), `${JSON.stringify(manifest, null, 2)}\n`, "utf8");

      const checksums = listFiles(releaseDir).map((file) => {
        const hash = createHash("sha256").update(readFileSync(file.path)).digest("hex");
        return `${hash}  ${file.name}`;
      });
      writeFileSync(join(releaseDir, "SHA256SUMS.txt"), `${checksums.join("\n")}\n`, "ascii");
    });

    if (!options.skipGit && !options.branch) {
      options.branch = getCurrentBranch();
    }

    if (!options.skipGit && !options.noCommit) {
      step("Commit release changes", () => {
        run("git", ["add", "-A"]);
        if (gitStatusShort()) {
          run("git", ["commit", "-m", options.message || `release: ${tagName}`]);
        } else {
          console.log("No git changes to commit.");
        }
      });
    }

    if (shouldPublishGitHubRelease) {
      step(`Create or update git tag ${tagName}`, () => {
        if (gitTagExists(tagName)) {
          run("git", ["tag", "-f", tagName]);
        } else {
          run("git", ["tag", tagName]);
        }
      });
    }

    if (!options.skipGit && !options.noPush) {
      step(`Push code to ${options.remote}/${options.branch}`, () => {
        run("git", ["push", options.remote, options.branch]);
        if (shouldPublishGitHubRelease) {
          run("git", ["push", options.remote, tagName, "--force"]);
        }
      });
    }

    if (shouldPublishGitHubRelease) {
      step(`Publish GitHub release ${tagName}`, () => {
        const releaseFiles = listFiles(releaseDir).map((file) => file.path);
        if (githubReleaseExists(githubCliPath, tagName)) {
          run(githubCliPath, ["release", "upload", tagName, ...releaseFiles, "--clobber"]);
        } else {
          const releaseArgs = ["release", "create", tagName, ...releaseFiles, "--title", tagName, "--notes", `Release ${tagName}`];
          if (options.draft) releaseArgs.push("--draft");
          if (options.prerelease) releaseArgs.push("--prerelease");
          run(githubCliPath, releaseArgs);
        }
      });
    }

    console.log("");
    console.log(`Release package is ready: ${releaseDir}`);
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

function parseArgs(args) {
  const parsed = {
    version: "",
    message: "",
    remote: "origin",
    branch: "",
    skipInstall: false,
    skipGit: false,
    noCommit: false,
    noPush: false,
    noGitHubRelease: false,
    noAutoInstallGitHubCli: false,
    noInteractiveGitHubLogin: false,
    draft: false,
    prerelease: false,
  };

  const aliases = {
    version: "version",
    v: "version",
    message: "message",
    m: "message",
    remote: "remote",
    branch: "branch",
    skipinstall: "skipInstall",
    skipgit: "skipGit",
    nocommit: "noCommit",
    nopush: "noPush",
    nogithubrelease: "noGitHubRelease",
    noautoinstallgithubcli: "noAutoInstallGitHubCli",
    nointeractivegithublogin: "noInteractiveGitHubLogin",
    draft: "draft",
    prerelease: "prerelease",
  };
  const valueOptions = new Set(["version", "message", "remote", "branch"]);

  for (let i = 0; i < args.length; i += 1) {
    const arg = args[i];
    if (!arg.startsWith("-")) {
      throw new Error(`Unexpected argument: ${arg}`);
    }

    const trimmed = arg.replace(/^-+/, "");
    const [rawKey, inlineValue] = trimmed.split(/=(.*)/s, 2);
    const normalizedKey = rawKey.toLowerCase().replace(/[-_]/g, "");
    const key = aliases[normalizedKey];
    if (!key) {
      throw new Error(`Unknown option: ${arg}`);
    }

    if (valueOptions.has(key)) {
      const value = inlineValue !== undefined ? inlineValue : args[++i];
      if (!value || value.startsWith("-")) {
        throw new Error(`Missing value for ${arg}`);
      }
      parsed[key] = value;
    } else {
      parsed[key] = true;
    }
  }

  return parsed;
}

function step(title, action) {
  console.log("");
  console.log(`==> ${title}`);
  action();
}

function run(command, args, runOptions = {}) {
  const executable = resolveRunnableCommand(command);
  console.log(`> ${quoteCommand(executable, args)}`);
  const launch = getLaunchCommand(executable, args);
  const result = spawnSync(launch.command, launch.args, {
    cwd: repoRoot,
    env: process.env,
    stdio: "inherit",
    shell: false,
    windowsHide: false,
    ...runOptions,
  });

  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error(`Command failed with exit code ${result.status}: ${quoteCommand(executable, args)}`);
  }
}

function capture(command, args) {
  const executable = resolveRunnableCommand(command);
  const launch = getLaunchCommand(executable, args);
  const result = spawnSync(launch.command, launch.args, {
    cwd: repoRoot,
    env: process.env,
    encoding: "utf8",
    shell: false,
    windowsHide: true,
  });
  return {
    ok: result.status === 0,
    stdout: result.stdout?.trim() || "",
    stderr: result.stderr?.trim() || "",
  };
}

function quoteCommand(command, args) {
  return [command, ...args].map((part) => (/\s/.test(part) ? `"${part}"` : part)).join(" ");
}

function getLaunchCommand(command, args) {
  if (process.platform === "win32" && /\.(cmd|bat)$/i.test(command)) {
    return {
      command: process.env.ComSpec || "cmd.exe",
      args: ["/d", "/c", "call", command, ...args],
    };
  }
  return { command, args };
}

function getAppVersion() {
  return JSON.parse(readFileSync(tauriConfigPath, "utf8")).version;
}

function updateReleaseVersion(version) {
  setAppVersion(version);
  updateReadmeVersion(version);
}

function setAppVersion(version) {
  const packageJson = JSON.parse(readFileSync(packageJsonPath, "utf8"));
  packageJson.version = version;
  writeFileSync(packageJsonPath, `${JSON.stringify(packageJson, null, 2)}\n`, "utf8");

  const tauriConfig = JSON.parse(readFileSync(tauriConfigPath, "utf8"));
  tauriConfig.version = version;
  writeFileSync(tauriConfigPath, `${JSON.stringify(tauriConfig, null, 2)}\n`, "utf8");

  const cargoToml = readFileSync(cargoTomlPath, "utf8").replace(/^version\s*=\s*"[^"]+"/m, `version = "${version}"`);
  writeFileSync(cargoTomlPath, cargoToml, "utf8");
}

function updateReadmeVersion(version) {
  if (!existsSync(readmePath)) {
    return;
  }

  const tagName = `v${version}`;
  const installerName = `Money Like Water_${version}_x64-setup.exe`;
  const installerUrl = `https://github.com/MarioLuLu7/money-like-water/releases/download/${tagName}/${encodeURIComponent(installerName)}`;
  let readme = readFileSync(readmePath, "utf8");

  readme = readme.replace(
    /\[Download v[^\]]+ for Windows x64\]\([^)]+\)/,
    `[Download ${tagName} for Windows x64](${installerUrl})`,
  );
  readme = readme.replace(
    /\[下载 v[^\]]+ Windows x64 版本\]\([^)]+\)/,
    `[下载 ${tagName} Windows x64 版本](${installerUrl})`,
  );

  writeFileSync(readmePath, readme, "utf8");
}

function getCurrentBranch() {
  const result = capture("git", ["branch", "--show-current"]);
  if (!result.ok || !result.stdout) {
    throw new Error("Cannot resolve current git branch. Pass --branch explicitly.");
  }
  return result.stdout;
}

function gitStatusShort() {
  const result = capture("git", ["status", "--short"]);
  if (!result.ok) {
    throw new Error(result.stderr || "Failed to read git status.");
  }
  return result.stdout;
}

function gitTagExists(tagName) {
  const result = capture("git", ["tag", "--list", tagName]);
  return result.ok && result.stdout === tagName;
}

function getReleaseArtifacts() {
  if (!existsSync(bundleRoot)) {
    return [];
  }

  const allowedExtensions = new Set([".msi", ".exe", ".msix", ".appinstaller", ".zip"]);
  return walkFiles(bundleRoot)
    .filter((file) => allowedExtensions.has(extname(file.path).toLowerCase()))
    .sort((a, b) => a.path.localeCompare(b.path));
}

function ensureUpdaterSigningKey() {
  if (process.env.TAURI_SIGNING_PRIVATE_KEY || process.env.TAURI_SIGNING_PRIVATE_KEY_PATH) {
    return;
  }

  if (existsSync(localUpdaterKeyPath)) {
    process.env.TAURI_SIGNING_PRIVATE_KEY_PATH = localUpdaterKeyPath;
    if (!process.env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD && existsSync(localUpdaterKeyPasswordPath)) {
      process.env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD = readFileSync(localUpdaterKeyPasswordPath, "utf8").trim();
    }
    return;
  }

  throw new Error(
    "Updater signing key not found. Generate one with `npm run tauri -- signer generate --ci -w .tauri\\updater.key`, keep it private, and rerun release.",
  );
}

function signUpdaterArtifact(artifactPath) {
  if (existsSync(`${artifactPath}.sig`)) {
    return;
  }

  const args = ["run", "tauri", "--", "signer", "sign", "--private-key-path", process.env.TAURI_SIGNING_PRIVATE_KEY_PATH || localUpdaterKeyPath];
  if (process.env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD) {
    args.push("--password", process.env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD);
  }
  args.push(artifactPath);
  run("npm", args);
}

function createUpdaterManifest({ releaseDir, artifacts, tagName, version }) {
  const artifact = selectUpdaterArtifact(artifacts);
  if (!artifact) {
    throw new Error("No Windows installer artifact was available for the updater manifest.");
  }

  const copiedArtifact = join(releaseDir, artifact.name);
  const signaturePath = `${copiedArtifact}.sig`;
  if (!existsSync(signaturePath)) {
    throw new Error(`Missing updater signature for ${artifact.name}. Check TAURI_SIGNING_PRIVATE_KEY_PATH or TAURI_SIGNING_PRIVATE_KEY.`);
  }

  const repoSlug = getGitHubRepoSlug();
  return {
    version,
    notes: `Release ${tagName}`,
    pub_date: new Date().toISOString(),
    platforms: {
      "windows-x86_64": {
        signature: readFileSync(signaturePath, "utf8").trim(),
        url: `https://github.com/${repoSlug}/releases/download/${tagName}/${encodeURIComponent(artifact.name)}`,
      },
    },
  };
}

function selectUpdaterArtifact(artifacts) {
  return (
    artifacts.find((artifact) => artifact.name.toLowerCase().endsWith(".exe") && artifact.name.toLowerCase().includes("setup")) ||
    artifacts.find((artifact) => artifact.name.toLowerCase().endsWith(".msi")) ||
    artifacts.find((artifact) => artifact.name.toLowerCase().endsWith(".exe")) ||
    null
  );
}

function getGitHubRepoSlug() {
  const result = capture("git", ["remote", "get-url", options.remote]);
  if (!result.ok || !result.stdout) {
    return "MarioLuLu7/money-like-water";
  }

  const remote = result.stdout.replace(/\.git$/, "");
  const httpsMatch = remote.match(/github\.com[:/](?<owner>[^/]+)\/(?<repo>[^/]+)$/);
  if (httpsMatch?.groups) {
    return `${httpsMatch.groups.owner}/${httpsMatch.groups.repo}`;
  }

  return "MarioLuLu7/money-like-water";
}

function listFiles(directory) {
  return walkFiles(directory).sort((a, b) => a.name.localeCompare(b.name));
}

function walkFiles(directory) {
  const files = [];
  for (const entry of readdirSync(directory)) {
    const path = join(directory, entry);
    const stats = statSync(path);
    if (stats.isDirectory()) {
      files.push(...walkFiles(path));
    } else if (stats.isFile()) {
      files.push({ path, name: entry });
    }
  }
  return files;
}

function ensureGitHubCli() {
  const resolved = resolveGitHubCli();
  if (resolved) {
    return resolved;
  }

  if (options.noAutoInstallGitHubCli) {
    throw new Error("GitHub CLI 'gh' was not found. Install it, sign in with 'gh auth login', or rerun with --no-github-release.");
  }

  const winget = resolveCommand("winget");
  if (!winget) {
    throw new Error("GitHub CLI 'gh' was not found and winget is unavailable. Install GitHub CLI, sign in with 'gh auth login', or rerun with --no-github-release.");
  }

  step("Install GitHub CLI with winget", () => {
    run(winget, ["install", "--id", "GitHub.cli", "-e", "--source", "winget", "--accept-package-agreements", "--accept-source-agreements"]);
  });

  const installed = resolveGitHubCli();
  if (!installed) {
    throw new Error("GitHub CLI was installed, but gh.exe could not be found in PATH or common install locations. Open a new terminal and rerun npm run release.");
  }
  return installed;
}

function resolveGitHubCli() {
  const fromPath = resolveCommand("gh");
  if (fromPath) {
    return fromPath;
  }

  const candidates = [
    process.env.ProgramFiles && join(process.env.ProgramFiles, "GitHub CLI", "gh.exe"),
    process.env["ProgramFiles(x86)"] && join(process.env["ProgramFiles(x86)"], "GitHub CLI", "gh.exe"),
    process.env.LOCALAPPDATA && join(process.env.LOCALAPPDATA, "Programs", "GitHub CLI", "gh.exe"),
  ].filter(Boolean);

  return candidates.find((candidate) => existsSync(candidate)) || "";
}

function resolveCommand(command) {
  const resolver = process.platform === "win32" ? "where.exe" : "which";
  const result = spawnSync(resolver, [command], {
    cwd: repoRoot,
    env: process.env,
    encoding: "utf8",
    shell: false,
    windowsHide: true,
  });
  const stdout = result.stdout?.trim() || "";
  if (result.status !== 0 || !stdout) {
    return "";
  }
  const matches = stdout.split(/\r?\n/).map((line) => line.trim()).filter(Boolean);
  if (process.platform !== "win32") {
    return matches[0] || "";
  }

  return matches.find((match) => /\.(cmd|exe|bat)$/i.test(match)) || matches[0] || "";
}

function resolveRunnableCommand(command) {
  if (/[\\/]/.test(command) || /^[A-Za-z]:[\\/]/.test(command)) {
    return command;
  }
  return resolveCommand(command) || command;
}

function ensureGitHubAuth(ghPath) {
  const status = capture(ghPath, ["auth", "status"]);
  if (status.ok) {
    return;
  }

  if (options.noInteractiveGitHubLogin) {
    throw new Error("GitHub CLI is installed but not signed in. Run 'gh auth login' or rerun without --no-interactive-github-login.");
  }

  step("Sign in to GitHub CLI", () => {
    run(ghPath, ["auth", "login", "--hostname", "github.com", "--git-protocol", "https", "--web"]);
  });
}

function githubReleaseExists(ghPath, tagName) {
  return capture(ghPath, ["release", "view", tagName]).ok;
}
