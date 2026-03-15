'use client';

import { useState } from 'react';
import toast from 'react-hot-toast';
import { CreditCard, CheckCircle2 } from 'lucide-react';
import { upgradePlanAction, verifyCryptoPaymentAction } from './actions';

export default function BillingClient({ billingPlan }: { billingPlan: string }) {
  const [isProcessing, setIsProcessing] = useState<string | null>(null);
  const [isCryptoModalOpen, setIsCryptoModalOpen] = useState(false);
  const [cryptoPlanSelected, setCryptoPlanSelected] = useState<'growth' | 'dedicated'>(billingPlan === 'developer' ? 'growth' : 'dedicated');

  const handleStripeUpgrade = async (plan: 'growth' | 'dedicated') => {
    setIsProcessing(plan);
    try {
      await upgradePlanAction(plan);
    } catch (error: any) {
      toast.error(error.message || 'Failed to start checkout session');
      setIsProcessing(null);
    }
  };

  const handleCryptoPayment = async () => {
    setIsProcessing('crypto');
    toast.loading('Waiting for wallet signature...', { id: 'crypto-pay' });
    
    try {
      // In a real implementation with @cityofzion/wallet-connect-sdk-core:
      // 1. WcSdk.init()
      // 2. WcSdk.connect()
      // 3. WcSdk.sendTransaction({ to: 'NeoNexusTreasuryAddress', asset: 'GAS', amount: cost })
      
      // Simulate wallet connection and transaction delay
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      // Simulated tx hash
      const txHash = '0x' + Math.random().toString(16).substring(2, 40) + '...';
      
      toast.loading('Verifying transaction on blockchain...', { id: 'crypto-pay' });
      
      // Send to backend to verify
      const result = await verifyCryptoPaymentAction(cryptoPlanSelected, txHash);
      
      if (result.success) {
        toast.success(`Successfully upgraded to ${cryptoPlanSelected} using GAS!`, { id: 'crypto-pay' });
        window.location.reload();
      } else {
        throw new Error(result.error);
      }
      
    } catch (error: any) {
      toast.error(error.message || 'Payment failed or rejected by user.', { id: 'crypto-pay' });
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
          <div className="bg-[#1A1A1A] border border-[#00E599]/30 rounded-xl p-6 relative overflow-hidden">
            <div className="absolute top-0 right-0 w-32 h-32 bg-[#00E599]/10 rounded-full blur-3xl"></div>
            <h2 className="text-sm font-bold text-[#00E599] uppercase tracking-wider mb-2">Current Plan</h2>
            <div className="flex items-baseline gap-2 mb-4">
              <span className="text-4xl font-bold text-white capitalize">{billingPlan}</span>
            </div>
            <p className="text-sm text-gray-300 mb-6 w-3/4">
              {billingPlan === 'developer' ? 'You are on the free Developer plan. Limited requests, shared nodes only.' : 'You have access to premium features and higher request limits.'}
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
              <div className="flex flex-col sm:flex-row gap-4">
                {billingPlan === 'developer' && (
                  <button 
                    onClick={() => handleStripeUpgrade('growth')}
                    disabled={!!isProcessing}
                    className="bg-[#00E599] hover:bg-[#00cc88] disabled:opacity-50 text-black px-4 py-2 rounded-md font-bold transition-colors flex items-center justify-center gap-2"
                  >
                    {isProcessing === 'growth' ? 'Loading...' : 'Upgrade to Growth ($49/mo)'}
                  </button>
                )}
                <button 
                  onClick={() => handleStripeUpgrade('dedicated')}
                  disabled={!!isProcessing}
                  className="bg-[#333333] hover:bg-[#444444] disabled:opacity-50 text-white px-4 py-2 rounded-md font-bold transition-colors flex items-center justify-center gap-2"
                >
                  {isProcessing === 'dedicated' ? 'Loading...' : 'Upgrade to Dedicated ($99/mo)'}
                </button>
              </div>
            )}
            {billingPlan !== 'developer' && (
              <button className={`text-red-400 hover:text-red-300 px-4 py-2 rounded-md font-medium transition-colors border border-red-500/30 bg-red-500/10 ${billingPlan !== 'dedicated' ? 'mt-4' : ''}`}>
                Cancel Subscription
              </button>
            )}
          </div>

          <div className="bg-[#1A1A1A] border border-[#333333] rounded-xl overflow-hidden">
            <div className="p-6 border-b border-[#333333]">
              <h2 className="text-lg font-medium text-white">Invoices</h2>
            </div>
            <div className="divide-y divide-[#333333] p-6 text-sm text-gray-400">
              No recent invoices. Your payment history will appear here.
            </div>
          </div>
        </div>

        <div>
          <div className="bg-[#1A1A1A] border border-[#333333] rounded-xl p-6 sticky top-6">
            <h2 className="text-lg font-medium text-white mb-6">Payment Method</h2>
            
            <div className="bg-[#111111] border border-[#333333] rounded-lg p-4 flex items-center gap-4 mb-4">
              <div className="bg-gray-800 p-2 rounded">
                <CreditCard className="w-6 h-6 text-gray-400" />
              </div>
              <div className="flex-1">
                <div className="text-gray-400 font-medium">No fiat card on file</div>
              </div>
            </div>

            <button className="w-full bg-[#333333] hover:bg-[#444444] text-white py-2 rounded-md font-medium transition-colors mb-6 cursor-not-allowed opacity-50">
              Add Card via Stripe
            </button>

            <div className="pt-6 border-t border-[#333333]">
              <h3 className="text-sm font-medium text-white mb-4">Web3 Native Payment</h3>
              <p className="text-sm text-gray-400 mb-6">Pay pseudo-anonymously using Neo N3 GAS tokens via WalletConnect.</p>
              
              {billingPlan !== 'dedicated' ? (
                  <button 
                    onClick={() => setIsCryptoModalOpen(true)}
                    className="w-full bg-[#00E599]/10 hover:bg-[#00E599]/20 text-[#00E599] border border-[#00E599]/30 py-3 rounded-xl font-bold transition-colors flex items-center justify-center gap-2"
                  >
                    <div className="w-4 h-4 rounded-full bg-[#00E599] flex items-center justify-center text-black text-[10px] font-bold">N</div>
                    {billingPlan === 'developer' ? 'Pay with GAS' : 'Upgrade to Dedicated with GAS'}
                  </button>
              ) : (
                  <div className="text-sm text-green-400 border border-green-400/30 bg-green-400/10 p-3 rounded-lg text-center">
                      Subscription active via smart contract.
                  </div>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Crypto Payment Modal */}
      {isCryptoModalOpen && (
        <div className="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="bg-[#111111] border border-[#333333] rounded-2xl max-w-md w-full p-6 shadow-2xl relative">
            <button onClick={() => setIsCryptoModalOpen(false)} className="absolute top-4 right-4 text-gray-500 hover:text-white">✕</button>
            <h3 className="text-xl font-bold text-white mb-6 flex items-center gap-2">
              <div className="w-6 h-6 rounded-full bg-[#00E599] flex items-center justify-center text-black text-xs font-bold">N</div>
              Pay with Neo N3
            </h3>
            
            <div className="space-y-4 mb-6">
              {billingPlan === 'developer' && (
                <label className={`block border rounded-xl p-4 cursor-pointer transition-colors ${cryptoPlanSelected === 'growth' ? 'border-[#00E599] bg-[#00E599]/5' : 'border-[#333333] hover:border-gray-500'}`}>
                  <div className="flex items-center justify-between mb-1">
                    <div className="flex items-center gap-3">
                      <input type="radio" checked={cryptoPlanSelected === 'growth'} onChange={() => setCryptoPlanSelected('growth')} className="accent-[#00E599] w-4 h-4" />
                      <span className="font-bold text-white">Growth Plan</span>
                    </div>
                    <span className="text-[#00E599] font-medium">~15 GAS / mo</span>
                  </div>
                  <div className="pl-7 text-xs text-gray-400">Equivalent to $49 USD</div>
                </label>
              )}

              <label className={`block border rounded-xl p-4 cursor-pointer transition-colors ${cryptoPlanSelected === 'dedicated' ? 'border-[#00E599] bg-[#00E599]/5' : 'border-[#333333] hover:border-gray-500'}`}>
                <div className="flex items-center justify-between mb-1">
                  <div className="flex items-center gap-3">
                    <input type="radio" checked={cryptoPlanSelected === 'dedicated'} onChange={() => setCryptoPlanSelected('dedicated')} className="accent-[#00E599] w-4 h-4" />
                    <span className="font-bold text-white">Dedicated Plan</span>
                  </div>
                  <span className="text-[#00E599] font-medium">~30 GAS / mo</span>
                </div>
                <div className="pl-7 text-xs text-gray-400">Equivalent to $99 USD</div>
              </label>
            </div>

            <button 
              onClick={handleCryptoPayment}
              disabled={isProcessing === 'crypto'}
              className="w-full bg-[#00E599] hover:bg-[#00cc88] disabled:opacity-50 text-black py-3 rounded-xl font-bold transition-all shadow-[0_0_15px_rgba(0,229,153,0.2)] flex items-center justify-center gap-2"
            >
              {isProcessing === 'crypto' ? 'Waiting for wallet...' : 'Connect NeoLine / O3'}
            </button>
            <p className="text-xs text-center text-gray-500 mt-4">
              WalletConnect protocol will prompt you to sign a transaction.
            </p>
          </div>
        </div>
      )}

    </div>
  );
}