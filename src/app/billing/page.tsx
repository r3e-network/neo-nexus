import { CheckCircle2, CreditCard } from 'lucide-react';

export default function Billing() {
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
              <span className="text-4xl font-bold text-white">Pro</span>
              <span className="text-gray-400">/ $99/mo</span>
            </div>
            <p className="text-sm text-gray-300 mb-6 w-3/4">You are currently on the Pro plan. Includes 5M shared requests and priority support.</p>
            
            <div className="space-y-3 mb-8">
              <div className="flex items-center gap-2 text-sm text-gray-300">
                <CheckCircle2 className="w-4 h-4 text-[#00E599]" /> Up to 3 Dedicated Nodes
              </div>
              <div className="flex items-center gap-2 text-sm text-gray-300">
                <CheckCircle2 className="w-4 h-4 text-[#00E599]" /> Advanced Analytics
              </div>
              <div className="flex items-center gap-2 text-sm text-gray-300">
                <CheckCircle2 className="w-4 h-4 text-[#00E599]" /> Marketplace Plugins Access
              </div>
            </div>

            <div className="flex gap-4">
              <button className="bg-[#333333] hover:bg-[#444444] text-white px-4 py-2 rounded-md font-medium transition-colors">
                Change Plan
              </button>
              <button className="text-red-400 hover:text-red-300 px-4 py-2 rounded-md font-medium transition-colors">
                Cancel
              </button>
            </div>
          </div>

          <div className="bg-[#1A1A1A] border border-[#333333] rounded-xl overflow-hidden">
            <div className="p-6 border-b border-[#333333]">
              <h2 className="text-lg font-medium text-white">Invoices</h2>
            </div>
            <div className="divide-y divide-[#333333]">
              {[1, 2, 3].map((i) => (
                <div key={i} className="flex justify-between items-center p-6">
                  <div>
                    <div className="text-white font-medium">Invoice #NN-{2400 + i}</div>
                    <div className="text-sm text-gray-400">Oct {i * 5}, 2023</div>
                  </div>
                  <div className="flex items-center gap-6">
                    <span className="text-white font-medium">$99.00</span>
                    <button className="text-blue-400 hover:text-blue-300 text-sm">Download PDF</button>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>

        <div>
          <div className="bg-[#1A1A1A] border border-[#333333] rounded-xl p-6 sticky top-6">
            <h2 className="text-lg font-medium text-white mb-6">Payment Method</h2>
            
            <div className="bg-[#111111] border border-[#333333] rounded-lg p-4 flex items-center gap-4 mb-4">
              <div className="bg-gray-800 p-2 rounded">
                <CreditCard className="w-6 h-6 text-white" />
              </div>
              <div className="flex-1">
                <div className="text-white font-medium">•••• •••• •••• 4242</div>
                <div className="text-xs text-gray-400">Expires 12/25</div>
              </div>
            </div>

            <button className="w-full bg-[#333333] hover:bg-[#444444] text-white py-2 rounded-md font-medium transition-colors mb-6">
              Update Card
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
