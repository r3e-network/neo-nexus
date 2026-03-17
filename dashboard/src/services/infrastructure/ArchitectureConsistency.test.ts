import fs from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

describe('ArchitectureConsistency', () => {
  it('does not depend on the legacy kubernetes deployer path', () => {
    const repoRoot = path.resolve(process.cwd(), '..');
    const dashboardRoot = process.cwd();
    const packageJson = JSON.parse(
      fs.readFileSync(path.join(dashboardRoot, 'package.json'), 'utf8'),
    ) as { dependencies?: Record<string, string> };

    expect(packageJson.dependencies?.['@kubernetes/client-node']).toBeUndefined();
    expect(
      fs.existsSync(path.join(repoRoot, 'dashboard/src/services/KubernetesDeployer.ts')),
    ).toBe(false);
  });

  it('does not retain legacy kubernetes endpoint columns in the active schema', () => {
    const repoRoot = path.resolve(process.cwd(), '..');
    const prismaSchema = fs.readFileSync(path.join(repoRoot, 'dashboard/prisma/schema.prisma'), 'utf8');
    const sqlSchema = fs.readFileSync(path.join(repoRoot, 'infrastructure/database/schema.sql'), 'utf8');

    expect(prismaSchema).not.toContain('k8sNamespace');
    expect(prismaSchema).not.toContain('k8sDeploymentName');
    expect(sqlSchema).not.toContain('"k8sNamespace"');
    expect(sqlSchema).not.toContain('"k8sDeploymentName"');
  });
});
