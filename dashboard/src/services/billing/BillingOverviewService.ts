import { formatAtomicGasAmount } from './CryptoBillingService';

type CryptoBillingSummary = {
  id: number;
  plan: string;
  txHash: string;
  amountAtomic: string;
  verifiedAt: string;
};

export type BillingHistoryItem = {
  id: string;
  source: 'crypto';
  title: string;
  subtitle: string;
  amountLabel: string;
  status: string;
  createdAt: string;
  href: string | null;
};

export function buildBillingOverview(input: {
  cryptoTransactions: CryptoBillingSummary[];
}) {
  const cryptoItems: BillingHistoryItem[] = input.cryptoTransactions.map((transaction) => ({
    id: `crypto-${transaction.id}`,
    source: 'crypto',
    title: `${transaction.plan.charAt(0).toUpperCase()}${transaction.plan.slice(1)} plan`,
    subtitle: transaction.txHash,
    amountLabel: `${formatAtomicGasAmount(BigInt(transaction.amountAtomic))} GAS`,
    status: 'verified',
    createdAt: transaction.verifiedAt,
    href: null,
  }));

  const items = cryptoItems.sort(
    (left, right) => new Date(right.createdAt).getTime() - new Date(left.createdAt).getTime(),
  );

  return {
    items,
  };
}
