import net from 'node:net';

export type FirewallRuleType = 'ip_allow' | 'origin_allow' | 'method_block';

type FirewallRuleLike = {
  type: string;
  value: string;
};


function normalizeOrigin(value: string): string {
  const url = new URL(value.trim());
  if (url.protocol !== 'http:' && url.protocol !== 'https:') {
    throw new Error('Origin allowlist entries must use http or https.');
  }

  return `${url.protocol}//${url.host}`;
}

function normalizeIpOrCidr(value: string): string {
  const trimmed = value.trim();
  const [ip, cidr] = trimmed.split('/');

  if (!net.isIP(ip)) {
    throw new Error('IP allowlist entries must be valid IP addresses or CIDR ranges.');
  }

  if (!cidr) {
    return ip;
  }

  const prefix = Number.parseInt(cidr, 10);
  const maxPrefix = net.isIP(ip) === 6 ? 128 : 32;

  if (!Number.isInteger(prefix) || prefix < 0 || prefix > maxPrefix) {
    throw new Error('CIDR prefix length is invalid.');
  }

  return `${ip}/${prefix}`;
}

function normalizeRpcMethod(value: string): string {
  const normalized = value.trim().toLowerCase();
  if (!/^[a-z0-9_]+$/.test(normalized)) {
    throw new Error('Method firewall entries must be valid JSON-RPC method names.');
  }

  return normalized;
}

export function normalizeFirewallRuleValue(type: FirewallRuleType, value: string): string {
  if (type === 'origin_allow') {
    return normalizeOrigin(value);
  }

  if (type === 'method_block') {
    return normalizeRpcMethod(value);
  }

  return normalizeIpOrCidr(value);
}

function buildMethodFirewallFunction(blockedMethods: string[]) {
  const blockedMap = blockedMethods
    .map((method) => `["${method}"]=true`)
    .join(', ');

  return `return function(conf, ctx)
  local cjson = require("cjson.safe")
  ngx.req.read_body()
  local body = ngx.req.get_body_data()
  if not body then
    local body_file = ngx.req.get_body_file()
    if body_file then
      local handler = io.open(body_file, "rb")
      if handler then
        body = handler:read("*a")
        handler:close()
      end
    end
  end
  if not body then
    return
  end
  local payload = cjson.decode(body)
  if type(payload) ~= "table" then
    return
  end
  local blocked = {${blockedMap}}
  local function should_block(item)
    return type(item) == "table" and type(item.method) == "string" and blocked[string.lower(item.method)] == true
  end
  local blocked_method = nil
  if payload[1] ~= nil then
    for _, item in ipairs(payload) do
      if should_block(item) then
        blocked_method = item.method
        break
      end
    end
  elseif should_block(payload) then
    blocked_method = payload.method
  end
  if blocked_method then
    ngx.status = 403
    ngx.header["Content-Type"] = "application/json"
    ngx.say(cjson.encode({ error = "RPC method blocked by organization policy", method = blocked_method }))
    return ngx.exit(403)
  end
end`;
}

export function buildRouteSecurityPlugins(rules: FirewallRuleLike[]) {
  const ipAllowlist = rules
    .filter((rule) => rule.type === 'ip_allow')
    .map((rule) => rule.value);
  const originAllowlist = rules
    .filter((rule) => rule.type === 'origin_allow')
    .map((rule) => rule.value);
  const blockedMethods = Array.from(new Set(
    rules
      .filter((rule) => rule.type === 'method_block')
      .map((rule) => normalizeRpcMethod(rule.value)),
  ));

  return {
    ...(ipAllowlist.length > 0
      ? {
          'ip-restriction': {
            whitelist: ipAllowlist,
            message: 'Source IP is not allowed.',
            response_code: 403,
          },
        }
      : {}),
    ...(originAllowlist.length > 0
      ? {
          cors: {
            allow_origins: originAllowlist.join(','),
            allow_methods: 'GET,POST,OPTIONS',
            allow_headers: '*',
            expose_headers: '*',
            allow_credential: false,
          },
        }
      : {}),
    ...(blockedMethods.length > 0
      ? {
          'serverless-pre-function': {
            phase: 'access',
            functions: [buildMethodFirewallFunction(blockedMethods)],
          },
        }
      : {}),
  };
}
