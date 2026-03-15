import Sidebar from "@/components/Sidebar";
import AuthCheck from "@/components/AuthCheck";
import { Toaster } from "react-hot-toast";

export default function DashboardLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <div className="flex h-screen overflow-hidden bg-[#1A1C23] text-white">
      <Toaster position="top-right" toastOptions={{ style: { background: '#333', color: '#fff' } }} />
      <Sidebar />
      <main className="flex-1 overflow-y-auto bg-[#1A1C23]">
        <div className="mx-auto max-w-7xl px-8 py-8">
          <AuthCheck>
            {children}
          </AuthCheck>
        </div>
      </main>
    </div>
  );
}
