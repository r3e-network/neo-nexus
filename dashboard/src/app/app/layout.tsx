import Sidebar from '@/components/Sidebar';
import AuthCheck from '@/components/AuthCheck';
import { Toaster } from 'react-hot-toast';
import { getCurrentUserContext } from '@/server/organization';

export default async function DashboardLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const userContext = await getCurrentUserContext();

  return (
    <div className="flex h-screen overflow-hidden bg-[var(--color-dark-panel)] text-white">
      <Toaster position="top-right" toastOptions={{ style: { background: '#333', color: '#fff' } }} />
      <Sidebar showOperations={userContext?.role === 'operator'} />
      <main className="flex-1 overflow-y-auto bg-[var(--color-dark-panel)]">
        <div className="mx-auto max-w-7xl px-8 py-8">
          <AuthCheck>
            {children}
          </AuthCheck>
        </div>
      </main>
    </div>
  );
}
