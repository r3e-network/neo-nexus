import Link from 'next/link';

export default function Footer() {
  return (
    <footer className="bg-[#050505] border-t border-white/5 pt-20 pb-10 mt-auto">
      <div className="max-w-7xl mx-auto px-6">
        <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 gap-10 mb-16">
          <div className="col-span-2 lg:col-span-2">
            <Link href="/" className="flex items-center gap-2 mb-4">
              <div className="w-8 h-8 rounded bg-[#00E599] flex items-center justify-center font-bold text-black text-lg">N</div>
              <span className="text-xl font-bold tracking-tight text-white">NeoNexus</span>
            </Link>
            <p className="text-gray-400 text-sm leading-relaxed mb-6 max-w-sm">
              The leading Web3 infrastructure provider for the Neo ecosystem. Scalable, reliable, and developer-friendly.
            </p>
          </div>
          
          <div>
            <h4 className="text-white font-bold mb-4">Products</h4>
            <ul className="space-y-3 text-sm text-gray-400">
              <li><Link href="/pricing" className="hover:text-[#00E599] transition-colors">Shared Endpoints</Link></li>
              <li><Link href="/pricing" className="hover:text-[#00E599] transition-colors">Dedicated Nodes</Link></li>
              <li><Link href="/developers" className="hover:text-[#00E599] transition-colors">Managed Plugins</Link></li>
              <li><Link href="/pricing" className="hover:text-[#00E599] transition-colors">Enterprise</Link></li>
            </ul>
          </div>

          <div>
            <h4 className="text-white font-bold mb-4">Developers</h4>
            <ul className="space-y-3 text-sm text-gray-400">
              <li><Link href="/docs" className="hover:text-[#00E599] transition-colors">Documentation</Link></li>
              <li><Link href="/developers" className="hover:text-[#00E599] transition-colors">Developer Hub</Link></li>
              <li><Link href="/docs" className="hover:text-[#00E599] transition-colors">Tutorials</Link></li>
              <li><a href="https://github.com/r3e-network/neonexus" target="_blank" rel="noreferrer" className="hover:text-[#00E599] transition-colors">GitHub</a></li>
            </ul>
          </div>

          <div>
            <h4 className="text-white font-bold mb-4">Company</h4>
            <ul className="space-y-3 text-sm text-gray-400">
              <li><Link href="/pricing" className="hover:text-[#00E599] transition-colors">Pricing</Link></li>
              <li><Link href="/" className="hover:text-[#00E599] transition-colors">About</Link></li>
              <li><span className="text-gray-600">Blog (Soon)</span></li>
              <li><span className="text-gray-600">Careers (Soon)</span></li>
            </ul>
          </div>
        </div>

        <div className="flex flex-col md:flex-row items-center justify-between pt-8 border-t border-white/10 text-xs text-gray-500">
          <p>© {new Date().getFullYear()} NeoNexus Inc. All rights reserved.</p>
          <div className="flex gap-6 mt-4 md:mt-0">
            <span className="text-gray-600">Privacy Policy (Soon)</span>
            <span className="text-gray-600">Terms of Service (Soon)</span>
            <span className="text-gray-600">Cookie Policy (Soon)</span>
          </div>
        </div>
      </div>
    </footer>
  );
}
