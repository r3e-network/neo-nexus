import { CONST, rpc, u, wallet } from '@cityofzion/neon-js';
import type { ApplicationLogJson, GetRawTransactionResult } from '@cityofzion/neon-core/lib/rpc/Query';
import type { StackItemJson } from '@cityofzion/neon-core/lib/sc/StackItem';
import type { BillingPlan } from '@/server/organization';

const GAS_DECIMALS = BigInt(8);
const GAS_FACTOR = BigInt(10) ** GAS_DECIMALS;

type ApplicationNotification = ApplicationLogJson['executions'][number]['notifications'][number];

export type CryptoBillingConfig = {
  rpcUrl: string;
  treasuryAddress: string;
  treasuryScriptHash: string;
  growthAmountAtomic: bigint;
  dedicatedAmountAtomic: bigint;
  minConfirmations: number;
};

export type VerifiedGasTransfer = {
  amountAtomic: bigint;
  recipientScriptHash: string;
};

export type PublicCryptoBillingConfig = {
  treasuryAddress: string;
  growthAmountGas: string;
  dedicatedAmountGas: string;
  minConfirmations: number;
};

function parseGasAmountToAtomic(value: string): bigint {
  const trimmedValue = value.trim();

  if (!/^\d+(\.\d{1,8})?$/.test(trimmedValue)) {
    throw new Error(`Invalid GAS amount: ${value}`);
  }

  const [whole, fraction = ''] = trimmedValue.split('.');
  const paddedFraction = `${fraction}00000000`.slice(0, 8);
  return BigInt(whole) * GAS_FACTOR + BigInt(paddedFraction);
}

export function formatAtomicGasAmount(amountAtomic: bigint): string {
  const whole = amountAtomic / GAS_FACTOR;
  const fraction = amountAtomic % GAS_FACTOR;

  if (fraction === BigInt(0)) {
    return whole.toString();
  }

  return `${whole}.${fraction.toString().padStart(8, '0').replace(/0+$/, '')}`;
}

function parsePositiveInteger(value: string | undefined, fallback: number): number {
  if (!value) {
    return fallback;
  }

  const parsed = Number.parseInt(value, 10);
  if (!Number.isInteger(parsed) || parsed < 0) {
    throw new Error('CRYPTO_BILLING_MIN_CONFIRMATIONS must be a non-negative integer.');
  }

  return parsed;
}

function normalizeHash(hash: string): string {
  return hash.replace(/^0x/i, '').toLowerCase();
}

function normalizeHashCandidates(hash: string): string[] {
  const normalized = normalizeHash(hash);

  try {
    return [normalized, normalizeHash(u.reverseHex(normalized))];
  } catch {
    return [normalized];
  }
}

function readHash160Value(item: { type: string; value?: string | boolean | number | StackItemJson[] }): string | null {
  if ((item.type !== 'ByteString' && item.type !== 'Buffer') || typeof item.value !== 'string') {
    return null;
  }

  const decoded = Buffer.from(item.value, 'base64').toString('hex');
  return decoded ? normalizeHash(decoded) : null;
}

function readIntegerValue(item: { type: string; value?: string | boolean | number | StackItemJson[] }): bigint | null {
  if (item.type === 'Integer' && typeof item.value === 'string') {
    return BigInt(item.value);
  }

  if ((item.type === 'ByteString' || item.type === 'Buffer') && typeof item.value === 'string') {
    const decoded = Buffer.from(item.value, 'base64').toString('hex');
    if (!decoded) {
      return null;
    }

    return BigInt(`0x${decoded}`);
  }

  return null;
}

function isTransferState(
  state: ApplicationNotification['state'],
): state is { type: 'Array'; value: StackItemJson[] } {
  return state.type === 'Array' && Array.isArray(state.value);
}

export function parseCryptoBillingConfig(
  env: Record<string, string | undefined> = process.env,
): CryptoBillingConfig {
  const rpcUrl = env.NEO_N3_RPC_URL?.trim();
  const treasuryAddress = env.CRYPTO_BILLING_TREASURY_ADDRESS?.trim();
  const growthAmount = env.CRYPTO_BILLING_GROWTH_AMOUNT_GAS?.trim();
  const dedicatedAmount = env.CRYPTO_BILLING_DEDICATED_AMOUNT_GAS?.trim();

  if (!rpcUrl) {
    throw new Error('NEO_N3_RPC_URL must be configured for crypto billing verification.');
  }

  if (!treasuryAddress || !wallet.isAddress(treasuryAddress)) {
    throw new Error('CRYPTO_BILLING_TREASURY_ADDRESS must be configured with a valid N3 address.');
  }

  if (!growthAmount || !dedicatedAmount) {
    throw new Error('CRYPTO_BILLING_GROWTH_AMOUNT_GAS and CRYPTO_BILLING_DEDICATED_AMOUNT_GAS must be configured.');
  }

  return {
    rpcUrl,
    treasuryAddress,
    treasuryScriptHash: normalizeHash(wallet.getScriptHashFromAddress(treasuryAddress)),
    growthAmountAtomic: parseGasAmountToAtomic(growthAmount),
    dedicatedAmountAtomic: parseGasAmountToAtomic(dedicatedAmount),
    minConfirmations: parsePositiveInteger(env.CRYPTO_BILLING_MIN_CONFIRMATIONS, 1),
  };
}

export function getRequiredAtomicAmount(plan: BillingPlan, config: CryptoBillingConfig): bigint {
  if (plan === 'growth') {
    return config.growthAmountAtomic;
  }

  if (plan === 'dedicated') {
    return config.dedicatedAmountAtomic;
  }

  throw new Error(`Unsupported billing plan for crypto verification: ${plan}`);
}

export function getPublicCryptoBillingConfig(
  env: Record<string, string | undefined> = process.env,
): PublicCryptoBillingConfig | null {
  try {
    const config = parseCryptoBillingConfig(env);

    return {
      treasuryAddress: config.treasuryAddress,
      growthAmountGas: formatAtomicGasAmount(config.growthAmountAtomic),
      dedicatedAmountGas: formatAtomicGasAmount(config.dedicatedAmountAtomic),
      minConfirmations: config.minConfirmations,
    };
  } catch {
    return null;
  }
}

export function selectMatchingGasTransfer(
  applicationLog: ApplicationLogJson,
  treasuryScriptHash: string,
): VerifiedGasTransfer | null {
  const normalizedTreasuryHash = normalizeHash(treasuryScriptHash);

  for (const execution of applicationLog.executions) {
    if (execution.vmstate !== 'HALT') {
      continue;
    }

    for (const notification of execution.notifications) {
      if (normalizeHash(notification.contract) !== normalizeHash(CONST.NATIVE_CONTRACT_HASH.GasToken)) {
        continue;
      }

      if (notification.eventname !== 'Transfer' || !isTransferState(notification.state)) {
        continue;
      }

      const [, recipientItem, amountItem] = notification.state.value;
      if (!recipientItem || !amountItem) {
        continue;
      }

      const recipientHash = readHash160Value(recipientItem);
      const amountAtomic = readIntegerValue(amountItem);
      if (!recipientHash || amountAtomic === null) {
        continue;
      }

      if (normalizeHashCandidates(recipientHash).includes(normalizedTreasuryHash)) {
        return {
          amountAtomic,
          recipientScriptHash: normalizedTreasuryHash,
        };
      }
    }
  }

  return null;
}

export function getTransactionConfirmationCount(transaction: GetRawTransactionResult): number {
  return typeof transaction.confirmations === 'number' ? transaction.confirmations : 0;
}

export async function verifyCryptoTransferOnChain(
  txHash: string,
  plan: Extract<BillingPlan, 'growth' | 'dedicated'>,
  env: Record<string, string | undefined> = process.env,
) {
  const config = parseCryptoBillingConfig(env);
  const client = new rpc.RPCClient(config.rpcUrl);
  const transaction = await client.getRawTransaction(txHash, true);
  const confirmations = getTransactionConfirmationCount(transaction);

  if (transaction.vm_state !== 'HALT') {
    throw new Error('Transaction did not complete successfully on-chain.');
  }

  if (confirmations < config.minConfirmations) {
    throw new Error(`Transaction requires at least ${config.minConfirmations} confirmation(s).`);
  }

  const applicationLog = await client.getApplicationLog(txHash);
  const transfer = selectMatchingGasTransfer(applicationLog, config.treasuryScriptHash);
  if (!transfer) {
    throw new Error('No GAS transfer to the configured treasury address was found in the transaction.');
  }

  const requiredAmountAtomic = getRequiredAtomicAmount(plan, config);
  if (transfer.amountAtomic < requiredAmountAtomic) {
    throw new Error('Transaction amount does not satisfy the required billing amount.');
  }

  return {
    config,
    confirmations,
    amountAtomic: transfer.amountAtomic,
  };
}
