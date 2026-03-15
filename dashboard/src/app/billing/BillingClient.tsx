'use client';

import { CheckCircle2, CreditCard, ChevronRight } from 'lucide-react';
import { useState } from 'react';
import { upgradePlanAction } from './actions';
import toast from 'react-hot-toast';

export default function BillingClient({ billingPlan }: { billingPlan: string }) {
  const [isProcessing, setIsProcessing] = useState<string | null>(null);

  const handleUpgrade = async (plan: 'growth' | 'dedicated') => {
    setIsProcessing(plan);
    try {
      await upgradePlanAction(plan);
      // upgradePlanAction redirects, so we shouldn't reach here if successful
    } catch (error: any) {
      toast.error(error.message || 'Failed to start checkout session');
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

            {billingPlan === 'developer' && (
              <div className="flex gap-4">
                <button 
                  onClick={() => handleUpgrade('growth')}
                  disabled={!!isProcessing}
                  className="bg-[#00E599] hover:bg-[#00cc88] disabled:opacity-50 text-black px-4 py-2 rounded-md font-bold transition-colors flex items-center gap-2"
                >
                  {isProcessing === 'growth' ? 'Loading...' : 'Upgrade to Growth ($49/mo)'}
                </button>
                <button 
                  onClick={() => handleUpgrade('dedicated')}
                  disabled={!!isProcessing}
                  className="bg-[#333333] hover:bg-[#444444] disabled:opacity-50 text-white px-4 py-2 rounded-md font-bold transition-colors flex items-center gap-2"
                >
                  {isProcessing === 'dedicated' ? 'Loading...' : 'Upgrade to Dedicated ($99/mo)'}
                </button>
              </div>
            )}
            {billingPlan !== 'developer' && (
              <button className="text-red-400 hover:text-red-300 px-4 py-2 rounded-md font-medium transition-colors border border-red-500/30 bg-red-500/10">
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
                <div className="text-gray-400 font-medium">No card on file</div>
              </div>
            </div>

            <button className="w-full bg-[#333333] hover:bg-[#444444] text-white py-2 rounded-md font-medium transition-colors mb-6 cursor-not-allowed opacity-50">
              Add Card via Stripe
            </button>

            <div className="pt-6 border-t border-[#333333]">
              <h3 className="text-sm font-medium text-white mb-4">Crypto Top-up</h3>
              <p className="text-sm text-gray-400 mb-4">Pay anonymously using NEP-17 GAS tokens.</p>
              <button className="w-full bg-[#00E599]/10 hover:bg-[#00E599]/20 text-[#00E599] border border-[#00E599]/30 py-2 rounded-md font-medium transition-colors">
                Pay with GAS
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
