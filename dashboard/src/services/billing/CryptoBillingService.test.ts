import { describe, expect, it } from 'vitest';
import { wallet } from '@cityofzion/neon-js';
import type { ApplicationLogJson, GetRawTransactionResult } from '@cityofzion/neon-core/lib/rpc/Query';
import {
  getRequiredAtomicAmount,
  getTransactionConfirmationCount,
  parseCryptoBillingConfig,
  selectMatchingGasTransfer,
} from './CryptoBillingService';

const treasuryScriptHash = '0123456789abcdef0123456789abcdef01234567';
const treasuryAddress = wallet.getAddressFromScriptHash(treasuryScriptHash);

function hash160StackItem(scriptHash: string) {
  return {
    type: 'ByteString',
    value: Buffer.from(scriptHash, 'hex').toString('base64'),
  } as const;
}

describe('CryptoBillingService', () => {
  it('parses the required billing configuration', () => {
    const config = parseCryptoBillingConfig({
      NEO_N3_RPC_URL: 'https://rpc.example.com',
      CRYPTO_BILLING_TREASURY_ADDRESS: treasuryAddress,
      CRYPTO_BILLING_GROWTH_AMOUNT_GAS: '15',
      CRYPTO_BILLING_DEDICATED_AMOUNT_GAS: '30',
      CRYPTO_BILLING_MIN_CONFIRMATIONS: '4',
    });

    expect(config.rpcUrl).toBe('https://rpc.example.com');
    expect(config.minConfirmations).toBe(4);
    expect(config.treasuryAddress).toBe(treasuryAddress);
    expect(getRequiredAtomicAmount('growth', config)).toBe(BigInt(1_500_000_000));
  });

  it('finds a matching GAS transfer to the configured treasury', () => {
    const applicationLog = {
      txid: '0x123',
      executions: [
        {
          trigger: 'Application',
          vmstate: 'HALT',
          gasconsumed: '0',
          notifications: [
            {
              contract: 'd2a4cff31913016155e38e474a2c06d08be276cf',
              eventname: 'Transfer',
              state: {
                type: 'Array',
                value: [
                  hash160StackItem('aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'),
                  hash160StackItem(treasuryScriptHash),
                  { type: 'Integer', value: '1500000000' },
                ],
              },
            },
          ],
        },
      ],
    } satisfies ApplicationLogJson;

    const transfer = selectMatchingGasTransfer(applicationLog, treasuryScriptHash);

    expect(transfer).toEqual({
      amountAtomic: BigInt(1_500_000_000),
      recipientScriptHash: treasuryScriptHash,
    });
  });

  it('uses confirmations from the raw transaction result', () => {
    const transaction = {
      confirmations: 7,
    } as GetRawTransactionResult;

    expect(getTransactionConfirmationCount(transaction)).toBe(7);
  });
});
