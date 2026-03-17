import { formatAtomicGasAmount } from './CryptoBillingService';

type StripeInvoiceSummary = {
  id: string;
  amountPaidCents: number;
  currency: string;
  status: string;
  hostedInvoiceUrl: string | null;
  createdAt: string;
  description: string;
};

type CryptoBillingSummary = {
  id: number;
  plan: string;
  txHash: string;
  amountAtomic: string;
  verifiedAt: string;
};

type StripeCardSummary = {
  brand: string;
  last4: string;
  expMonth: number;
  expYear: number;
};

export type BillingHistoryItem = {
  id: string;
  source: 'stripe' | 'crypto';
  title: string;
  subtitle: string;
  amountLabel: string;
  status: string;
  createdAt: string;
  href: string | null;
};

export function buildBillingOverview(input: {
  stripeInvoices: StripeInvoiceSummary[];
  cryptoTransactions: CryptoBillingSummary[];
  stripeCard?: StripeCardSummary | null;
}) {
  const stripeItems: BillingHistoryItem[] = input.stripeInvoices.map((invoice) => ({
    id: invoice.id,
    source: 'stripe',
    title: invoice.description || 'Stripe invoice',
    subtitle: invoice.id,
    amountLabel: `${(invoice.amountPaidCents / 100).toFixed(2)} ${invoice.currency.toUpperCase()}`,
    status: invoice.status,
    createdAt: invoice.createdAt,
    href: invoice.hostedInvoiceUrl,
  }));

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

  const items = [...stripeItems, ...cryptoItems].sort(
    (left, right) => new Date(right.createdAt).getTime() - new Date(left.createdAt).getTime(),
  );

  return {
    items,
    cardSummary: input.stripeCard
      ? `${input.stripeCard.brand.toUpperCase()} ending in ${input.stripeCard.last4}`
      : null,
    cardExpiry: input.stripeCard
      ? `${String(input.stripeCard.expMonth).padStart(2, '0')}/${input.stripeCard.expYear}`
      : null,
  };
}
