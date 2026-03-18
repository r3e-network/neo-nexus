'use client';

import { useRouter } from 'next/navigation';
import { useState } from 'react';
import toast from 'react-hot-toast';
import { CheckCircle2 } from 'lucide-react';
import type { BillingHistoryItem } from '@/services/billing/BillingOverviewService';
import type { PublicCryptoBillingConfig } from '@/services/billing/CryptoBillingService';
import { verifyCryptoPaymentAction } from './actions';

const getErrorMessage = (error: unknown, fallback: string) =>
  error instanceof Error ? error.message : fallback;

type BillingClientProps = {
  billingPlan: string;
  cryptoBillingConfig: PublicCryptoBillingConfig | null;
  billingOverview: {
    items: BillingHistoryItem[];
  };
};

export default function BillingClient({ billingPlan, cryptoBillingConfig, billingOverview }: BillingClientProps) {
  const router = useRouter();
  const [isProcessing, setIsProcessing] = useState<string | null>(null);
  const [isCryptoModalOpen, setIsCryptoModalOpen] = useState(false);
  const [txHash, setTxHash] = useState('');
  const [cryptoPlanSelected, setCryptoPlanSelected] = useState<'growth' | 'dedicated'>(
    billingPlan === 'developer' ? 'growth' : 'dedicated',
  );

  const handleCryptoPayment = async () => {
    if (!txHash.trim()) {
      toast.error('Paste the submitted transaction hash before verifying.');
      return;
    }

    setIsProcessing('crypto');
    toast.loading('Verifying transaction on-chain...', { id: 'crypto-pay' });

    try {
      const result = await verifyCryptoPaymentAction(cryptoPlanSelected, txHash.trim());

      if (result.success) {
        toast.success(
          result.alreadyVerified
            ? 'This payment was already verified for your account.'
            : `Successfully upgraded to ${cryptoPlanSelected} using GAS!`,
          { id: 'crypto-pay' },
        );
        setIsCryptoModalOpen(false);
        setIsProcessing(null);
        router.refresh();
      } else {
        throw new Error(result.error);
      }
    } catch (error: unknown) {
      toast.error(getErrorMessage(error, 'Payment verification failed.'), { id: 'crypto-pay' });
      setIsProcessing(null);
    }
  };

  const handleNeoLinePayment = async () => {
    if (!cryptoBillingConfig) return;
    
    interface NeoLineTxResult { txid: string }
    interface WindowWithNeoLine extends Window {
      NEOLineN3?: {
        Init: new () => {
          getAccount: () => Promise<{ address: string }>;
          send: (args: { fromAddress: string, toAddress: string, asset: string, amount: string, network: string }) => Promise<NeoLineTxResult>;
        };
      };
    }

    const win = typeof window !== 'undefined' ? (window as unknown as WindowWithNeoLine) : null;

    // Check if NeoLine is installed
    if (!win || !win.NEOLineN3) {
      toast.error('NeoLine wallet is not installed. Please install it or manually transfer GAS and paste the transaction hash.');
      return;
    }

    setIsProcessing('crypto-wallet');
    toast.loading('Connecting to NeoLine...', { id: 'crypto-pay' });

    try {
      const neolineN3 = new win.NEOLineN3.Init();
      const account = await neolineN3.getAccount();
      
      const amount = cryptoPlanSelected === 'growth' 
        ? cryptoBillingConfig.growthAmountGas 
        : cryptoBillingConfig.dedicatedAmountGas;

      toast.loading('Please confirm the transaction in your wallet...', { id: 'crypto-pay' });

      const txResult = await neolineN3.send({
        fromAddress: account.address,
        toAddress: cryptoBillingConfig.treasuryAddress,
        asset: 'GAS',
        amount: amount.toString(),
        network: 'MainNet'
      });

      if (txResult && txResult.txid) {
        setTxHash(txResult.txid);
        toast.success('Transaction sent! Waiting for verification...', { id: 'crypto-pay' });
        
        // Wait a few seconds to give the network time to index it (not strictly necessary but helpful)
        await new Promise((resolve) => setTimeout(resolve, 3000));
        
        setIsProcessing('crypto');
        toast.loading('Verifying transaction on-chain...', { id: 'crypto-pay' });
        
        const verifyResult = await verifyCryptoPaymentAction(cryptoPlanSelected, txResult.txid);
        
        if (verifyResult.success) {
          toast.success(
            verifyResult.alreadyVerified
              ? 'This payment was already verified for your account.'
              : `Successfully upgraded to ${cryptoPlanSelected} using GAS!`,
            { id: 'crypto-pay' },
          );
          setIsCryptoModalOpen(false);
          setIsProcessing(null);
          router.refresh();
        } else {
          throw new Error(verifyResult.error);
        }
      } else {
        throw new Error('Transaction was cancelled or failed.');
      }
    } catch (error: unknown) {
      console.error(error);
      const errObj = error as Record<string, unknown>;
      const errorMessage = (typeof errObj?.description === 'string' ? errObj.description : null) || 
                           (typeof errObj?.message === 'string' ? errObj.message : null) || 
                           'Wallet transaction failed or was cancelled.';
      toast.error(errorMessage, { id: 'crypto-pay' });
      setIsProcessing(null);
    }
  };

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">Billing</h1>
        <p className="text-gray-400 mt-1">Manage your subscriptions and payment methods.</p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2 space-y-6">
          <div className="bg-[var(--color-dark-panel)] border border-[#00E599]/30 rounded-xl p-6 relative overflow-hidden">
            <div className="absolute top-0 right-0 w-32 h-32 bg-[#00E599]/10 rounded-full blur-3xl"></div>
            <h2 className="text-sm font-bold text-[#00E599] uppercase tracking-wider mb-2">Current Plan</h2>
            <div className="flex items-baseline gap-2 mb-4">
              <span className="text-4xl font-bold text-white capitalize">{billingPlan}</span>
            </div>
            <p className="text-sm text-gray-300 mb-6 w-3/4">
              {billingPlan === 'developer'
                ? 'You are on the free Developer plan. Limited requests, shared nodes only.'
                : 'You have access to premium features and higher request limits.'}
            </p>

            <div className="space-y-3 mb-8">
              <div className="flex items-center gap-2 text-sm text-gray-300">
                <CheckCircle2 className="w-4 h-4 text-[#00E599]" />
                {billingPlan === 'developer' ? 'Basic Analytics' : 'Advanced Analytics'}
              </div>
              <div className="flex items-center gap-2 text-sm text-gray-300">
                <CheckCircle2 className="w-4 h-4 text-[#00E599]" />
                {billingPlan === 'developer' ? 'Community Support' : 'Priority Support'}
              </div>
              {billingPlan !== 'developer' && (
                <div className="flex items-center gap-2 text-sm text-gray-300">
                  <CheckCircle2 className="w-4 h-4 text-[#00E599]" /> Marketplace Plugins Access
                </div>
              )}
            </div>

            {billingPlan !== 'dedicated' && (
              <div className="flex flex-col sm:flex-row gap-4 mt-4">
                <button
                  onClick={() => { setCryptoPlanSelected('growth'); setIsCryptoModalOpen(true); }}
                  className="bg-[#00E599] hover:bg-[#00cc88] text-black px-4 py-2 rounded-md font-bold transition-colors flex items-center justify-center gap-2"
                >
                  Upgrade to Growth ({cryptoBillingConfig?.growthAmountGas || 0} GAS/mo)
                </button>
                <button
                  onClick={() => { setCryptoPlanSelected('dedicated'); setIsCryptoModalOpen(true); }}
                  className="bg-[#333333] hover:bg-[#444444] text-white px-4 py-2 rounded-md font-bold transition-colors flex items-center justify-center gap-2"
                >
                  Upgrade to Dedicated ({cryptoBillingConfig?.dedicatedAmountGas || 0} GAS/mo)
                </button>
              </div>
            )}
          </div>

          <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl overflow-hidden">
            <div className="p-6 border-b border-[var(--color-dark-border)]">
              <h2 className="text-lg font-medium text-white">Invoices & On-Chain Payments</h2>
            </div>
            <div className="divide-y divide-[#333333]">
              {billingOverview.items.length === 0 ? (
                <div className="p-6 text-sm text-gray-400">
                  No recent billing activity yet.
                </div>
              ) : (
                billingOverview.items.map((item) => (
                  <div key={item.id} className="p-6 flex flex-col md:flex-row md:items-center justify-between gap-4 text-sm">
                    <div>
                      <div className="font-medium text-white">{item.title}</div>
                      <div className="text-gray-500 mt-1">{item.subtitle}</div>
                      <div className="text-xs text-gray-500 mt-2">{new Date(item.createdAt).toLocaleString()}</div>
                    </div>
                    <div className="text-right">
                      <div className="font-medium text-white">{item.amountLabel}</div>
                      <div className="text-xs uppercase tracking-wide text-[#00E599] mt-1">{item.status}</div>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>

        <div>
          <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6 sticky top-6">
            <h2 className="text-lg font-medium text-white mb-6">Payment Method</h2>

            <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-lg p-4 flex items-center gap-4 mb-4">
              <div className="bg-[#00E599]/20 p-2 rounded">
                <div className="w-6 h-6 rounded-full bg-[#00E599] flex items-center justify-center text-black text-xs font-bold">N</div>
              </div>
              <div className="flex-1">
                <div className="text-gray-300 font-medium">Neo N3 Wallet</div>
                <div className="text-xs text-gray-500 mt-1">Native Web3 Integration</div>
              </div>
            </div>

            <div className="pt-2">
              {cryptoBillingConfig ? (
                <>
                  <p className="text-sm text-gray-400 mb-6">
                    Connect your NeoLine wallet to pay for subscriptions directly using GAS.
                  </p>

                  {billingPlan !== 'dedicated' ? (
                    <button
                      onClick={() => setIsCryptoModalOpen(true)}
                      className="w-full bg-[#00E599]/10 hover:bg-[#00E599]/20 text-[#00E599] border border-[#00E599]/30 py-3 rounded-xl font-bold transition-colors flex items-center justify-center gap-2"
                    >
                      <div className="w-4 h-4 rounded-full bg-[#00E599] flex items-center justify-center text-black text-[10px] font-bold">N</div>
                      {billingPlan === 'developer' ? 'Upgrade Plan with GAS' : 'Upgrade to Dedicated'}
                    </button>
                  ) : (
                    <div className="text-sm text-green-400 border border-green-400/30 bg-green-400/10 p-3 rounded-lg text-center font-medium">
                      Subscription active via verified on-chain payment.
                    </div>
                  )}
                </>
              ) : (
                <div className="text-sm text-yellow-400 border border-yellow-400/30 bg-yellow-400/10 p-3 rounded-lg">
                  Crypto billing is unavailable until the server is configured with a treasury address, an N3 RPC endpoint, and plan pricing.
                </div>
              )}
            </div>
          </div>
        </div>
      </div>

      {isCryptoModalOpen && cryptoBillingConfig && (
        <div className="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-2xl max-w-md w-full p-6 shadow-2xl relative">
            <button
              onClick={() => setIsCryptoModalOpen(false)}
              className="absolute top-4 right-4 text-gray-500 hover:text-white"
            >
              ✕
            </button>
            <h3 className="text-xl font-bold text-white mb-6 flex items-center gap-2">
              <div className="w-6 h-6 rounded-full bg-[#00E599] flex items-center justify-center text-black text-xs font-bold">N</div>
              Pay with Neo N3
            </h3>

            <div className="space-y-4 mb-6">
              {billingPlan === 'developer' && (
                <label
                  className={`block border rounded-xl p-4 cursor-pointer transition-colors ${cryptoPlanSelected === 'growth' ? 'border-[#00E599] bg-[#00E599]/5' : 'border-[var(--color-dark-border)] hover:border-gray-500'}`}
                >
                  <div className="flex items-center justify-between mb-1">
                    <div className="flex items-center gap-3">
                      <input
                        type="radio"
                        checked={cryptoPlanSelected === 'growth'}
                        onChange={() => setCryptoPlanSelected('growth')}
                        className="accent-[#00E599] w-4 h-4"
                      />
                      <span className="font-bold text-white">Growth Plan</span>
                    </div>
                    <span className="text-[#00E599] font-medium">{cryptoBillingConfig.growthAmountGas} GAS</span>
                  </div>
                  <div className="pl-7 text-xs text-gray-400">Unlock marketplace plugins and advanced features.</div>
                </label>
              )}

              <label
                className={`block border rounded-xl p-4 cursor-pointer transition-colors ${cryptoPlanSelected === 'dedicated' ? 'border-[#00E599] bg-[#00E599]/5' : 'border-[var(--color-dark-border)] hover:border-gray-500'}`}
              >
                <div className="flex items-center justify-between mb-1">
                  <div className="flex items-center gap-3">
                    <input
                      type="radio"
                      checked={cryptoPlanSelected === 'dedicated'}
                      onChange={() => setCryptoPlanSelected('dedicated')}
                      className="accent-[#00E599] w-4 h-4"
                    />
                    <span className="font-bold text-white">Dedicated Plan</span>
                  </div>
                  <span className="text-[#00E599] font-medium">{cryptoBillingConfig.dedicatedAmountGas} GAS</span>
                </div>
                <div className="pl-7 text-xs text-gray-400">Deploy dedicated infrastructure and custom configurations.</div>
              </label>
            </div>

            <div className="space-y-3 mb-6">
              <div className="text-xs uppercase tracking-wide text-gray-500">Treasury Address</div>
              <code className="block rounded-lg border border-[var(--color-dark-border)] bg-black/20 px-3 py-3 text-sm text-[#00E599] break-all">
                {cryptoBillingConfig.treasuryAddress}
              </code>
              <div className="text-xs text-gray-500">
                Minimum confirmations required: {cryptoBillingConfig.minConfirmations}
              </div>
            </div>

            <div className="space-y-4 mb-6">
              <button
                onClick={handleNeoLinePayment}
                disabled={!!isProcessing}
                className="w-full bg-[#00E599] hover:bg-[#00cc88] disabled:opacity-50 text-black py-3 rounded-xl font-bold transition-all shadow-[0_0_15px_rgba(0,229,153,0.2)] flex items-center justify-center gap-2"
              >
                {isProcessing === 'crypto-wallet' ? 'Connecting...' : 'Pay with NeoLine Wallet'}
              </button>

              <div className="relative flex items-center py-2">
                <div className="flex-grow border-t border-[var(--color-dark-border)]"></div>
                <span className="flex-shrink-0 mx-4 text-xs text-gray-500 font-medium uppercase">Or enter manually</span>
                <div className="flex-grow border-t border-[var(--color-dark-border)]"></div>
              </div>

              <label className="block">
                <span className="text-sm font-medium text-gray-300 mb-2 block">Submitted Transaction Hash</span>
                <input
                  value={txHash}
                  onChange={(event) => setTxHash(event.target.value)}
                  placeholder="0x..."
                  className="w-full rounded-xl border border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] px-4 py-3 text-white focus:outline-none focus:border-[#00E599]"
                />
              </label>

              <button
                onClick={handleCryptoPayment}
                disabled={isProcessing === 'crypto'}
                className="w-full bg-[#333333] hover:bg-[#444444] disabled:opacity-50 text-white py-3 rounded-xl font-bold transition-colors"
              >
                {isProcessing === 'crypto' ? 'Verifying...' : 'Verify Manual Payment'}
              </button>
            </div>

            <p className="text-xs text-center text-gray-500 mt-4">
              The server validates the submitted transaction against the configured N3 RPC node and treasury requirements.
            </p>
          </div>
        </div>
      )}
    </div>
  );
}
