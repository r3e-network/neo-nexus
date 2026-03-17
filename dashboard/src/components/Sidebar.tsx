'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { LayoutDashboard, Server, BarChart3, Puzzle, Shield, CreditCard, Wrench } from 'lucide-react';

const baseNavigation = [
  { name: 'Overview', href: '/app', icon: LayoutDashboard },
  { name: 'Endpoints', href: '/app/endpoints', icon: Server },
  { name: 'Analytics', href: '/app/analytics', icon: BarChart3 },
  { name: 'Marketplace', href: '/app/marketplace', icon: Puzzle },
  { name: 'Security', href: '/app/security', icon: Shield },
  { name: 'Billing', href: '/app/billing', icon: CreditCard },
];

type SidebarProps = {
  showOperations: boolean;
};

export default function Sidebar({ showOperations }: SidebarProps) {
  const pathname = usePathname();
  const navigation = showOperations
    ? [...baseNavigation, { name: 'Operations', href: '/app/operations', icon: Wrench }]
    : baseNavigation;

  return (
    <div className="flex h-screen w-64 flex-col bg-[var(--color-dark-panel)] border-r border-[var(--color-dark-border)] shrink-0">
      <div className="flex h-16 shrink-0 items-center px-6 border-b border-[var(--color-dark-border)]">
        <div className="flex items-center gap-2">
          <div className="w-8 h-8 rounded bg-[#00E599] flex items-center justify-center font-bold text-black">N</div>
          <span className="text-xl font-bold tracking-tight text-white">NeoNexus</span>
        </div>
      </div>
      <nav className="flex flex-1 flex-col px-4 py-6 space-y-2">
        {navigation.map((item) => {
          const isActive = item.href === '/app'
            ? pathname === '/app'
            : pathname === item.href || pathname.startsWith(`${item.href}/`);
          return (
            <Link
              key={item.name}
              href={item.href}
              className={`flex items-center gap-3 px-3 py-2 rounded-md transition-colors ${
                isActive 
                  ? 'bg-[#333333] text-[#00E599]' 
                  : 'text-gray-400 hover:text-white hover:bg-[#252525]'
              }`}
            >
              <item.icon className="h-5 w-5 shrink-0" />
              <span className="text-sm font-medium">{item.name}</span>
            </Link>
          );
        })}
      </nav>
      <div className="p-4 border-t border-[var(--color-dark-border)]">
        <div className="flex items-center gap-3 px-3 py-2 rounded-md hover:bg-[#252525] cursor-pointer">
          <div className="w-8 h-8 rounded-full bg-gradient-to-tr from-[#00E599] to-blue-500"></div>
          <div className="flex flex-col">
            <span className="text-sm font-medium text-white">Neo Dev</span>
            <span className="text-xs text-gray-400">Pro Plan</span>
          </div>
        </div>
      </div>
    </div>
  );
}
