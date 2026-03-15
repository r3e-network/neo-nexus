'use client';

import Link from 'next/link';
import { Github } from 'lucide-react';
import { useSearchParams } from 'next/navigation';
import { Suspense, useState } from 'react';
import { signIn } from 'next-auth/react';

function LoginContent() {
  const searchParams = useSearchParams();
  const isSignup = searchParams.get('signup') === 'true';
  // Point to dashboard in production, else local dashboard port
  const callbackUrl = process.env.NEXT_PUBLIC_DASHBOARD_URL || 'http://localhost:3001';
  const [isLoading, setIsLoading] = useState<string | null>(null);

  const handleLogin = async (provider: string) => {
    setIsLoading(provider);
    await signIn(provider, { callbackUrl });
  };

  return (
    <div className="w-full max-w-md bg-[#111111] border border-[#333333] rounded-3xl p-8 md:p-12 shadow-2xl relative overflow-hidden">
      
      {/* Glow effect */}
      <div className="absolute top-0 right-0 w-32 h-32 bg-[#00E599]/10 rounded-full blur-[50px]"></div>

      <div className="text-center mb-8 relative z-10">
        <Link href="/" className="inline-flex items-center gap-2 mb-6">
          <div className="w-10 h-10 rounded bg-[#00E599] flex items-center justify-center font-bold text-black text-xl">N</div>
        </Link>
        <h1 className="text-3xl font-bold text-white mb-2">
          {isSignup ? 'Create an account' : 'Welcome back'}
        </h1>
        <p className="text-gray-400">
          {isSignup ? 'Start building on Neo N3 in seconds.' : 'Sign in to your NeoNexus console.'}
        </p>
      </div>

      <div className="space-y-4 relative z-10">
        <button 
          onClick={() => handleLogin('github')}
          disabled={!!isLoading}
          className="w-full bg-white text-black hover:bg-gray-200 disabled:opacity-50 py-3 rounded-xl font-bold transition-colors flex items-center justify-center gap-3"
        >
          {isLoading === 'github' ? <span className="animate-spin w-5 h-5 border-2 border-black border-t-transparent rounded-full"></span> : <Github className="w-5 h-5" />}
          {isSignup ? 'Sign up with GitHub' : 'Continue with GitHub'}
        </button>
        
        <button 
          onClick={() => handleLogin('google')}
          disabled={!!isLoading}
          className="w-full bg-[#1A1A1A] border border-[#333333] hover:border-gray-500 disabled:opacity-50 text-white py-3 rounded-xl font-bold transition-colors flex items-center justify-center gap-3"
        >
          {isLoading === 'google' ? (
            <span className="animate-spin w-5 h-5 border-2 border-white border-t-transparent rounded-full"></span>
          ) : (
            <svg viewBox="0 0 24 24" width="20" height="20" xmlns="http://www.w3.org/2000/svg">
              <path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z" fill="#4285F4"/>
              <path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853"/>
              <path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" fill="#FBBC05"/>
              <path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" fill="#EA4335"/>
            </svg>
          )}
          {isSignup ? 'Sign up with Google' : 'Continue with Google'}
        </button>

        <div className="relative py-4">
          <div className="absolute inset-0 flex items-center">
            <div className="w-full border-t border-[#333333]"></div>
          </div>
          <div className="relative flex justify-center text-xs">
            <span className="bg-[#111111] px-4 text-gray-500">Or</span>
          </div>
        </div>

        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-400 mb-1">Email Address</label>
            <input 
              type="email" 
              placeholder="you@company.com" 
              className="w-full bg-[#0A0A0A] border border-[#333333] rounded-xl px-4 py-3 text-white focus:outline-none focus:border-[#00E599] transition-colors"
            />
          </div>
          <button className="w-full bg-[#00E599] hover:bg-[#00cc88] text-black py-3 rounded-xl font-bold transition-all shadow-[0_0_15px_rgba(0,229,153,0.2)]">
            {isSignup ? 'Create Account' : 'Sign In'}
          </button>
        </div>
      </div>

      <p className="text-center text-sm text-gray-500 mt-8 relative z-10">
        {isSignup ? 'Already have an account?' : "Don't have an account?"}
        <Link href={isSignup ? '/login' : '/login?signup=true'} className="text-[#00E599] hover:underline ml-2 font-medium">
          {isSignup ? 'Sign in' : 'Sign up'}
        </Link>
      </p>

    </div>
  );
}

export default function Login() {
  return (
    <div className="min-h-screen flex items-center justify-center bg-[#0A0A0A] py-20 px-6 absolute top-0 left-0 w-full z-50">
      <Suspense fallback={<div className="text-white">Loading...</div>}>
        <LoginContent />
      </Suspense>
    </div>
  );
}
