import { buildDefaultNodeSettings } from '../settings/NodeSettings';
import {
  REMOTE_NEO_GO_CONFIG_PATH,
  REMOTE_NODE_SETTINGS_PATH,
  REMOTE_NODE_SYNC_PATH,
  renderNodeRuntimeArtifacts,
} from '../settings/RenderedNodeRuntime';

type BootstrapInput = {
  endpointName: string;
  protocol: 'neo-n3' | 'neo-x';
  clientEngine: 'neo-go' | 'neo-cli' | 'neo-x-geth';
  network: 'mainnet' | 'testnet' | 'private';
  syncMode: 'full' | 'archive';
  operatorPublicKey?: string;
};

function toWriteFileBlock(path: string, permissions: string, content: string): string {
  return `  - path: ${path}
    permissions: '${permissions}'
    owner: root:root
    content: |
${content
  .split('\n')
  .map((line) => `      ${line}`)
  .join('\n')}
`;
}

export function buildNodeBootstrapScript(input: BootstrapInput): string {
  const defaultSettings = buildDefaultNodeSettings(input.clientEngine);
  const renderedRuntime = renderNodeRuntimeArtifacts({
    clientEngine: input.clientEngine,
    network: input.network,
    settings: defaultSettings,
  });
  const operatorKeyBlock = input.operatorPublicKey
    ? `
ssh_authorized_keys:
  - ${input.operatorPublicKey}
`
    : '';

  const writeFiles = `
write_files:
${toWriteFileBlock(REMOTE_NODE_SETTINGS_PATH, '0644', JSON.stringify(defaultSettings, null, 2))}${renderedRuntime.neoGoConfig ? toWriteFileBlock(REMOTE_NEO_GO_CONFIG_PATH, '0644', renderedRuntime.neoGoConfig) : ''}${toWriteFileBlock(REMOTE_NODE_SYNC_PATH, '0755', renderedRuntime.runScript)}  - path: /usr/local/bin/neonexus-plugin-sync
    permissions: '0755'
    owner: root:root
    content: |
      #!/usr/bin/env python3
      import glob
      import json
      import os
      import subprocess
      from pathlib import Path

      PLUGIN_DIR = Path('/etc/neonexus/plugins')
      RUNTIME_DIR = Path('/var/lib/neonexus/plugins')
      PLUGIN_DIR.mkdir(parents=True, exist_ok=True)
      RUNTIME_DIR.mkdir(parents=True, exist_ok=True)

      desired_containers = set()

      def run(cmd, check=True):
          subprocess.run(cmd, check=check, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)

      for config_path in glob.glob(str(PLUGIN_DIR / '*.json')):
          with open(config_path, 'r', encoding='utf-8') as file:
              payload = json.load(file)

          plugin_id = payload['pluginId']
          runtime = payload.get('runtime', {})
          image = runtime.get('image')
          if not image:
              raise RuntimeError(f'No runtime image configured for plugin {plugin_id}')

          container_name = f'neonexus-plugin-{plugin_id}'
          desired_containers.add(container_name)

          plugin_runtime_dir = RUNTIME_DIR / plugin_id
          plugin_runtime_dir.mkdir(parents=True, exist_ok=True)

          env_path = plugin_runtime_dir / 'plugin.env'
          secret_payloads = payload.get('secretPayloads', {})
          with open(env_path, 'w', encoding='utf-8') as env_file:
              env_file.write(f'NEONEXUS_PLUGIN_ID={plugin_id}\n')
              env_file.write('NEONEXUS_PLUGIN_CONFIG=/etc/neonexus/plugin.json\n')
              for key, value in secret_payloads.items():
                  env_file.write(f'{key.upper()}={value}\n')

          run(['docker', 'rm', '-f', container_name], check=False)
          run([
              'docker', 'run', '-d',
              '--name', container_name,
              '--restart', 'unless-stopped',
              '--network', 'host',
              '--env-file', str(env_path),
              '-v', f'{config_path}:/etc/neonexus/plugin.json:ro',
              image,
          ])

      existing = subprocess.run(
          ['docker', 'ps', '-a', '--format', '{{.Names}}'],
          check=True,
          stdout=subprocess.PIPE,
          stderr=subprocess.PIPE,
          text=True,
      )
      for line in existing.stdout.splitlines():
          if line.startswith('neonexus-plugin-') and line not in desired_containers:
              run(['docker', 'rm', '-f', line], check=False)
  - path: /etc/systemd/system/neonexus-plugin-sync.service
    permissions: '0644'
    owner: root:root
    content: |
      [Unit]
      Description=NeoNexus plugin sync
      After=network.target docker.service

      [Service]
      Type=oneshot
      ExecStart=/usr/local/bin/neonexus-plugin-sync
      RemainAfterExit=yes

      [Install]
      WantedBy=multi-user.target
`;

  return `#cloud-config
package_update: true
packages:
  - docker.io
  - ufw
  - python3
${operatorKeyBlock}
${writeFiles}
runcmd:
  - systemctl enable docker
  - systemctl start docker
  - systemctl daemon-reload
  - systemctl enable neonexus-plugin-sync
  - mkdir -p /var/lib/neonexus
  - mkdir -p /etc/neonexus/plugins
  - ufw --force reset
  - ufw default deny incoming
  - ufw default allow outgoing
  - ufw allow 22/tcp
  - ${input.protocol === 'neo-x' ? 'ufw allow 8545/tcp && ufw allow 8546/tcp && ufw allow 30303/tcp' : 'ufw allow 10332/tcp && ufw allow 10333/tcp && ufw allow 2112/tcp'}
  - ufw --force enable
  - ${REMOTE_NODE_SYNC_PATH}
final_message: "NeoNexus bootstrap complete for ${input.endpointName}"
`;
}
