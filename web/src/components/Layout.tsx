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
  KeyRound,
  Sparkles,
  X
} from 'lucide-react';
import { useAuth } from '../hooks/useAuth';
import { useNotifications } from '../hooks/useNotifications';
import { PROJECT_LINKS } from '../config/constants';

interface LayoutProps {
  children: ReactNode;
}

const navItems = [
  { path: '/', icon: LayoutDashboard, label: 'Overview' },
  { path: '/nodes', icon: Server, label: 'Nodes' },
  { path: '/servers', icon: Network, label: 'Servers' },
  { path: '/plugins', icon: Puzzle, label: 'Plugins' },
  { path: '/integrations', icon: Plug, label: 'Integrations' },
  { path: '/agent', icon: Sparkles, label: 'Hermes Agent' },
  { path: '/settings', icon: Settings, label: 'Settings' },
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
    if (level === 'error') return 'text-red-600';
    if (level === 'warning') return 'text-amber-600';
    if (level === 'success') return 'text-emerald-600';
    return 'text-blue-600';
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
        id="app-sidebar"
        className={`
          fixed lg:static inset-y-0 left-0 z-50 w-[224px] border-r border-slate-200
          bg-white shadow-xl shadow-slate-950/10 lg:shadow-none
          transform transition-transform duration-300 ease-in-out
          ${sidebarOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0'}
        `}
      >
        <div className="h-full flex flex-col">
          {/* Logo */}
          <div className="h-16 flex items-center px-4 border-b border-slate-200">
            <div className="mr-3 flex h-9 w-9 items-center justify-center rounded-lg bg-teal-700 text-white">
              <Activity className="w-5 h-5" />
            </div>
            <div>
              <h1 className="text-base font-semibold text-slate-950">
                NeoNexus
              </h1>
              <p className="text-xs text-slate-500">Operations console</p>
            </div>
          </div>

          {/* Navigation */}
          <nav className="flex-1 px-3 py-4 space-y-1">
            <p className="px-2 pb-2 text-[11px] font-semibold uppercase text-slate-400">Operate</p>
            {navItems.map((item, i) => (
              <NavLink
                key={item.path}
                to={item.path}
                onClick={() => setSidebarOpen(false)}
                className={({ isActive }) => `
                  animate-fade-in stagger-${i + 1}
                  group relative flex items-center gap-3 overflow-hidden px-3 py-2.5 rounded-lg text-sm font-medium transition-all
                  ${isActive
                    ? 'bg-teal-50 text-teal-950 ring-1 ring-teal-100 shadow-[inset_3px_0_0_#0f766e]'
                    : 'text-slate-600 hover:bg-slate-100 hover:text-slate-950'
                  }
                `}
              >
                <item.icon className="w-5 h-5 shrink-0" />
                <span className="min-w-0 truncate">{item.label}</span>
              </NavLink>
            ))}

            <div className="mx-2 my-4 border-t border-slate-200" />
            <Link
              to="/settings#secure-signers"
              onClick={() => setSidebarOpen(false)}
              className={`mx-0 flex items-center gap-3 rounded-lg border px-3 py-2.5 text-sm transition-all ${
                secureSignerActive
                  ? 'border-teal-200 bg-teal-50 text-teal-950'
                  : 'border-transparent bg-transparent text-slate-600 hover:bg-slate-100 hover:text-slate-950'
              }`}
            >
              <KeyRound className="h-5 w-5" />
              <span className="font-medium">Key vault</span>
            </Link>
          </nav>

          {/* Footer */}
          <div className="p-4 border-t border-slate-200 space-y-2">
            <a
              href={PROJECT_LINKS.repositoryUrl}
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-2 text-sm text-slate-600 hover:text-slate-950"
            >
              <Github className="w-4 h-4" />
              GitHub
            </a>
            <p className="text-xs text-slate-500">Version {__APP_VERSION__}</p>
          </div>
        </div>
      </aside>

      {/* Main content */}
      <div className="flex-1 flex flex-col min-w-0">
        {/* Header */}
        <header className="sticky top-0 z-30 h-16 flex items-center justify-between gap-4 border-b border-slate-200 bg-white/95 px-4 backdrop-blur lg:px-6">
          <div className="flex items-center lg:hidden">
            <button
              onClick={() => setSidebarOpen(true)}
              className="rounded-lg p-2 text-slate-600 hover:bg-slate-100 hover:text-slate-950"
              aria-label="Open navigation menu"
              aria-controls="app-sidebar"
              aria-expanded={sidebarOpen}
            >
              <Menu className="w-6 h-6" />
            </button>
            <div className="ml-3 flex items-center">
              <Activity className="w-6 h-6 text-teal-700 mr-2" />
              <span className="font-semibold text-slate-950">NeoNexus</span>
            </div>
          </div>

          <div className="hidden min-w-0 flex-1 items-center gap-3 lg:flex">
            <div className="flex items-center gap-2 rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 text-xs font-medium text-slate-600">
              <span className="h-2 w-2 rounded-full bg-emerald-500" />
              Local control plane
            </div>
            <div className="hidden items-center gap-2 rounded-lg border border-teal-100 bg-teal-50 px-3 py-2 text-xs font-medium text-teal-900 xl:flex">
              <Shield className="h-4 w-4" />
              Fail-closed defaults
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
                className="relative rounded-lg p-2 text-slate-600 transition-colors hover:bg-slate-100 hover:text-slate-950"
                aria-label={unreadCount > 0 ? `Open notifications (${unreadCount} unread)` : 'Open notifications'}
                aria-controls="notifications-panel"
                aria-expanded={notificationsOpen}
                aria-haspopup="dialog"
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
                  <div
                    id="notifications-panel"
                    className="animate-slide-down absolute right-0 mt-2 w-96 max-w-[calc(100vw-2rem)] overflow-hidden rounded-lg border border-slate-200 bg-white shadow-xl z-50"
                  >
                    <div className="p-3 border-b border-slate-200 flex items-center justify-between">
                      <div>
                        <p className="text-slate-950 font-medium">Notifications</p>
                        <p className="text-xs text-slate-500">Realtime node alerts and status changes</p>
                      </div>
                      <button
                        onClick={markAllRead}
                        className="text-xs font-medium text-blue-600 hover:text-blue-800"
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
                              className={`w-full text-left p-4 border-b border-slate-100 transition-colors hover:bg-slate-50 ${
                                notification.read ? 'opacity-70' : ''
                              }`}
                            >
                              <div className="flex items-start gap-3">
                                <Icon className={`w-4 h-4 mt-0.5 ${colorForLevel(notification.level)}`} />
                                <div className="min-w-0 flex-1">
                                  <div className="flex items-center justify-between gap-3">
                                    <p className="text-sm font-medium text-slate-950">{notification.title}</p>
                                    <span className="text-xs text-slate-500">
                                      {new Date(notification.createdAt).toLocaleTimeString()}
                                    </span>
                                  </div>
                                  <p className="mt-1 text-sm text-slate-600 break-words">{notification.message}</p>
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
                className="flex items-center gap-2 rounded-lg border border-slate-200 bg-white px-3 py-2 text-slate-700 transition-colors hover:bg-slate-50 hover:text-slate-950"
                aria-label={user?.username ? `Open user menu for ${user.username}` : 'Open user menu'}
                aria-controls="user-menu"
                aria-expanded={userMenuOpen}
                aria-haspopup="menu"
              >
                <div className="w-8 h-8 rounded-md bg-teal-50 ring-1 ring-teal-100 flex items-center justify-center">
                  <User className="w-4 h-4 text-teal-700" />
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
                  <div id="user-menu" className="animate-slide-down absolute right-0 mt-2 w-48 rounded-lg border border-slate-200 bg-white shadow-xl z-50">
                    <div className="p-3 border-b border-slate-200">
                      <p className="text-slate-950 font-medium">{user?.username}</p>
                      <p className="text-xs text-slate-500 capitalize">{user?.role}</p>
                    </div>
                    <button
                      onClick={() => {
                        setUserMenuOpen(false);
                        logout();
                      }}
                      className="w-full flex items-center gap-2 px-3 py-2 text-sm text-slate-700 transition-colors hover:bg-slate-50 hover:text-slate-950"
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
        <main className="flex-1 overflow-auto p-4 lg:p-6">
          <div className="mx-auto max-w-[1440px]">
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
              className="pointer-events-auto rounded-lg border border-slate-200 bg-white px-4 py-3 shadow-xl animate-slide-in-right"
            >
              <div className="flex items-start gap-3">
                <Icon className={`w-4 h-4 mt-0.5 ${colorForLevel(notification.level)}`} />
                <div className="min-w-0 flex-1">
                  <div className="flex items-center justify-between gap-3">
                    <p className="text-sm font-medium text-slate-950">{notification.title}</p>
                    <button
                      onClick={() => dismissNotification(notification.id)}
                      className="text-slate-500 hover:text-slate-950"
                      aria-label={`Dismiss notification: ${notification.title}`}
                    >
                      <X className="w-4 h-4" />
                    </button>
                  </div>
                  <p className="mt-1 text-sm text-slate-600 break-words">{notification.message}</p>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
