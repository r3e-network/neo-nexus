import { existsSync, readFileSync } from "fs";
import { join } from "path";

const root = process.cwd();
const installer = () => readFileSync(join(root, "install.sh"), "utf8");
const readme = () => readFileSync(join(root, "README.md"), "utf8");

describe("installation experience", () => {
  it("uses the current repository and installs both backend and frontend dependencies", () => {
    const script = installer();

    expect(script).toContain('REPO_URL="${REPO_URL:-https://github.com/r3e-network/neo-nexus.git}"');
    expect(script).toContain("git clone --depth 1 \"$REPO_URL\"");
    expect(script).toContain("npm ci");
    expect(script).toContain("npm --prefix web ci");
    expect(script).not.toContain("r3e-network/neonexus.git");
  });

  it("creates a persistent production environment for one-command installs", () => {
    const script = installer();

    expect(script).toContain('ENV_FILE="$INSTALL_DIR/.env"');
    expect(script).toContain("JWT_SECRET=");
    expect(script).toContain("EnvironmentFile=$ENV_FILE");
    expect(script).toContain('set -a; source "$ENV_FILE"; set +a');
  });

  it("documents the one-command installer and manual production start", () => {
    const docs = readme();

    expect(docs).toContain("curl -fsSL https://raw.githubusercontent.com/r3e-network/neo-nexus/main/install.sh | bash");
    expect(docs).toContain("JWT_SECRET=$(openssl rand -hex 32) npm start");
  });

  it("ships a Docker Compose entrypoint for simple local deployment", () => {
    expect(existsSync(join(root, "Dockerfile"))).toBe(true);
    expect(existsSync(join(root, "docker-compose.yml"))).toBe(true);
  });
});
