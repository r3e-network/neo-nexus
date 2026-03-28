import { useState, type FormEvent } from "react";
import { Users, Trash2, ChevronDown, ChevronUp, UserPlus } from "lucide-react";
import { FeedbackBanner } from "../../components/FeedbackBanner";
import { useAuth } from "../../hooks/useAuth";
import { useUsers, useCreateUser, useDeleteUser } from "../../hooks/useUsers";

function formatCreatedAt(ts?: number): string {
  if (!ts) return "—";
  return new Date(ts * 1000).toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

export function UserManagement() {
  const { user: currentUser } = useAuth();
  const { data: users = [], isLoading } = useUsers();
  const createUser = useCreateUser();
  const deleteUser = useDeleteUser();

  // Add user form state
  const [showAddForm, setShowAddForm] = useState(false);
  const [newUsername, setNewUsername] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [newRole, setNewRole] = useState<"admin" | "viewer">("viewer");
  const [formError, setFormError] = useState("");
  const [formSuccess, setFormSuccess] = useState("");

  // Delete confirmation state
  const [pendingDeleteId, setPendingDeleteId] = useState<string | null>(null);
  const [deleteError, setDeleteError] = useState("");

  const handleAddUser = async (event: FormEvent) => {
    event.preventDefault();
    setFormError("");
    setFormSuccess("");

    if (!newUsername.trim() || !newPassword.trim()) {
      setFormError("Username and password are required.");
      return;
    }

    if (newPassword.length < 8) {
      setFormError("Password must be at least 8 characters.");
      return;
    }

    try {
      await createUser.mutateAsync({ username: newUsername.trim(), password: newPassword, role: newRole });
      setFormSuccess(`User "${newUsername.trim()}" created successfully.`);
      setNewUsername("");
      setNewPassword("");
      setNewRole("viewer");
      setShowAddForm(false);
    } catch (error) {
      setFormError(error instanceof Error ? error.message : "Failed to create user.");
    }
  };

  const handleDeleteConfirm = async () => {
    if (!pendingDeleteId) return;
    setDeleteError("");
    try {
      await deleteUser.mutateAsync(pendingDeleteId);
      setPendingDeleteId(null);
    } catch (error) {
      setDeleteError(error instanceof Error ? error.message : "Failed to delete user.");
    }
  };

  const pendingDeleteUser = users.find((u) => u.id === pendingDeleteId);

  return (
    <div className="card">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-lg bg-indigo-500/10 flex items-center justify-center">
            <Users className="w-5 h-5 text-indigo-400" />
          </div>
          <div>
            <h2 className="text-lg font-semibold text-white">User Management</h2>
            <p className="text-slate-400 text-sm">Manage accounts that can access NeoNexus</p>
          </div>
        </div>
        <button
          className="btn btn-secondary flex items-center gap-2 text-sm"
          onClick={() => {
            setShowAddForm((v) => !v);
            setFormError("");
            setFormSuccess("");
          }}
        >
          <UserPlus className="w-4 h-4" />
          Add User
          {showAddForm ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
        </button>
      </div>

      {/* Global feedback */}
      <FeedbackBanner error={deleteError} success={formSuccess} />

      {/* Collapsible add user form */}
      {showAddForm && (
        <div className="mb-4 p-4 bg-slate-800/50 rounded-lg border border-slate-700/50">
          <h3 className="text-sm font-medium text-slate-300 mb-3">New User</h3>
          <FeedbackBanner error={formError} />
          <form className="grid grid-cols-1 md:grid-cols-4 gap-3 mt-3" onSubmit={handleAddUser}>
            <div>
              <label className="mb-1 block text-xs font-medium text-slate-400">Username</label>
              <input
                type="text"
                value={newUsername}
                onChange={(e) => setNewUsername(e.target.value)}
                className="input w-full"
                placeholder="username"
                autoComplete="off"
              />
            </div>
            <div>
              <label className="mb-1 block text-xs font-medium text-slate-400">Password</label>
              <input
                type="password"
                value={newPassword}
                onChange={(e) => setNewPassword(e.target.value)}
                className="input w-full"
                placeholder="At least 8 characters"
              />
            </div>
            <div>
              <label className="mb-1 block text-xs font-medium text-slate-400">Role</label>
              <select
                value={newRole}
                onChange={(e) => setNewRole(e.target.value as "admin" | "viewer")}
                className="input w-full"
              >
                <option value="viewer">Viewer</option>
                <option value="admin">Admin</option>
              </select>
            </div>
            <div className="flex items-end">
              <button
                type="submit"
                className="btn btn-primary w-full"
                disabled={createUser.isPending}
              >
                {createUser.isPending ? "Creating..." : "Create User"}
              </button>
            </div>
          </form>
        </div>
      )}

      {/* User table */}
      {isLoading ? (
        <div className="text-slate-400 text-sm py-4 text-center">Loading users...</div>
      ) : users.length === 0 ? (
        <div className="text-slate-400 text-sm py-4 text-center">No users found.</div>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-slate-700/50">
                <th className="pb-2 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">Username</th>
                <th className="pb-2 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">Role</th>
                <th className="pb-2 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">Created</th>
                <th className="pb-2 text-right text-xs font-medium text-slate-400 uppercase tracking-wider">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-700/30">
              {users.map((u) => {
                const isSelf = u.id === currentUser?.id;
                return (
                  <tr key={u.id} className="group">
                    <td className="py-3 text-white font-medium">{u.username}</td>
                    <td className="py-3">
                      <span
                        className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${
                          u.role === "admin"
                            ? "bg-indigo-500/15 text-indigo-300"
                            : "bg-slate-500/20 text-slate-300"
                        }`}
                      >
                        {u.role}
                      </span>
                    </td>
                    <td className="py-3 text-slate-400">{formatCreatedAt(u.createdAt)}</td>
                    <td className="py-3 text-right">
                      <div className="relative inline-block group/del">
                        <button
                          onClick={() => {
                            setDeleteError("");
                            setPendingDeleteId(u.id);
                          }}
                          disabled={isSelf}
                          className={`p-1.5 rounded transition-colors ${
                            isSelf
                              ? "opacity-30 cursor-not-allowed text-slate-500"
                              : "text-slate-400 hover:text-red-400 hover:bg-red-400/10"
                          }`}
                          aria-label={isSelf ? "Cannot delete your own account" : `Delete ${u.username}`}
                        >
                          <Trash2 className="w-4 h-4" />
                        </button>
                        {isSelf && (
                          <span className="pointer-events-none absolute right-full top-1/2 -translate-y-1/2 mr-2 hidden group-hover/del:block whitespace-nowrap rounded bg-slate-700 px-2 py-1 text-xs text-slate-200 shadow-lg z-10">
                            Cannot delete your own account
                          </span>
                        )}
                      </div>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}

      {/* Delete confirmation dialog */}
      {pendingDeleteId && pendingDeleteUser && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
          <div className="card max-w-sm w-full mx-4 border border-slate-700 shadow-2xl">
            <h3 className="text-base font-semibold text-white mb-2">Delete User</h3>
            <p className="text-slate-300 text-sm mb-4">
              Are you sure you want to delete user{" "}
              <span className="font-medium text-white">"{pendingDeleteUser.username}"</span>? This action cannot be
              undone.
            </p>
            <FeedbackBanner error={deleteError} />
            <div className="flex justify-end gap-3 mt-4">
              <button
                className="btn btn-secondary"
                onClick={() => {
                  setPendingDeleteId(null);
                  setDeleteError("");
                }}
                disabled={deleteUser.isPending}
              >
                Cancel
              </button>
              <button
                className="btn bg-red-600 hover:bg-red-700 text-white border-0"
                onClick={handleDeleteConfirm}
                disabled={deleteUser.isPending}
              >
                {deleteUser.isPending ? "Deleting..." : "Delete User"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
