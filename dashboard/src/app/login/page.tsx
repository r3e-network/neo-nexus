'use client';

import Link from 'next/link';
import { useSearchParams } from 'next/navigation';
import { Suspense, useState, useEffect } from 'react';
import { signIn } from 'next-auth/react';
import toast from 'react-hot-toast';

interface NeoLineTxResult { txid: string }
interface WindowWithNeoLine extends Window {
  NEOLineN3?: {
    Init: new () => {
      getAccount: () => Promise<{ address: string }>;
      signMessage: (args: { message: string }) => Promise<{ publicKey: string, data: string, salt: string, message: string }>;
      send: (args: { fromAddress: string, toAddress: string, asset: string, amount: string, network: string }) => Promise<NeoLineTxResult>;
    };
  };
}

function LoginContent() {
  const searchParams = useSearchParams();
  const isSignup = searchParams.get('signup') === 'true';
  const callbackUrl = searchParams.get('callbackUrl') || '/app';
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [hasNeoLine, setHasNeoLine] = useState<boolean>(false);

  useEffect(() => {
    // Check if NeoLine is injected
    if (typeof window !== 'undefined') {
      const checkNeoLine = () => {
        const win = window as unknown as WindowWithNeoLine;
        if (win.NEOLineN3) {
          setHasNeoLine(true);
        }
      };
      checkNeoLine();
      // Sometimes it takes a moment to inject
      setTimeout(checkNeoLine, 500);
      setTimeout(checkNeoLine, 1000);
    }
  }, []);

  const handleNeoLogin = async () => {
    setIsLoading(true);
    
    try {
      const win = window as unknown as WindowWithNeoLine;
      if (!win.NEOLineN3) {
        throw new Error('NeoLine wallet not found. Please install the NeoLine extension.');
      }

      const neoline = new win.NEOLineN3.Init();
      const account = await neoline.getAccount();
      
      const message = `Sign in to NeoNexus\nNonce: ${Date.now()}`;
      
      const signed = await neoline.signMessage({
        message,
      });

      const result = await signIn('credentials', {
        redirect: false,
        address: account.address,
        publicKey: signed.publicKey,
        message: message,
        signature: signed.data,
      });

      if (result?.error) {
        throw new Error(result.error);
      }

      // Success, redirect
      window.location.href = callbackUrl;

    } catch (error: unknown) {
      console.error(error);
      const errObj = error as Record<string, unknown>;
      const errorMessage = (typeof errObj?.description === 'string' ? errObj.description : null) || 
                           (typeof errObj?.message === 'string' ? errObj.message : null) || 
                           'Failed to authenticate with Neo wallet.';
      toast.error(errorMessage);
      setIsLoading(false);
    }
  };

  return (
    <div className="w-full max-w-md bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-3xl p-8 md:p-12 shadow-2xl relative overflow-hidden">
      {/* Glow effect */}
      <div className="absolute top-0 right-0 w-32 h-32 bg-[#00E599]/10 rounded-full blur-[50px]"></div>

      <div className="text-center mb-8 relative z-10">
        <div className="inline-flex items-center gap-2 mb-6">
          <div className="w-10 h-10 rounded bg-[#00E599] flex items-center justify-center font-bold text-black text-xl">N</div>
        </div>
        <h1 className="text-3xl font-bold text-white mb-2">
          {isSignup ? 'Create an account' : 'Welcome back'}
        </h1>
        <p className="text-gray-400">
          Sign in using your Neo N3 wallet to access the control plane.
        </p>
      </div>

      <div className="space-y-4 relative z-10">
        {!hasNeoLine && (
          <div className="bg-yellow-500/10 border border-yellow-500/20 text-yellow-500 text-sm p-4 rounded-xl mb-4 text-left">
            NeoLine wallet extension not detected. Please install NeoLine for your browser to continue.
          </div>
        )}

        <button 
          onClick={handleNeoLogin}
          disabled={isLoading || !hasNeoLine}
          className="w-full bg-[#00E599] hover:bg-[#00cc88] disabled:opacity-50 text-black py-3 rounded-xl font-bold transition-all shadow-[0_0_15px_rgba(0,229,153,0.2)] flex items-center justify-center gap-2"
        >
          {isLoading ? (
            <span className="animate-spin w-5 h-5 border-2 border-black border-t-transparent rounded-full"></span>
          ) : (
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M12 22C17.5228 22 22 17.5228 22 12C22 6.47715 17.5228 2 12 2C6.47715 2 2 6.47715 2 12C2 17.5228 6.47715 22 12 22Z" stroke="currentColor" strokeWidth="2"/><path d="M7 12H17M12 7V17" stroke="currentColor" strokeWidth="2" strokeLinecap="round"/></svg>
          )}
          {isSignup ? 'Connect Wallet to Sign Up' : 'Connect Wallet to Sign In'}
        </button>
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
    <div className="min-h-screen flex items-center justify-center bg-[var(--color-dark-bg)] py-20 px-6 w-full absolute top-0 left-0 z-50">
      <Suspense fallback={<div className="text-white">Loading...</div>}>
        <LoginContent />
      </Suspense>
    </div>
  );
}