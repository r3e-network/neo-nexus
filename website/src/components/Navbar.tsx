'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { Menu, X, ChevronRight } from 'lucide-react';
import { useState, useEffect } from 'react';

export default function Navbar() {
  const [isScrolled, setIsScrolled] = useState(false);
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const pathname = usePathname();

  useEffect(() => {
    const handleScroll = () => {
      setIsScrolled(window.scrollY > 20);
    };
    window.addEventListener('scroll', handleScroll);
    return () => window.removeEventListener('scroll', handleScroll);
  }, []);

  const navLinks = [
    { name: 'Products', href: '/#products' },
    { name: 'Developers', href: '/developers' },
    { name: 'Pricing', href: '/pricing' },
    { name: 'Docs', href: '/docs' },
  ];

  return (
    <header className={`fixed top-0 left-0 right-0 z-50 transition-all duration-300 ${isScrolled ? 'bg-[#0A0A0A]/80 backdrop-blur-md border-b border-white/10 py-3' : 'bg-transparent py-5'}`}>
      <div className="max-w-7xl mx-auto px-6 flex items-center justify-between">
        {/* Logo */}
        <Link href="/" className="flex items-center gap-2 group">
          <div className="w-8 h-8 rounded bg-[#00E599] flex items-center justify-center font-bold text-black text-lg transition-transform group-hover:scale-105">N</div>
          <span className="text-xl font-bold tracking-tight text-white">NeoNexus</span>
        </Link>

        {/* Desktop Nav */}
        <nav className="hidden md:flex items-center gap-8">
          {navLinks.map((link) => (
            <Link 
              key={link.name} 
              href={link.href}
              className={`text-sm font-medium transition-colors ${pathname === link.href ? 'text-[#00E599]' : 'text-gray-300 hover:text-white'}`}
            >
              {link.name}
            </Link>
          ))}
        </nav>

        {/* Auth Buttons */}
        <div className="hidden md:flex items-center gap-4">
          <Link href="/app/login" className="text-sm font-medium text-gray-300 hover:text-white transition-colors">
            Log in
          </Link>
          <Link href="/app/login?signup=true" className="bg-[#00E599] hover:bg-[#00cc88] text-black px-5 py-2 rounded-full text-sm font-bold transition-all hover:shadow-[0_0_15px_rgba(0,229,153,0.3)] flex items-center gap-1">
            Start for free <ChevronRight className="w-4 h-4" />
          </Link>
        </div>

        {/* Mobile Menu Toggle */}
        <button className="md:hidden text-white" onClick={() => setMobileMenuOpen(!mobileMenuOpen)}>
          {mobileMenuOpen ? <X className="w-6 h-6" /> : <Menu className="w-6 h-6" />}
        </button>
      </div>

      {/* Mobile Menu */}
      {mobileMenuOpen && (
        <div className="md:hidden absolute top-full left-0 right-0 bg-[#111111] border-b border-white/10 p-6 flex flex-col gap-4 shadow-xl">
          {navLinks.map((link) => (
            <Link 
              key={link.name} 
              href={link.href}
              onClick={() => setMobileMenuOpen(false)}
              className="text-base font-medium text-gray-300 hover:text-white"
            >
              {link.name}
            </Link>
          ))}
          <div className="h-px bg-white/10 my-2"></div>
          <Link href="/app/login" className="text-base font-medium text-gray-300 hover:text-white" onClick={() => setMobileMenuOpen(false)}>
            Log in
          </Link>
          <Link href="/app/login?signup=true" className="bg-[#00E599] text-black px-5 py-3 rounded-lg text-center font-bold" onClick={() => setMobileMenuOpen(false)}>
            Start for free
          </Link>
        </div>
      )}
    </header>
  );
}
