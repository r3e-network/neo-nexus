import { ReactNode, useState } from 'react';
import { NavLink } from 'react-router-dom';
import { 
  LayoutDashboard, 
  Server, 
  Puzzle, 
  Settings, 
  Network,
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
  X
} from 'lucide-react';
import { useAuth } from '../hooks/useAuth';
import { useNotifications } from '../hooks/useNotifications';

interface LayoutProps {
  children: ReactNode;
}

const navItems = [
  { path: '/', icon: LayoutDashboard, label: 'Dashboard' },
  { path: '/nodes', icon: Server, label: 'Nodes' },
  { path: '/servers', icon: Network, label: 'Servers' },
  { path: '/plugins', icon: Puzzle, label: 'Features' },
  { path: '/settings', icon: Settings, label: 'Settings' },
];

export default function Layout({ children }: LayoutProps) {
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [userMenuOpen, setUserMenuOpen] = useState(false);
  const [notificationsOpen, setNotificationsOpen] = useState(false);
  const { user, logout } = useAuth();
  const { notifications, unreadCount, dismissNotification, markAllRead, markNotificationRead } = useNotifications();

  const recentNotifications = notifications.slice(0, 8);
  const toastNotifications = notifications.filter((notification) => !notification.read).slice(0, 3);

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
          fixed lg:static inset-y-0 left-0 z-50 w-64 bg-slate-900 border-r border-slate-800
          transform transition-transform duration-200 ease-in-out
          ${sidebarOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0'}
        `}
      >
        <div className="h-full flex flex-col">
          {/* Logo */}
          <div className="h-16 flex items-center px-6 border-b border-slate-800">
            <Activity className="w-8 h-8 text-blue-500 mr-3" />
            <div>
              <h1 className="text-lg font-bold">
              <span className="bg-gradient-to-r from-blue-400 to-cyan-400 bg-clip-text text-transparent">NeoNexus</span>
            </h1>
              <p className="text-xs text-slate-400">Node Manager</p>
            </div>
          </div>

          {/* Navigation */}
          <nav className="flex-1 py-4 px-3 space-y-1">
            {navItems.map((item, i) => (
              <NavLink
                key={item.path}
                to={item.path}
                onClick={() => setSidebarOpen(false)}
                className={({ isActive }) => `
                  animate-fade-in stagger-${i + 1}
                  flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors
                  ${isActive
                    ? 'bg-blue-500/10 text-blue-400'
                    : 'text-slate-400 hover:text-white hover:bg-slate-800'
                  }
                `}
              >
                <item.icon className="w-5 h-5" />
                {item.label}
              </NavLink>
            ))}
          </nav>

          {/* Footer */}
          <div className="p-4 border-t border-slate-800">
            <a 
              href="https://github.com/r3e-network/neonexus"
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-2 text-slate-400 hover:text-white text-sm"
            >
              <Github className="w-4 h-4" />
              View on GitHub
            </a>
            <p className="mt-2 text-xs text-slate-500">Version {__APP_VERSION__}</p>
          </div>
        </div>
      </aside>

      {/* Main content */}
      <div className="flex-1 flex flex-col min-w-0">
        {/* Header */}
        <header className="h-16 flex items-center justify-between px-4 lg:px-8 border-b border-slate-800 bg-slate-900">
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
                className="relative p-2 rounded-lg text-slate-300 hover:text-white hover:bg-slate-800 transition-colors"
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
                  <div className="animate-slide-down absolute right-0 mt-2 w-96 max-w-[calc(100vw-2rem)] bg-slate-800 rounded-lg shadow-lg border border-slate-700 z-50 overflow-hidden">
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
                className="flex items-center gap-2 px-3 py-2 rounded-lg text-slate-300 hover:text-white hover:bg-slate-800 transition-colors"
              >
                <div className="w-8 h-8 rounded-full bg-blue-500/20 flex items-center justify-center">
                  <User className="w-4 h-4 text-blue-400" />
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
                  <div className="animate-slide-down absolute right-0 mt-2 w-48 bg-slate-800 rounded-lg shadow-lg border border-slate-700 z-50">
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
        <main className="flex-1 p-4 lg:p-8 overflow-auto animate-fade-in">
          {children}
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
