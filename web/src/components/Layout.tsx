import { ReactNode, useState } from 'react';
import { Link, NavLink, useLocation } from 'react-router-dom';
import { 
  LayoutDashboard, 
  Server, 
  Puzzle,
  Settings,
  Network,
  Plug,
  Menu,
  Github,
  Activity,
  User,
  LogOut,
  Shield,
  Bell,
  CheckCircle2,
  AlertTriangle,
  AlertOctagon,
  Search,
  KeyRound,
  X
} from 'lucide-react';
import { useAuth } from '../hooks/useAuth';
import { useNotifications } from '../hooks/useNotifications';

interface LayoutProps {
  children: ReactNode;
}

const navItems = [
  { path: '/', icon: LayoutDashboard, label: 'Overview', description: 'Fleet health' },
  { path: '/nodes', icon: Server, label: 'Node Fleet', description: 'Lifecycle & config' },
  { path: '/servers', icon: Network, label: 'Servers', description: 'Host inventory' },
  { path: '/plugins', icon: Puzzle, label: 'Plugins', description: 'Node features' },
  { path: '/integrations', icon: Plug, label: 'Integrations', description: 'SaaS & agents' },
  { path: '/settings', icon: Settings, label: 'Settings', description: 'Security & audit' },
];

export default function Layout({ children }: LayoutProps) {
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [userMenuOpen, setUserMenuOpen] = useState(false);
  const [notificationsOpen, setNotificationsOpen] = useState(false);
  const location = useLocation();
  const { user, logout } = useAuth();
  const { notifications, unreadCount, dismissNotification, markAllRead, markNotificationRead } = useNotifications();

  const recentNotifications = notifications.slice(0, 8);
  const toastNotifications = notifications.filter((notification) => !notification.read).slice(0, 3);
  const secureSignerActive = location.pathname === '/settings' && location.hash === '#secure-signers';

  const iconForLevel = (level: string) => {
    if (level === 'error') return AlertOctagon;
    if (level === 'warning') return AlertTriangle;
    return CheckCircle2;
  };

  const colorForLevel = (level: string) => {
    if (level === 'error') return 'text-red-400';
    if (level === 'warning') return 'text-amber-400';
    if (level === 'success') return 'text-emerald-400';
    return 'text-blue-400';
  };

  return (
    <div className="min-h-screen flex">
      {/* Mobile sidebar overlay */}
      {sidebarOpen && (
        <div 
          className="fixed inset-0 bg-black/50 z-40 lg:hidden"
          onClick={() => setSidebarOpen(false)}
        />
      )}

      {/* Sidebar */}
      <aside 
        className={`
          fixed lg:static inset-y-0 left-0 z-50 w-72 border-r border-white/[0.07]
          bg-[linear-gradient(180deg,rgba(8,11,22,0.96),rgba(5,7,13,0.92))] backdrop-blur-2xl
          shadow-[16px_0_80px_rgba(0,0,0,0.28)]
          transform transition-transform duration-300 ease-in-out
          ${sidebarOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0'}
        `}
      >
        <div className="h-full flex flex-col">
          {/* Logo */}
          <div className="h-16 flex items-center px-6 border-b border-white/[0.07]">
            <div className="mr-3 flex h-10 w-10 items-center justify-center rounded-2xl border border-indigo-300/20 bg-[radial-gradient(circle_at_35%_20%,rgba(165,180,252,0.35),rgba(79,70,229,0.18)_42%,rgba(15,23,42,0.6))] shadow-[0_0_38px_rgba(113,112,255,0.24)]">
              <Activity className="w-6 h-6 text-indigo-100" />
            </div>
            <div>
              <h1 className="text-lg font-semibold tracking-[-0.03em]">
                <span className="bg-gradient-to-r from-white via-indigo-100 to-cyan-200 bg-clip-text text-transparent">NeoNexus</span>
              </h1>
              <p className="text-xs text-slate-500">Native Neo Control Plane</p>
            </div>
          </div>

          {/* Navigation */}
          <nav className="flex-1 py-5 px-3 space-y-1">
            <p className="px-3 pb-2 text-[11px] font-semibold uppercase tracking-[0.2em] text-slate-600">Operate</p>
            {navItems.map((item, i) => (
              <NavLink
                key={item.path}
                to={item.path}
                onClick={() => setSidebarOpen(false)}
                className={({ isActive }) => `
                  animate-fade-in stagger-${i + 1}
                  group relative flex items-center gap-3 overflow-hidden px-3 py-3 rounded-2xl text-sm transition-all
                  ${isActive
                    ? 'bg-white/[0.055] text-white ring-1 ring-indigo-300/20 shadow-[inset_3px_0_0_rgba(165,180,252,0.82),0_16px_36px_-28px_rgba(113,112,255,0.9)]'
                    : 'text-slate-400 hover:text-white hover:bg-white/[0.045] hover:ring-1 hover:ring-white/[0.06]'
                  }
                `}
              >
                <item.icon className="w-5 h-5 shrink-0" />
                <span className="min-w-0">
                  <span className="block font-medium leading-5">{item.label}</span>
                  <span className="block truncate text-xs text-slate-500 group-hover:text-slate-400">{item.description}</span>
                </span>
              </NavLink>
            ))}

            <div className="mx-3 my-5 border-t border-white/[0.07]" />
            <Link
              to="/settings#secure-signers"
              onClick={() => setSidebarOpen(false)}
              className={`mx-0 flex items-center gap-3 rounded-2xl border px-3 py-3 text-sm transition-all ${
                secureSignerActive
                  ? 'border-cyan-200/35 bg-[linear-gradient(135deg,rgba(34,211,238,0.14),rgba(113,112,255,0.08))] text-cyan-50 shadow-[0_18px_42px_-30px_rgba(34,211,238,0.85)]'
                  : 'border-transparent bg-transparent text-slate-500 hover:border-cyan-200/20 hover:bg-cyan-300/[0.05] hover:text-cyan-100'
              }`}
            >
              <KeyRound className="h-5 w-5" />
              <span>
                <span className="block font-medium">Private key vault</span>
                <span className="block text-xs text-cyan-200/60">TEE / HSM signer profiles</span>
              </span>
            </Link>
          </nav>

          {/* Footer */}
          <div className="p-4 border-t border-white/[0.07] space-y-3">
            <div className="rounded-2xl border border-emerald-300/15 bg-[linear-gradient(135deg,rgba(16,185,129,0.09),rgba(255,255,255,0.025))] p-3 shadow-[0_12px_36px_-30px_rgba(16,185,129,0.8)]">
              <p className="text-xs font-medium uppercase tracking-[0.16em] text-emerald-300/80">Console mode</p>
              <p className="mt-1 text-sm text-slate-300">Fail-closed security posture</p>
            </div>
            <a 
              href="https://github.com/r3e-network/neonexus"
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-2 text-slate-400 hover:text-white text-sm"
            >
              <Github className="w-4 h-4" />
              View on GitHub
            </a>
            <p className="text-xs text-slate-500">Version {__APP_VERSION__}</p>
          </div>
        </div>
      </aside>

      {/* Main content */}
      <div className="flex-1 flex flex-col min-w-0">
        {/* Header */}
        <header className="sticky top-0 z-30 h-16 flex items-center justify-between gap-4 border-b border-white/[0.07] bg-[rgba(5,7,13,0.72)] px-4 shadow-[0_18px_48px_-34px_rgba(0,0,0,0.9)] backdrop-blur-2xl lg:px-8">
          <div className="flex items-center lg:hidden">
            <button
              onClick={() => setSidebarOpen(true)}
              className="p-2 text-slate-400 hover:text-white"
            >
              <Menu className="w-6 h-6" />
            </button>
            <div className="ml-3 flex items-center">
              <Activity className="w-6 h-6 text-blue-500 mr-2" />
              <span className="font-bold text-white">NeoNexus</span>
            </div>
          </div>

          <div className="hidden min-w-0 flex-1 items-center gap-3 lg:flex">
            <div className="relative max-w-xl flex-1">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-500" />
              <div className="flex h-10 items-center justify-between rounded-2xl border border-white/[0.08] bg-white/[0.035] pl-10 pr-2 text-sm text-slate-500 shadow-[inset_0_1px_0_rgba(255,255,255,0.035)]">
                <span>Search nodes, plugins, signers, alerts...</span>
                <span className="command-pill">⌘K soon</span>
              </div>
            </div>
            <div className="hidden items-center gap-2 rounded-2xl border border-emerald-300/15 bg-emerald-300/5 px-3 py-2 text-xs text-slate-300 shadow-[0_10px_26px_-24px_rgba(16,185,129,0.9)] xl:flex">
              <span className="h-2 w-2 rounded-full bg-emerald-400 shadow-[0_0_12px_rgba(52,211,153,0.8)]" />
              Native-node safe mode
            </div>
          </div>

          {/* User Menu */}
          <div className="flex items-center gap-4 ml-auto">
            <div className="relative">
              <button
                onClick={() => {
                  const nextOpen = !notificationsOpen;
                  setNotificationsOpen(nextOpen);
                  if (nextOpen) {
                    markAllRead();
                  }
                }}
                className="relative p-2 rounded-xl text-slate-300 transition-colors hover:bg-white/[0.055] hover:text-white hover:ring-1 hover:ring-white/[0.08]"
              >
                <Bell className="w-5 h-5" />
                {unreadCount > 0 && (
                  <span className="absolute -top-1 -right-1 min-w-5 h-5 px-1 rounded-full bg-red-500 text-white text-xs flex items-center justify-center">
                    {unreadCount > 9 ? '9+' : unreadCount}
                  </span>
                )}
              </button>

              {notificationsOpen && (
                <>
                  <div
                    className="fixed inset-0 z-40"
                    onClick={() => setNotificationsOpen(false)}
                  />
                  <div className="animate-slide-down absolute right-0 mt-2 w-96 max-w-[calc(100vw-2rem)] bg-slate-900/80 backdrop-blur-xl rounded-xl shadow-2xl border border-slate-700/50 z-50 overflow-hidden">
                    <div className="p-3 border-b border-slate-700 flex items-center justify-between">
                      <div>
                        <p className="text-white font-medium">Notifications</p>
                        <p className="text-xs text-slate-400">Realtime node alerts and status changes</p>
                      </div>
                      <button
                        onClick={markAllRead}
                        className="text-xs text-blue-400 hover:text-blue-300"
                      >
                        Mark all read
                      </button>
                    </div>
                    <div className="max-h-96 overflow-y-auto">
                      {recentNotifications.length === 0 ? (
                        <div className="p-4 text-sm text-slate-400">No alerts yet.</div>
                      ) : (
                        recentNotifications.map((notification) => {
                          const Icon = iconForLevel(notification.level);
                          return (
                            <button
                              key={notification.id}
                              onClick={() => markNotificationRead(notification.id)}
                              className={`w-full text-left p-4 border-b border-slate-700/60 hover:bg-slate-700/40 transition-colors ${
                                notification.read ? 'opacity-70' : ''
                              }`}
                            >
                              <div className="flex items-start gap-3">
                                <Icon className={`w-4 h-4 mt-0.5 ${colorForLevel(notification.level)}`} />
                                <div className="min-w-0 flex-1">
                                  <div className="flex items-center justify-between gap-3">
                                    <p className="text-sm font-medium text-white">{notification.title}</p>
                                    <span className="text-xs text-slate-500">
                                      {new Date(notification.createdAt).toLocaleTimeString()}
                                    </span>
                                  </div>
                                  <p className="mt-1 text-sm text-slate-300 break-words">{notification.message}</p>
                                </div>
                              </div>
                            </button>
                          );
                        })
                      )}
                    </div>
                  </div>
                </>
              )}
            </div>

            <div className="relative">
              <button
                onClick={() => setUserMenuOpen(!userMenuOpen)}
                className="flex items-center gap-2 rounded-2xl border border-white/[0.08] bg-white/[0.035] px-3 py-2 text-slate-300 transition-colors hover:bg-white/[0.065] hover:text-white"
              >
                <div className="w-8 h-8 rounded-full bg-indigo-400/15 ring-1 ring-indigo-200/15 flex items-center justify-center">
                  <User className="w-4 h-4 text-indigo-200" />
                </div>
                <span className="hidden sm:block text-sm">{user?.username}</span>
                {user?.role === 'admin' && (
                  <Shield className="w-4 h-4 text-yellow-500" />
                )}
              </button>

              {/* Dropdown */}
              {userMenuOpen && (
                <>
                  <div 
                    className="fixed inset-0 z-40"
                    onClick={() => setUserMenuOpen(false)}
                  />
                  <div className="animate-slide-down absolute right-0 mt-2 w-48 bg-slate-900/80 backdrop-blur-xl rounded-xl shadow-2xl border border-slate-700/50 z-50">
                    <div className="p-3 border-b border-slate-700">
                      <p className="text-white font-medium">{user?.username}</p>
                      <p className="text-xs text-slate-400 capitalize">{user?.role}</p>
                    </div>
                    <button
                      onClick={() => {
                        setUserMenuOpen(false);
                        logout();
                      }}
                      className="w-full flex items-center gap-2 px-3 py-2 text-sm text-slate-300 hover:text-white hover:bg-slate-700 transition-colors"
                    >
                      <LogOut className="w-4 h-4" />
                      Logout
                    </button>
                  </div>
                </>
              )}
            </div>
          </div>
        </header>

        {/* Page content */}
        <main className="flex-1 overflow-auto p-5 lg:p-10">
          <div className="mx-auto max-w-[1600px]">
            {children}
          </div>
        </main>
      </div>

      <div className="fixed top-20 right-4 z-[70] space-y-3 w-80 max-w-[calc(100vw-2rem)] pointer-events-none">
        {toastNotifications.map((notification) => {
          const Icon = iconForLevel(notification.level);
          return (
            <div
              key={notification.id}
              className="pointer-events-auto rounded-xl border border-slate-700 bg-slate-900/95 shadow-xl backdrop-blur px-4 py-3 animate-slide-in-right"
            >
              <div className="flex items-start gap-3">
                <Icon className={`w-4 h-4 mt-0.5 ${colorForLevel(notification.level)}`} />
                <div className="min-w-0 flex-1">
                  <div className="flex items-center justify-between gap-3">
                    <p className="text-sm font-medium text-white">{notification.title}</p>
                    <button
                      onClick={() => dismissNotification(notification.id)}
                      className="text-slate-500 hover:text-white"
                    >
                      <X className="w-4 h-4" />
                    </button>
                  </div>
                  <p className="mt-1 text-sm text-slate-300 break-words">{notification.message}</p>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
