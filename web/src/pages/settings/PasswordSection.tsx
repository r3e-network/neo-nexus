import { useState, type FormEvent } from "react";
import { Loader2, Lock } from "lucide-react";
import { FeedbackBanner } from "../../components/FeedbackBanner";
import { ApiRequestError } from "../../utils/api";
import { useAuth } from "../../hooks/useAuth";

export function PasswordSection() {
  const { changePassword, isChangingPassword } = useAuth();
  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [passwordError, setPasswordError] = useState("");
  const [passwordSuggestion, setPasswordSuggestion] = useState("");
  const [passwordCode, setPasswordCode] = useState("");
  const [passwordSuccess, setPasswordSuccess] = useState("");

  const handlePasswordSubmit = async (event: FormEvent) => {
    event.preventDefault();
    setPasswordError("");
    setPasswordSuggestion("");
    setPasswordCode("");
    setPasswordSuccess("");

    if (!currentPassword || !newPassword || !confirmPassword) {
      setPasswordError("All password fields are required.");
      return;
    }

    if (newPassword.length < 8) {
      setPasswordError("New password must be at least 8 characters.");
      return;
    }

    if (newPassword !== confirmPassword) {
      setPasswordError("New passwords do not match.");
      return;
    }

    try {
      await changePassword(currentPassword, newPassword);
      setCurrentPassword("");
      setNewPassword("");
      setConfirmPassword("");
      setPasswordSuccess("Password updated successfully.");
    } catch (error) {
      if (error instanceof ApiRequestError) {
        setPasswordError(error.message);
        setPasswordSuggestion(error.suggestion ?? "");
        setPasswordCode(error.code ?? "");
      } else {
        setPasswordError(error instanceof Error ? error.message : "Failed to update password.");
        setPasswordSuggestion("");
        setPasswordCode("");
      }
    }
  };

  return (
    <div className="card">
      <div className="flex items-center gap-3 mb-4">
        <div className="w-10 h-10 rounded-lg bg-emerald-500/10 flex items-center justify-center">
          <Lock className="w-5 h-5 text-emerald-400" />
        </div>
        <div>
          <h2 className="text-lg font-semibold text-white">Change Password</h2>
          <p className="text-slate-400 text-sm">Update the administrator password used to access NeoNexus</p>
        </div>
      </div>

      <FeedbackBanner error={passwordError} suggestion={passwordSuggestion} code={passwordCode} success={passwordSuccess} />

      <form className="grid grid-cols-1 gap-4 md:grid-cols-3" onSubmit={handlePasswordSubmit}>
        <div>
          <label className="mb-2 block text-sm font-medium text-slate-300">Current Password</label>
          <input
            type="password"
            value={currentPassword}
            onChange={(event) => setCurrentPassword(event.target.value)}
            className="input"
            placeholder="Current password"
          />
        </div>
        <div>
          <label className="mb-2 block text-sm font-medium text-slate-300">New Password</label>
          <input
            type="password"
            value={newPassword}
            onChange={(event) => setNewPassword(event.target.value)}
            className="input"
            placeholder="At least 8 characters"
          />
        </div>
        <div>
          <label className="mb-2 block text-sm font-medium text-slate-300">Confirm Password</label>
          <input
            type="password"
            value={confirmPassword}
            onChange={(event) => setConfirmPassword(event.target.value)}
            className="input"
            placeholder="Repeat new password"
          />
        </div>

        <div className="md:col-span-3 flex justify-end">
          <button className="btn btn-primary" disabled={isChangingPassword} type="submit">
            {isChangingPassword ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Lock className="w-4 h-4" />
            )}
            {isChangingPassword ? "Updating..." : "Change Password"}
          </button>
        </div>
      </form>
    </div>
  );
}
