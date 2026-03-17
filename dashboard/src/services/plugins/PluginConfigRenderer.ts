type RenderInput = {
  pluginId: string;
  endpointId: number;
  secretRefs: string[];
  secretPayloads: Record<string, string>;
  runtimeImage: string;
  configData: unknown;
};

export function renderPluginConfig(input: RenderInput): string {
  return JSON.stringify(
    {
      pluginId: input.pluginId,
      endpointId: input.endpointId,
      secretRefs: input.secretRefs,
      secretPayloads: input.secretPayloads,
      runtime: {
        image: input.runtimeImage,
        configPath: '/etc/neonexus/plugin.json',
      },
      config: input.configData,
    },
    null,
    2,
  );
}
